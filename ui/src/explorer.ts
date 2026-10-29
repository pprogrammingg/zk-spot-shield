export function isLocalRpc(rpc: string): boolean {
  return rpc.includes("127.0.0.1") || rpc.includes("localhost");
}

/** Public explorer URL, or null on localnet (HTTPS explorer cannot call http://127.0.0.1). */
export function accountUrl(pk: string, rpc: string): string | null {
  if (isLocalRpc(rpc)) return null;
  if (rpc.includes("devnet")) {
    return `https://explorer.solana.com/address/${pk}?cluster=devnet`;
  }
  return `https://explorer.solana.com/address/${pk}?cluster=devnet`;
}

export function txUrl(sig: string, rpc: string): string | null {
  if (isLocalRpc(rpc)) return null;
  if (rpc.includes("devnet")) {
    return `https://explorer.solana.com/tx/${sig}?cluster=devnet`;
  }
  return `https://explorer.solana.com/tx/${sig}?cluster=devnet`;
}

export function localConfirmCmd(sig: string, rpc: string): string {
  return `solana confirm -v ${sig} --url ${rpc}`;
}
