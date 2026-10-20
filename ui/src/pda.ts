import { PublicKey } from "@solana/web3.js";
import { SEEDS } from "./config.ts";

export function findGlobalConfigPda(programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync([SEEDS.globalConfig], programId)[0];
}

export function findVaultPda(programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync([SEEDS.spotVault], programId)[0];
}

export function findNullifierPda(
  programId: PublicKey,
  nullifier: Uint8Array,
): PublicKey {
  return PublicKey.findProgramAddressSync(
    [SEEDS.nullifier, nullifier],
    programId,
  )[0];
}

export function findCleanFundsRootPda(
  programId: PublicKey,
  root: Uint8Array,
): PublicKey {
  return PublicKey.findProgramAddressSync(
    [SEEDS.cleanFundsRoot, root],
    programId,
  )[0];
}
