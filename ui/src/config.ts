/** Default localnet program id (`declare_id!` / Anchor.toml). */
export const DEFAULT_PROGRAM_ID =
  "H6X5T8TP6T8hyTySiDVAug8bfULbKGMhCTV5PA9VMHPv";

export const DEFAULT_RPC_LIST = "http://127.0.0.1:8899";

export const SETTLE_CU_LIMIT = 1_400_000;

export const SETTLE_DISCRIMINATOR = Uint8Array.from([
  233, 96, 13, 60, 148, 116, 38, 171,
]);

export const SEEDS = {
  globalConfig: new TextEncoder().encode("global-config"),
  spotVault: new TextEncoder().encode("spot_vault"),
  nullifier: new TextEncoder().encode("nullifier"),
  cleanFundsRoot: new TextEncoder().encode("clean_funds_root"),
} as const;
