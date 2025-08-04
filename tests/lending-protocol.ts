import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LendingCore } from "../target/types/lending_core";
import { Keypair, LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";

describe("lending-protocol", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.LendingCore as Program<LendingCore>;
  const authority = provider.wallet;

  const lendingPool = Keypair.generate();
  const user = Keypair.generate();

  let userAccountPda: anchor.web3.PublicKey;
  let solVaultPda: anchor.web3.PublicKey;
  let userAccountBump: number;
  let solVaultBump: number;


  before(async () => {
    await provider.connection.requestAirdrop(user.publicKey, 10 * LAMPORTS_PER_SOL);

    [userAccountPda, userAccountBump] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_account"), user.publicKey.toBuffer()],
      program.programId
    );

    [solVaultPda, solVaultBump] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("sol_vault"), lendingPool.publicKey.toBuffer()],
      program.programId
    );
  });

  it("Is initialized!", async () => {

    await program.methods
      .initialize()
      .accounts({
        lendingPool: lendingPool.publicKey,
        authority: authority.publicKey,
      })
      .signers([lendingPool])
      .rpc();

    const poolState = await program.account.lendingPool.fetch(lendingPool.publicKey);
    assert.ok(poolState.authority.equals(authority.publicKey));
    assert.equal(poolState.totalSolDeposited.toNumber(), 0);
    assert.equal(poolState.totalStablecoinBorrowed.toNumber(), 0);
  });

  it("Deposits SOL collateral", async () => {
    const depositAmount = new anchor.BN(2 * LAMPORTS_PER_SOL); // Deposit 2 SOL

    await program.methods
      .deposit(depositAmount)
      .accounts({
        lendingPool: lendingPool.publicKey,
        userAccount: userAccountPda,
        solVault: solVaultPda,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const userAccountState = await program.account.userAccount.fetch(userAccountPda);
    assert.ok(userAccountState.owner.equals(user.publicKey));
    assert.equal(userAccountState.solDeposited.toString(), depositAmount.toString());

    const vaultBalance = await provider.connection.getBalance(solVaultPda);
    assert.equal(vaultBalance, depositAmount.toNumber());
  });

  it("Borrows stablecoin against collateral", async () => {
    // Based on 2 SOL collateral at $100/SOL, and 75% LTV
    // Max borrow = 2 * 100 * 0.75 = $150
    const borrowAmount = new anchor.BN(150);

    await program.methods
      .borrow(borrowAmount)
      .accounts({
        lendingPool: lendingPool.publicKey,
        userAccount: userAccountPda,
      })
      .signers([user])
      .rpc();

    const userAccountState = await program.account.userAccount.fetch(userAccountPda);
    assert.equal(userAccountState.stablecoinDebt.toString(), borrowAmount.toString());

    const poolState = await program.account.lendingPool.fetch(lendingPool.publicKey);
    assert.equal(poolState.totalStablecoinBorrowed.toString(), borrowAmount.toString());
  });
  
  it("Fails to borrow more than LTV ratio", async () => {
    // Try to borrow $1 more, which should fail
    const borrowAmount = new anchor.BN(1);
    try {
      await program.methods
        .borrow(borrowAmount)
        .accounts({
          lendingPool: lendingPool.publicKey,
          userAccount: userAccountPda,
        })
        .signers([user])
        .rpc();
      assert.fail("Should have failed to borrow beyond LTV limit.");
    } catch (err) {
      assert.equal(err.error.errorCode.code, "LtvExceeded");
    }
  });

  it("Repays the loan with interest", async () => {
    // Current debt is 150. Interest is 5%.
    // Interest due = 150 * 0.05 = 7.5 (which is 7 in integer math)
    // Total due = 150 + 7 = 157
    const repayAmount = new anchor.BN(157);

    await program.methods
      .repay(repayAmount)
      .accounts({
        lendingPool: lendingPool.publicKey,
        userAccount: userAccountPda,
        owner: user.publicKey,
      })
      .signers([user])
      .rpc();

    const userAccountState = await program.account.userAccount.fetch(userAccountPda);
    assert.equal(userAccountState.stablecoinDebt.toNumber(), 0);

    const poolState = await program.account.lendingPool.fetch(lendingPool.publicKey);
    assert.equal(poolState.totalStablecoinBorrowed.toNumber(), 0);
  });

  it("Withdraws SOL collateral", async () => {
    const withdrawAmount = new anchor.BN(1 * LAMPORTS_PER_SOL); // Withdraw 1 SOL
    const initialUserBalance = await provider.connection.getBalance(user.publicKey);

    await program.methods
      .withdraw(withdrawAmount)
      .accounts({
        lendingPool: lendingPool.publicKey,
        userAccount: userAccountPda,
        solVault: solVaultPda,
        owner: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const userAccountState = await program.account.userAccount.fetch(userAccountPda);
    // Should have 1 SOL remaining (2 SOL deposited - 1 SOL withdrawn)
    assert.equal(userAccountState.solDeposited.toString(), (new anchor.BN(1 * LAMPORTS_PER_SOL)).toString());

    const finalUserBalance = await provider.connection.getBalance(user.publicKey);
    // Check if user's balance increased by roughly the withdrawn amount (minus gas fees)
    assert.ok(finalUserBalance > initialUserBalance);
  });

  it("Fails to withdraw if it causes undercollateralization", async () => {
      // First, let's borrow again. Deposit is 1 SOL ($100). Max borrow is $75.
      const borrowAmount = new anchor.BN(75);
      await program.methods
        .borrow(borrowAmount)
        .accounts({
          lendingPool: lendingPool.publicKey,
          userAccount: userAccountPda,
          owner: user.publicKey,
        })
        .signers([user])
        .rpc();

      // Now, try to withdraw the remaining 1 SOL. This should fail.
      const withdrawAmount = new anchor.BN(1 * LAMPORTS_PER_SOL);
      try {
        await program.methods
          .withdraw(withdrawAmount)
          .accounts({
            lendingPool: lendingPool.publicKey,
            userAccount: userAccountPda,
            solVault: solVaultPda,
            owner: user.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();
        assert.fail("Should have failed to withdraw due to undercollateralization.");
      } catch (err) {
        assert.equal(err.error.errorCode.code, "Undercollateralized");
      }
  });
});
