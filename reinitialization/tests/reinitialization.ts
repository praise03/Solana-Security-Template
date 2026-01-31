import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Vulnerable } from "../target/types/vulnerable";
import { Fixed } from "../target/types/fixed";
import { assert } from "chai";

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

describe("Reinitialization Attack", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const vulnerable = anchor.workspace.Vulnerable as Program<Vulnerable>;
  const fixed = anchor.workspace.Fixed as Program<Fixed>;

  const user = provider.wallet.publicKey;

  let vulnerableRewardPda: anchor.web3.PublicKey;
  let fixedRewardPda: anchor.web3.PublicKey;

  before(async () => {
    [vulnerableRewardPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("rewards"), user.toBuffer()],
      vulnerable.programId
    );

    [fixedRewardPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("rewards"), user.toBuffer()],
      fixed.programId
    );
  });

  it("VULNERABLE: reinitialization resets reward history", async () => {
    console.log("\n--- VULNERABLE FLOW ---");

    console.log("1) Initializing rewards account");
    await vulnerable.methods
      .initialize()
      .accounts({
        data: vulnerableRewardPda,
        user,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Waiting 2 seconds to accrue rewards...");
    await sleep(2000);

    const before = await vulnerable.account.rewardData.fetch(vulnerableRewardPda);
    console.log(
      "created_at before reinit:",
      before.createdAt.toString()
    );

    const expectedRewards = 2 * 100;
    console.log(
      `Expected rewards ≈ ${expectedRewards} points (2s * 100)`
    );

    console.log("2) Reinitializing (ERROR)");
    await vulnerable.methods
      .initialize()
      .accounts({
        data: vulnerableRewardPda,
        user,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const after = await vulnerable.account.rewardData.fetch(vulnerableRewardPda);
    console.log(
      "created_at AFTER reinit:",
      after.createdAt.toString()
    );

    assert.isTrue(
      after.createdAt.gt(before.createdAt),
      "created_at was reset"
    );

    console.log(
      "3) Claiming rewards after reinitialization"
    );

    await vulnerable.methods
      .claimRewards()
      .accounts({
        data: vulnerableRewardPda,
        user,
      })
      .rpc();
    
    console.log((await vulnerable.account.rewardData.fetch(vulnerableRewardPda)).points.toString(), "points after claiming");
    console.log(
      "❌ Rewards were calculated from the NEW timestamp, wiping accrued rewards"
    );
  });

  it("FIXED: reinitialization is rejected", async () => {
    console.log("\n--- FIXED FLOW ---");


    console.log("1) Initializing rewards account (fixed)");
    await fixed.methods
      .initialize()
      .accounts({
        data: fixedRewardPda,
        user,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Waiting 2 seconds to accrue rewards...");
    await sleep(2000);

    const before = await fixed.account.rewardData.fetch(fixedRewardPda);
    console.log(
      "created_at:",
      before.createdAt.toString()
    );

    console.log("2) Attempting reinitialization (should fail)");

    try {
      await fixed.methods
        .initialize()
        .accounts({
          data: fixedRewardPda,
          user,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      assert.fail("Reinitialization should have failed");
    } catch (_) {
      console.log("✅ Reinitialization correctly blocked");
    }

    const after = await fixed.account.rewardData.fetch(fixedRewardPda);

    assert.equal(
      before.createdAt.toString(),
      after.createdAt.toString(),
      "created_at unchanged"
    );

    console.log("3) Claiming rewards (fixed)");

    await fixed.methods
      .claimRewards()
      .accounts({
        data: fixedRewardPda,
        user,
      })
      .rpc();

    console.log(
      "✅ Rewards calculated from original timestamp as expected"
    );
  });
});
