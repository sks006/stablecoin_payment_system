import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
import { expect } from "chai";
import { getProvider } from "./setup";

describe("Native Data Plane - JIT Execution", () => {
  const provider = getProvider();
  const anchorProgram = anchor.workspace.AnchorStablecoin as Program;
  const NATIVE_PROGRAM_ID = new PublicKey("NPlane1111111111111111111111111111111111112");

  it("Performs JIT minting using the Native program", async () => {
    const owner = Keypair.generate();
    const [vaultPda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), owner.publicKey.toBuffer()],
      anchorProgram.programId
    );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(owner.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL),
      "confirmed"
    );

    await anchorProgram.methods
      .initializeVault(bump)
      .accounts({
        vault: vaultPda,
        owner: owner.publicKey,
        payer: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([owner])
      .rpc();

    const amount = 1000n;
    const nonce = 42n;
    const instructionData = Buffer.alloc(17);
    instructionData.writeUInt8(0, 0); // MintJit
    instructionData.writeBigUInt64LE(amount, 1);
    instructionData.writeBigUInt64LE(nonce, 9);

    const tx = new Transaction().add(
      new TransactionInstruction({
        keys: [
          { pubkey: vaultPda, isSigner: false, isWritable: true },
          { pubkey: owner.publicKey, isSigner: true, isWritable: false },
        ],
        programId: NATIVE_PROGRAM_ID,
        data: instructionData,
      })
    );

    await provider.sendAndConfirm(tx, [owner]);

    const vaultAccount = await anchorProgram.account.userVaultState.fetch(vaultPda);
    expect(Number(vaultAccount.collateralBalance)).to.equal(1000);
    expect(Number(vaultAccount.debtBalance)).to.equal(1000);
  });
});
