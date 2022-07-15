import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { MusechainEscrow } from "../target/types/musechain_escrow";

describe("musechain-escrow", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.MusechainEscrow as Program<MusechainEscrow>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
