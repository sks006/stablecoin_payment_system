import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { getProvider } from "./setup";
import { Keypair, PublicKey } from "@solana/web3.js";

describe("Anchor Control Plane - Admin Flow", () => {
  const provider = getProvider();
  const program = anchor.workspace.AnchorStablecoin as Program;

  it("Initializes the vault", async () => {
    const owner = Keypair.generate();
    const [vaultPda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), owner.publicKey.toBuffer()],
      program.programId
    );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(owner.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL),
      "confirmed"
    );

    await program.methods
      .initializeVault(bump)
      .accounts({
        vault: vaultPda,
        owner: owner.publicKey,
        payer: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([owner])
      .rpc();

    const vaultAccount = await program.account.userVaultState.fetch(vaultPda);
    expect(vaultAccount.version).to.equal(1);
    expect(vaultAccount.owner.toBase58()).to.equal(owner.publicKey.toBase58());
  });
});
