import { AnchorProvider, Program, web3 } from '@coral-xyz/anchor';

export async function setupLocalEnvironment() {
  const connection = new web3.Connection('http://127.0.0.1:8899', 'processed');
  return { connection };
}
