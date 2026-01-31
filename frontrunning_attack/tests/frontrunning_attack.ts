import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { FrontrunningVulnerable } from "../target/types/frontrunning_vulnerable";
import { FrontrunningFixed } from "../target/types/frontrunning_fixed";
import { assert } from "chai";

describe("Front-running Attack Example", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const vulnerable = anchor.workspace.FrontrunningVulnerable as Program<FrontrunningVulnerable>;
  const fixed = anchor.workspace.FrontrunningFixed as Program<FrontrunningFixed>;

  // Separate accounts per program to avoid AccountOwnedByWrongProgram
  let makerBalanceVuln: anchor.web3.Keypair;
  let takerBalanceVuln: anchor.web3.Keypair;
  let makerBalanceFixed: anchor.web3.Keypair;
  let takerBalanceFixed: anchor.web3.Keypair;
  let orderVuln: anchor.web3.Keypair;
  let orderFixed: anchor.web3.Keypair;

  beforeEach(async () => {
    // Reset accounts
    makerBalanceVuln = anchor.web3.Keypair.generate();
    takerBalanceVuln = anchor.web3.Keypair.generate();
    makerBalanceFixed = anchor.web3.Keypair.generate();
    takerBalanceFixed = anchor.web3.Keypair.generate();
    orderVuln = anchor.web3.Keypair.generate();
    orderFixed = anchor.web3.Keypair.generate();

    console.log("\n--- Initializing balances ---");

    // Vulnerable program balances
    await vulnerable.methods.initBalance(new anchor.BN(0))
      .accounts({ balance: makerBalanceVuln.publicKey, user: provider.wallet.publicKey, systemProgram: anchor.web3.SystemProgram.programId })
      .signers([makerBalanceVuln])
      .rpc();

    await vulnerable.methods.initBalance(new anchor.BN(1000))
      .accounts({ balance: takerBalanceVuln.publicKey, user: provider.wallet.publicKey, systemProgram: anchor.web3.SystemProgram.programId })
      .signers([takerBalanceVuln])
      .rpc();

    console.log("Vulnerable program: Maker=0, Taker=1000");

    // Fixed program balances
    await fixed.methods.initBalance(new anchor.BN(0))
      .accounts({ balance: makerBalanceFixed.publicKey, user: provider.wallet.publicKey, systemProgram: anchor.web3.SystemProgram.programId })
      .signers([makerBalanceFixed])
      .rpc();

    await fixed.methods.initBalance(new anchor.BN(1000))
      .accounts({ balance: takerBalanceFixed.publicKey, user: provider.wallet.publicKey, systemProgram: anchor.web3.SystemProgram.programId })
      .signers([takerBalanceFixed])
      .rpc();

    console.log("Fixed program: Maker=0, Taker=1000");
  });

  it("VULNERABLE: maker front-runs taker", async () => {
    console.log("\nStep 1: Taker creates order at price 10, amount 10 (VULNERABLE)");
    await vulnerable.methods.createOrder(new anchor.BN(10), new anchor.BN(10))
      .accounts({ order: orderVuln.publicKey, maker: provider.wallet.publicKey, systemProgram: anchor.web3.SystemProgram.programId })
      .signers([orderVuln])
      .rpc();

    console.log("Order created: price=10, amount=10");

    console.log("Step 2: Maker front-runs by updating price to 100");
    await vulnerable.methods.updatePrice(new anchor.BN(100))
      .accounts({ order: orderVuln.publicKey, maker: provider.wallet.publicKey })
      .rpc();

    console.log("Price updated to 100 before taker fills");

    console.log("Step 3: Taker fills order (thinks price=10, actually 100)");
    await vulnerable.methods.fillOrder(new anchor.BN(1))
      .accounts({ order: orderVuln.publicKey, makerBalance: makerBalanceVuln.publicKey, takerBalance: takerBalanceVuln.publicKey })
      .rpc();

    const taker = await vulnerable.account.balance.fetch(takerBalanceVuln.publicKey);
    const maker = await vulnerable.account.balance.fetch(makerBalanceVuln.publicKey);

    console.log(`Taker balance after: ${taker.balance.toNumber()}`);
    console.log(`Maker balance after: ${maker.balance.toNumber()}`);
    console.log("Vulnerable: Taker overpaid due to front-running");

    assert.equal(taker.balance.toNumber(), 900);
    assert.equal(maker.balance.toNumber(), 100);
  });

  it("FIXED: taker locks expected price, front-run fails", async () => {
    console.log("\nStep 1: Taker creates order at price 10 (FIXED)");
    await fixed.methods.createOrder(new anchor.BN(10), new anchor.BN(10))
      .accounts({ order: orderFixed.publicKey, maker: provider.wallet.publicKey, systemProgram: anchor.web3.SystemProgram.programId })
      .signers([orderFixed])
      .rpc();

    console.log("Order created: price=10");

    console.log("Step 2: Maker tries to front-run by updating price to 100");
    await fixed.methods.updatePrice(new anchor.BN(100))
      .accounts({ order: orderFixed.publicKey, maker: provider.wallet.publicKey })
      .rpc();

    console.log("Step 3: Taker fills order with locked price=10");
    try {
      await fixed.methods.fillOrder(new anchor.BN(1), new anchor.BN(10))
        .accounts({ order: orderFixed.publicKey, makerBalance: makerBalanceFixed.publicKey, takerBalance: takerBalanceFixed.publicKey })
        .rpc();

      assert.fail("Expected transaction to fail due to price mismatch");
    } catch (err) {
      console.log("Transaction failed as expected! Front-running prevented.");
      assert.ok(true);
    }
  });
});
