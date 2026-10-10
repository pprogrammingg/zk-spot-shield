/**
 * Pack SPL + Anchor account bytes for `solana-test-validator --account` preloads.
 * Fixture mint is `[3u8;32]` (happy journal); mint_b is `[4u8;32]`.
 */

import { createHash } from "node:crypto";
import { writeFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import { Keypair, PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

export const MINT_A = new PublicKey(Buffer.alloc(32, 3));
export const MINT_B = new PublicKey(Buffer.alloc(32, 4));

export function accountDiscriminator(name: string): Buffer {
  return createHash("sha256")
    .update(`account:${name}`)
    .digest()
    .subarray(0, 8);
}

/** SPL Mint (82 bytes), initialized, decimals=0. */
export function packMint(supply: bigint): Buffer {
  const data = Buffer.alloc(82);
  // mint_authority: COption::None → 0
  data.writeBigUInt64LE(supply, 36);
  data[44] = 0; // decimals
  data[45] = 1; // is_initialized
  // freeze_authority: COption::None
  return data;
}

/** SPL Token Account (165 bytes). */
export function packTokenAccount(
  mint: PublicKey,
  owner: PublicKey,
  amount: bigint,
): Buffer {
  const data = Buffer.alloc(165);
  mint.toBuffer().copy(data, 0);
  owner.toBuffer().copy(data, 32);
  data.writeBigUInt64LE(amount, 64);
  // delegate COption::None at 72
  data[108] = 1; // AccountState::Initialized
  // rest zero
  return data;
}

/** Zero-copy VaultState (8 disc + 120). */
export function packVaultState(args: {
  authority: PublicKey;
  mintA: PublicKey;
  mintB: PublicKey;
  reserveA: bigint;
  reserveB: bigint;
  bump: number;
}): Buffer {
  const body = Buffer.alloc(120);
  args.authority.toBuffer().copy(body, 0);
  args.mintA.toBuffer().copy(body, 32);
  args.mintB.toBuffer().copy(body, 64);
  body.writeBigUInt64LE(args.reserveA, 96);
  body.writeBigUInt64LE(args.reserveB, 104);
  body[112] = args.bump;
  return Buffer.concat([accountDiscriminator("VaultState"), body]);
}

/** Borsh GlobalConfig (8 disc + 32 + 32 + 1). */
export function packGlobalConfig(args: {
  authority: PublicKey;
  vkeyHash: Buffer;
  pauseFlag: boolean;
}): Buffer {
  if (args.vkeyHash.length !== 32) throw new Error("vkeyHash must be 32 bytes");
  const body = Buffer.alloc(65);
  args.authority.toBuffer().copy(body, 0);
  args.vkeyHash.copy(body, 32);
  body[64] = args.pauseFlag ? 1 : 0;
  return Buffer.concat([accountDiscriminator("GlobalConfig"), body]);
}

export type ValidatorAccountFile = {
  pubkey: string;
  account: {
    lamports: number;
    data: [string, "base64"];
    owner: string;
    executable: boolean;
    rentEpoch: number;
    space?: number;
  };
};

export function toAccountFile(
  pubkey: PublicKey,
  owner: PublicKey,
  data: Buffer,
  lamports = 1_000_000_000,
): ValidatorAccountFile {
  return {
    pubkey: pubkey.toBase58(),
    account: {
      lamports,
      data: [data.toString("base64"), "base64"],
      owner: owner.toBase58(),
      executable: false,
      rentEpoch: 0,
      space: data.length,
    },
  };
}

export type E2EAccountBundle = {
  mintA: PublicKey;
  mintB: PublicKey;
  vaultTokenA: PublicKey;
  vaultTokenB: PublicKey;
  userTokenA: PublicKey;
  userTokenB: PublicKey;
  globalConfig: PublicKey;
  vault: PublicKey;
  vaultBump: number;
  files: { path: string; pubkey: PublicKey }[];
};

/**
 * Write account JSON files under `outDir` for validator `--account` flags.
 * Token account pubkeys are fresh random keys (stored in returned bundle).
 */
export function writeE2EAccountFiles(args: {
  outDir: string;
  programId: PublicKey;
  authority: PublicKey;
  vkeyHash: Buffer;
  reserveA: bigint;
  reserveB: bigint;
  userA: bigint;
  userB: bigint;
}): E2EAccountBundle {
  mkdirSync(args.outDir, { recursive: true });

  const [globalConfig] = PublicKey.findProgramAddressSync(
    [Buffer.from("global-config")],
    args.programId,
  );
  const [vault, vaultBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("spot_vault")],
    args.programId,
  );

  const vaultTokenA = Keypair.generate().publicKey;
  const vaultTokenB = Keypair.generate().publicKey;
  const userTokenA = Keypair.generate().publicKey;
  const userTokenB = Keypair.generate().publicKey;

  const files: { path: string; pubkey: PublicKey }[] = [];
  const write = (pk: PublicKey, owner: PublicKey, data: Buffer, name: string) => {
    const path = join(args.outDir, `${name}.json`);
    writeFileSync(
      path,
      JSON.stringify(toAccountFile(pk, owner, data), null, 2),
    );
    files.push({ path, pubkey: pk });
  };

  write(
    globalConfig,
    args.programId,
    packGlobalConfig({
      authority: args.authority,
      vkeyHash: args.vkeyHash,
      pauseFlag: false,
    }),
    "global_config",
  );
  write(
    vault,
    args.programId,
    packVaultState({
      authority: args.authority,
      mintA: MINT_A,
      mintB: MINT_B,
      reserveA: args.reserveA,
      reserveB: args.reserveB,
      bump: vaultBump,
    }),
    "vault",
  );
  write(
    MINT_A,
    TOKEN_PROGRAM_ID,
    packMint(args.reserveA + args.userA),
    "mint_a",
  );
  write(
    MINT_B,
    TOKEN_PROGRAM_ID,
    packMint(args.reserveB + args.userB),
    "mint_b",
  );
  write(
    vaultTokenA,
    TOKEN_PROGRAM_ID,
    packTokenAccount(MINT_A, vault, args.reserveA),
    "vault_token_a",
  );
  write(
    vaultTokenB,
    TOKEN_PROGRAM_ID,
    packTokenAccount(MINT_B, vault, args.reserveB),
    "vault_token_b",
  );
  write(
    userTokenA,
    TOKEN_PROGRAM_ID,
    packTokenAccount(MINT_A, args.authority, args.userA),
    "user_token_a",
  );
  write(
    userTokenB,
    TOKEN_PROGRAM_ID,
    packTokenAccount(MINT_B, args.authority, args.userB),
    "user_token_b",
  );

  // Persist token pubkeys for the test runner.
  writeFileSync(
    join(args.outDir, "bundle.json"),
    JSON.stringify(
      {
        mintA: MINT_A.toBase58(),
        mintB: MINT_B.toBase58(),
        vaultTokenA: vaultTokenA.toBase58(),
        vaultTokenB: vaultTokenB.toBase58(),
        userTokenA: userTokenA.toBase58(),
        userTokenB: userTokenB.toBase58(),
        globalConfig: globalConfig.toBase58(),
        vault: vault.toBase58(),
        vaultBump,
      },
      null,
      2,
    ),
  );

  return {
    mintA: MINT_A,
    mintB: MINT_B,
    vaultTokenA,
    vaultTokenB,
    userTokenA,
    userTokenB,
    globalConfig,
    vault,
    vaultBump,
    files,
  };
}
