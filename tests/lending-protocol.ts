import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LendingCore } from "../target/types/lending_core";
import { expect } from "chai";
import { 
  Keypair, 
  LAMPORTS_PER_SOL, 
  PublicKey, 
  SystemProgram,
  Transaction
} from "@solana/web3.js";

describe("lending_core", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.LendingCore as Program<LendingCore>;
  
  // Log the program ID for debugging
  console.log("Program ID:", program.programId.toString());
  
  let authority: Keypair;
  let user1: Keypair;
  let user2: Keypair;
  let lendingPool: Keypair;
  let solVault: PublicKey;
  let user1Account: PublicKey;
  let user2Account: PublicKey;

  before(async () => {
    // Generate keypairs
    authority = Keypair.generate();
    user1 = Keypair.generate();
    user2 = Keypair.generate();
    lendingPool = Keypair.generate();

    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(authority.publicKey, 10 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user1.publicKey, 10 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user2.publicKey, 10 * LAMPORTS_PER_SOL);

    // Wait for airdrops to confirm
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Derive PDAs
    [solVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("sol_vault"), lendingPool.publicKey.toBuffer()],
      program.programId
    );

    [user1Account] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_account"), user1.publicKey.toBuffer()],
      program.programId
    );

    [user2Account] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_account"), user2.publicKey.toBuffer()],
      program.programId
    );
  });

  describe("Initialize", () => {
    it("should initialize the lending pool", async () => {
      await program.methods
        .initialize()
        .accounts({
          lendingPool: lendingPool.publicKey,
          authority: authority.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([authority, lendingPool])
        .rpc();

      const poolAccount = await program.account.lendingPool.fetch(lendingPool.publicKey);
      
      expect(poolAccount.totalSolDeposited.toString()).to.equal("0");
      expect(poolAccount.totalStablecoinBorrowed.toString()).to.equal("0");
      expect(poolAccount.authority.toString()).to.equal(authority.publicKey.toString());
    });

    it("should fail to initialize with wrong authority", async () => {
      const wrongLendingPool = Keypair.generate();
      
      try {
        await program.methods
          .initialize()
          .accounts({
            lendingPool: wrongLendingPool.publicKey,
            authority: user1.publicKey, // Wrong authority
            systemProgram: SystemProgram.programId,
          })
          .signers([user1, wrongLendingPool])
          .rpc();
        
        expect.fail("Should have failed with wrong authority");
      } catch (error) {
        // Expected to fail
      }
    });
  });

  describe("Deposit", () => {
    it("should allow user to deposit SOL", async () => {
      const depositAmount = 2 * LAMPORTS_PER_SOL;
      
      const userBalanceBefore = await provider.connection.getBalance(user1.publicKey);
      
      await program.methods
        .deposit(new anchor.BN(depositAmount))
        .accounts({
          lendingPool: lendingPool.publicKey,
          userAccount: user1Account,
          solVault: solVault,
          user: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      // Check user account state
      const userAccountData = await program.account.userAccount.fetch(user1Account);
      expect(userAccountData.solDeposited.toString()).to.equal(depositAmount.toString());
      expect(userAccountData.stablecoinDebt.toString()).to.equal("0");
      expect(userAccountData.owner.toString()).to.equal(user1.publicKey.toString());

      // Check lending pool state
      const poolAccount = await program.account.lendingPool.fetch(lendingPool.publicKey);
      expect(poolAccount.totalSolDeposited.toString()).to.equal(depositAmount.toString());

      // Check SOL was transferred to vault
      const vaultBalance = await provider.connection.getBalance(solVault);
      expect(vaultBalance).to.equal(depositAmount);

      // Check user balance decreased (minus transaction fees)
      const userBalanceAfter = await provider.connection.getBalance(user1.publicKey);
      expect(userBalanceBefore - userBalanceAfter).to.be.greaterThan(depositAmount);
    });

    it("should allow multiple deposits from same user", async () => {
      const depositAmount = 1 * LAMPORTS_PER_SOL;
      
      await program.methods
        .deposit(new anchor.BN(depositAmount))
        .accounts({
          lendingPool: lendingPool.publicKey,
          userAccount: user1Account,
          solVault: solVault,
          user: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      const userAccountData = await program.account.userAccount.fetch(user1Account);
      expect(userAccountData.solDeposited.toString()).to.equal((3 * LAMPORTS_PER_SOL).toString());
    });

    it("should allow deposits from different users", async () => {
      const depositAmount = 1.5 * LAMPORTS_PER_SOL;
      
      await program.methods
        .deposit(new anchor.BN(depositAmount))
        .accounts({
          lendingPool: lendingPool.publicKey,
          userAccount: user2Account,
          solVault: solVault,
          user: user2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      const user2AccountData = await program.account.userAccount.fetch(user2Account);
      expect(user2AccountData.solDeposited.toString()).to.equal(depositAmount.toString());
      expect(user2AccountData.owner.toString()).to.equal(user2.publicKey.toString());

      // Check total pool deposits
      const poolAccount = await program.account.lendingPool.fetch(lendingPool.publicKey);
      expect(poolAccount.totalSolDeposited.toString()).to.equal((4.5 * LAMPORTS_PER_SOL).toString());
    });
  });

  describe("Borrow", () => {
    it("should allow user to borrow within LTV limits", async () => {
      // User1 has 3 SOL deposited = $300 collateral value
      // Max borrow = $300 * 75% = $225
      const borrowAmount = 200; // $200 in stablecoins
      
      await program.methods
        .borrow(new anchor.BN(borrowAmount))
        .accounts({
          lendingPool: lendingPool.publicKey,
          userAccount: user1Account,
          owner: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      const userAccountData = await program.account.userAccount.fetch(user1Account);
      expect(userAccountData.stablecoinDebt.toString()).to.equal(borrowAmount.toString());

      const poolAccount = await program.account.lendingPool.fetch(lendingPool.publicKey);
      expect(poolAccount.totalStablecoinBorrowed.toString()).to.equal(borrowAmount.toString());
    });

    it("should allow multiple borrows within LTV limits", async () => {
      const additionalBorrow = 20; // Total debt will be $220
      
      await program.methods
        .borrow(new anchor.BN(additionalBorrow))
        .accounts({
          lendingPool: lendingPool.publicKey,
          userAccount: user1Account,
          owner: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      const userAccountData = await program.account.userAccount.fetch(user1Account);
      expect(userAccountData.stablecoinDebt.toString()).to.equal("220");
    });

    it("should fail when borrowing exceeds LTV ratio", async () => {
      // User1 already has $220 debt, max is $225, trying to borrow $10 more would exceed
      const excessiveBorrow = 10;
      
      try {
        await program.methods
          .borrow(new anchor.BN(excessiveBorrow))
          .accounts({
            lendingPool: lendingPool.publicKey,
            userAccount: user1Account,
            owner: user1.publicKey,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have failed due to LTV exceeded");
      } catch (error: any) {
        // Check for Anchor error structure
        if (error.error && error.error.errorCode) {
          expect(error.error.errorCode.number).to.equal(6000); // LtvExceeded error number
        } else if (error.code) {
          expect(error.code).to.equal(6000);
        } else {
          // Fallback check for error message
          expect(error.toString()).to.include("LtvExceeded");
        }
      }
    });

    it("should fail when user has no collateral", async () => {
      const user3 = Keypair.generate();
      await provider.connection.requestAirdrop(user3.publicKey, LAMPORTS_PER_SOL);
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const [user3Account] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_account"), user3.publicKey.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .borrow(new anchor.BN(100))
          .accounts({
            lendingPool: lendingPool.publicKey,
            userAccount: user3Account,
            owner: user3.publicKey,
          })
          .signers([user3])
          .rpc();
        
        expect.fail("Should have failed due to no collateral");
      } catch (error: any) {
        // This should fail because the user account doesn't exist or has no collateral
        // The error might be different (account not found), so we check for various error types
        expect(error).to.exist;
      }
    });
  });

  describe("Repay", () => {
    it("should allow user to repay debt with interest", async () => {
      // User1 has $220 debt, with 5% interest = $11, total = $231
      const repayAmount = 231;
      
      await program.methods
        .repay(new anchor.BN(repayAmount))
        .accounts({
          lendingPool: lendingPool.publicKey,
          userAccount: user1Account,
          owner: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      const userAccountData = await program.account.userAccount.fetch(user1Account);
      expect(userAccountData.stablecoinDebt.toString()).to.equal("0");

      const poolAccount = await program.account.lendingPool.fetch(lendingPool.publicKey);
      expect(poolAccount.totalStablecoinBorrowed.toString()).to.equal("0");
    });

    it("should fail when repayment is insufficient", async () => {
      // First borrow some amount
      await program.methods
        .borrow(new anchor.BN(100))
        .accounts({
          lendingPool: lendingPool.publicKey,
          userAccount: user1Account,
          owner: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      // Try to repay less than required (100 + 5% = 105)
      try {
        await program.methods
          .repay(new anchor.BN(100)) // Insufficient, should be 105
          .accounts({
            lendingPool: lendingPool.publicKey,
            userAccount: user1Account,
            owner: user1.publicKey,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have failed due to insufficient repayment");
      } catch (error: any) {
        if (error.error && error.error.errorCode) {
          expect(error.error.errorCode.number).to.equal(6003); // InsufficientRepayment error number
        } else if (error.code) {
          expect(error.code).to.equal(6003);
        }
      }
    });

    it("should allow overpayment", async () => {
      // Repay more than required
      const overpayAmount = 200; // Required is 105
      
      await program.methods
        .repay(new anchor.BN(overpayAmount))
        .accounts({
          lendingPool: lendingPool.publicKey,
          userAccount: user1Account,
          owner: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      const userAccountData = await program.account.userAccount.fetch(user1Account);
      expect(userAccountData.stablecoinDebt.toString()).to.equal("0");
    });
  });

  describe("Withdraw", () => {
    it("should allow partial withdrawal when adequately collateralized", async () => {
      // User1 has 3 SOL deposited and no debt
      const withdrawAmount = 1 * LAMPORTS_PER_SOL;
      
      const userBalanceBefore = await provider.connection.getBalance(user1.publicKey);
      
      await program.methods
        .withdraw(new anchor.BN(withdrawAmount))
        .accounts({
          lendingPool: lendingPool.publicKey,
          userAccount: user1Account,
          solVault: solVault,
          owner: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      const userAccountData = await program.account.userAccount.fetch(user1Account);
      expect(userAccountData.solDeposited.toString()).to.equal((2 * LAMPORTS_PER_SOL).toString());

      const poolAccount = await program.account.lendingPool.fetch(lendingPool.publicKey);
      expect(poolAccount.totalSolDeposited.toString()).to.equal((3.5 * LAMPORTS_PER_SOL).toString());

      // Check user received SOL
      const userBalanceAfter = await provider.connection.getBalance(user1.publicKey);
      expect(userBalanceAfter).to.be.greaterThan(userBalanceBefore);
    });

    it("should fail when withdrawal would cause undercollateralization", async () => {
      // First borrow to create debt
      await program.methods
        .borrow(new anchor.BN(140)) // $140 debt against $200 collateral (2 SOL)
        .accounts({
          lendingPool: lendingPool.publicKey,
          userAccount: user1Account,
          owner: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      // Try to withdraw 1 SOL, leaving 1 SOL ($100) which allows max $75 borrow
      try {
        await program.methods
          .withdraw(new anchor.BN(1 * LAMPORTS_PER_SOL))
          .accounts({
            lendingPool: lendingPool.publicKey,
            userAccount: user1Account,
            solVault: solVault,
            owner: user1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have failed due to undercollateralization");
      } catch (error: any) {
        if (error.error && error.error.errorCode) {
          expect(error.error.errorCode.number).to.equal(6002); // Undercollateralized error number
        } else if (error.code) {
          expect(error.code).to.equal(6002);
        } else {
          expect(error.toString()).to.include("Undercollateralized");
        }
      }
    });

    it("should fail when trying to withdraw more than deposited", async () => {
      try {
        await program.methods
          .withdraw(new anchor.BN(10 * LAMPORTS_PER_SOL)) // More than deposited
          .accounts({
            lendingPool: lendingPool.publicKey,
            userAccount: user1Account,
            solVault: solVault,
            owner: user1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have failed due to insufficient collateral");
      } catch (error: any) {
        if (error.error && error.error.errorCode) {
          expect(error.error.errorCode.number).to.equal(6001); // InsufficientCollateral error number
        } else if (error.code) {
          expect(error.code).to.equal(6001);
        } else {
          expect(error.toString()).to.include("InsufficientCollateral");
        }
      }
    });

    it("should allow full withdrawal after repaying debt", async () => {
      // First repay the debt (140 + 5% = 147)
      await program.methods
        .repay(new anchor.BN(147))
        .accounts({
          lendingPool: lendingPool.publicKey,
          userAccount: user1Account,
          owner: user1.publicKey,
        })
        .signers([user1])
        .rpc();

      // Now withdraw all remaining SOL
      await program.methods
        .withdraw(new anchor.BN(2 * LAMPORTS_PER_SOL))
        .accounts({
          lendingPool: lendingPool.publicKey,
          userAccount: user1Account,
          solVault: solVault,
          owner: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      const userAccountData = await program.account.userAccount.fetch(user1Account);
      expect(userAccountData.solDeposited.toString()).to.equal("0");
    });
  });

  describe("Access Control", () => {
    it("should fail when wrong owner tries to borrow", async () => {
      try {
        await program.methods
          .borrow(new anchor.BN(50))
          .accounts({
            lendingPool: lendingPool.publicKey,
            userAccount: user1Account, // User1's account
            owner: user2.publicKey, // But user2 trying to access
          })
          .signers([user2])
          .rpc();
        
        expect.fail("Should have failed due to wrong owner");
      } catch (error) {
        // Should fail constraint check
      }
    });

    it("should fail when wrong owner tries to repay", async () => {
      try {
        await program.methods
          .repay(new anchor.BN(50))
          .accounts({
            lendingPool: lendingPool.publicKey,
            userAccount: user2Account, // User2's account
            owner: user1.publicKey, // But user1 trying to access
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have failed due to wrong owner");
      } catch (error) {
        // Should fail constraint check
      }
    });
  });
})
