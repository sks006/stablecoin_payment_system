import * as anchor from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";

export const getProvider = (): anchor.AnchorProvider => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  return provider;
};
