import * as anchor from "@coral-xyz/anchor";
import { assert } from "chai";
import { AttackerProgram } from "../target/types/attacker_program";
import { UnsafeCpiFixed } from "../target/types/unsafe_cpi_fixed";
import { UnsafeCpiVulnerable } from "../target/types/unsafe_cpi_vulnerable";

describe("Unchecked CPI vulnerability", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const attacker = anchor.workspace.AttackerProgram;
  const vulnerable = anchor.workspace.UnsafeCpiVulnerable;
  const fixed = anchor.workspace.UnsafeCpiFixed;

  const user = provider.wallet;

  it("VULNERABLE: attacker hijacks wallet", async () => {
    const before = await provider.connection.getAccountInfo(user.publicKey);
    const originalOwner = before!.owner.toBase58();

    await vulnerable.methods
      .callExternal()
      .accounts({
        user: user.publicKey,
        externalProgram: attacker.programId,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const after = await provider.connection.getAccountInfo(user.publicKey);
    const newOwner = after!.owner.toBase58();

    console.log("Before owner:", originalOwner);
    console.log("After owner:", newOwner);

    assert.equal(newOwner, attacker.programId.toBase58());
  });


  it("FIXED: rejects malicious CPI", async () => {
    try {
      await fixed.methods
        .callExternal()
        .accounts({
          user: user.publicKey,
          externalProgram: attacker.programId,
        })
        .rpc();

      assert.fail("Transaction should have failed");
    } catch (_) {
      const acct = await provider.connection.getAccountInfo(user.publicKey);
      assert.equal(
        acct!.owner.toBase58(),
        anchor.web3.SystemProgram.programId.toBase58()
      );
    }
  });
});
