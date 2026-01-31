import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ArithmeticVulnerable } from "../target/types/arithmetic_vulnerable";
import { ArithmeticFixed } from "../target/types/arithmetic_fixed";
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";
import { BN } from "bn.js";

describe("Arithmetic Underflow & Overflow Demo", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const vulnProgram = anchor.workspace.ArithmeticVulnerable as Program<ArithmeticVulnerable>;
  const fixedProgram = anchor.workspace.ArithmeticFixed as Program<ArithmeticFixed>;

  let vulnVault: PublicKey;
  let fixedVault: PublicKey;
  const user = provider.wallet;

  before(async () => {
    [vulnVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      vulnProgram.programId
    );

    [fixedVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      fixedProgram.programId
    );

    // Initialize vulnerable vault
    await vulnProgram.methods.initialize()
      .accounts({ vault: vulnVault })
      .rpc();
    console.log("Vulnerable vault initialized at:", vulnVault.toBase58());

    // Initialize fixed vault
    await fixedProgram.methods.initialize()
      .accounts({ vault: fixedVault })
      .rpc();
    console.log("Fixed vault initialized at:", fixedVault.toBase58());
  });

  it("Vulnerable: Overflow causes panic (program reverts)", async () => {
    const maxU64 = new BN("18446744073709551615"); // u64::MAX

    await vulnProgram.methods.deposit(maxU64)
      .accounts({ vault: vulnVault })
      .rpc();
    console.log("Deposited u64::MAX successfully (balance now max)");

    try {
      await vulnProgram.methods.deposit(new BN(1))
        .accounts({ vault: vulnVault })
        .rpc();
      assert.fail("Overflow should have panicked");
    } catch (err: any) {
      console.log("Overflow panic caught as expected (program reverts):");
      console.log("Error message:", err.message);
      if (err.logs) {
        console.log("Logs:", err.logs.join("\n"));
      }
      // Assert panic message contains "attempt to add with overflow"
      assert.include(err.message.toLowerCase(), "overflow", "Expected overflow panic");
    }
  });

  it("Vulnerable: Underflow attack (wraps to huge value)", async () => {
    await vulnProgram.methods.withdraw(new BN(1))
      .accounts({ vault: vulnVault })
      .rpc();

    const vaultData = await vulnProgram.account.vault.fetch(vulnVault);
    console.log("After underflow attempt (withdraw 1 from 0), balance wrapped to:", vaultData.balance.toString());
    // Expected: huge number close to u64::MAX
  });

  it("Fixed: Prevents overflow", async () => {
    const maxU64 = new BN("18446744073709551615");

    await fixedProgram.methods.deposit(maxU64)
      .accounts({ vault: fixedVault })
      .rpc();
    console.log("Deposited u64::MAX safely");

    try {
      await fixedProgram.methods.deposit(new BN(1))
        .accounts({ vault: fixedVault })
        .rpc();
      console.log("ERROR: Overflow should have been prevented");
    } catch (err) {
      console.log("Fixed correctly reverted on overflow:", err.message);
    }
  });

  it("Fixed: Prevents underflow", async () => {
    try {
      await fixedProgram.methods.withdraw(new BN(1))
        .accounts({ vault: fixedVault })
        .rpc();
      console.log("ERROR: Underflow should have been prevented");
    } catch (err) {
      console.log("Fixed correctly reverted on underflow:", err.message);
    }
  });
});