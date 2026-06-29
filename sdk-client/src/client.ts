import { Connection, PublicKey, Transaction, Keypair, sendAndConfirmTransaction } from "@solana/web3.js";
import { TxBuilder } from "./tx_builder";
import { UserVaultState } from "./types";

export class StablecoinClient {
  private connection: Connection;
  private nativeProgramId: PublicKey;
  private anchorProgramId: PublicKey;

  constructor(connection: Connection, nativeProgramId: PublicKey, anchorProgramId: PublicKey) {
    this.connection = connection;
    this.nativeProgramId = nativeProgramId;
    this.anchorProgramId = anchorProgramId;
  }

  async getVaultState(vaultPda: PublicKey): Promise<UserVaultState> {
    const info = await this.connection.getAccountInfo(vaultPda);
    if (!info) {
      throw new Error("Vault account not found");
    }

    const data = info.data.slice(8);

    const version = data.readUInt8(0);
    const bump = data.readUInt8(1);
    const owner = new PublicKey(data.slice(8, 40));
    const primaryDelegate = new PublicKey(data.slice(40, 72));
    const governanceAuthority = new PublicKey(data.slice(72, 104));
    const collateralBalance = data.readBigUInt64LE(104);
    const debtBalance = data.readBigUInt64LE(112);
    const stateFlags = data.readBigUInt64LE(120);

    return {
      version,
      bump,
      owner,
      primaryDelegate,
      governanceAuthority,
      collateralBalance,
      debtBalance,
      stateFlags,
    };
  }

  async mintJit(owner: Keypair, amount: bigint, nonce: bigint): Promise<string> {
    const [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), owner.publicKey.toBuffer()],
      this.anchorProgramId
    );

    const inst = TxBuilder.buildMintJitInstruction(
      vaultPda,
      owner.publicKey,
      this.nativeProgramId,
      amount,
      nonce
    );

    const tx = new Transaction().add(inst);
    return await sendAndConfirmTransaction(this.connection, tx, [owner]);
  }
}
