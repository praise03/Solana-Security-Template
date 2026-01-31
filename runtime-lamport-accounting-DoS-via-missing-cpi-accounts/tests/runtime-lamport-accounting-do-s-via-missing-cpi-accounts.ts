import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RuntimeLamportAccountingDoSViaMissingCpiAccounts } from "../target/types/runtime_lamport_accounting_do_s_via_missing_cpi_accounts";

describe("runtime-lamport-accounting-do-s-via-missing-cpi-accounts", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.runtimeLamportAccountingDoSViaMissingCpiAccounts as Program<RuntimeLamportAccountingDoSViaMissingCpiAccounts>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
