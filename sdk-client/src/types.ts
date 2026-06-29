import { PublicKey } from "@solana/web3.js";

export interface UserVaultState {
  version: number;
  bump: number;
  owner: PublicKey;
  primaryDelegate: PublicKey;
  governanceAuthority: PublicKey;
  collateralBalance: bigint;
  debtBalance: bigint;
  stateFlags: bigint;
}
