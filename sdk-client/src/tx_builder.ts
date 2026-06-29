import { PublicKey, TransactionInstruction } from "@solana/web3.js";

export class TxBuilder {
  static buildMintJitInstruction(
    vaultPda: PublicKey,
    owner: PublicKey,
    nativeProgramId: PublicKey,
    amount: bigint,
    nonce: bigint
  ): TransactionInstruction {
    const data = Buffer.alloc(17);
    data.writeUInt8(0, 0); // MintJit
    data.writeBigUInt64LE(amount, 1);
    data.writeBigUInt64LE(nonce, 9);

    return new TransactionInstruction({
      keys: [
        { pubkey: vaultPda, isSigner: false, isWritable: true },
        { pubkey: owner, isSigner: true, isWritable: false },
      ],
      programId: nativeProgramId,
      data,
    });
  }

  static buildLiquidateInstruction(
    vaultPda: PublicKey,
    liquidator: PublicKey,
    nativeProgramId: PublicKey,
    maxDebtToCover: bigint
  ): TransactionInstruction {
    const data = Buffer.alloc(9);
    data.writeUInt8(1, 0); // Liquidate
    data.writeBigUInt64LE(maxDebtToCover, 1);

    return new TransactionInstruction({
      keys: [
        { pubkey: vaultPda, isSigner: false, isWritable: true },
        { pubkey: liquidator, isSigner: true, isWritable: false },
      ],
      programId: nativeProgramId,
      data,
    });
  }

  static buildSettleInstruction(
    vaultPda: PublicKey,
    authority: PublicKey,
    nativeProgramId: PublicKey,
    amount: bigint
  ): TransactionInstruction {
    const data = Buffer.alloc(9);
    data.writeUInt8(2, 0); // Settle
    data.writeBigUInt64LE(amount, 1);

    return new TransactionInstruction({
      keys: [
        { pubkey: vaultPda, isSigner: false, isWritable: true },
        { pubkey: authority, isSigner: true, isWritable: false },
      ],
      programId: nativeProgramId,
      data,
    });
  }
}
