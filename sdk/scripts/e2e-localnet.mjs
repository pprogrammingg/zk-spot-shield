#!/usr/bin/env node
/**
 * Day 38: one command — build program, start local validator (program + preloaded
 * accounts), run happy + negative E2E, tear down.
 *
 * Usage (from repo root or sdk/):
 *   node sdk/scripts/e2e-localnet.mjs
 *   npm run test:e2e   # from sdk/
 */

import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
  rmSync,
} from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const SDK_ROOT = join(SCRIPT_DIR, "..");
const REPO_ROOT = join(SDK_ROOT, "..");
const LEDGER = join(REPO_ROOT, "test-ledger-e2e");
const ACCOUNTS_DIR = join(SDK_ROOT, ".e2e-accounts");
const PROGRAM_ID = "H6X5T8TP6T8hyTySiDVAug8bfULbKGMhCTV5PA9VMHPv";
const SO = join(REPO_ROOT, "target/deploy/zk_spot_shield.so");

const VKEY_HASH = Buffer.from(
  "00b3a15ce4c0ea94e3b0267473c6b7543a80c72d209b2c947f71886c6a5735d7",
  "hex",
);

function disc(name) {
  return createHash("sha256").update(`account:${name}`).digest().subarray(0, 8);
}

function packMint(supply) {
  const data = Buffer.alloc(82);
  data.writeBigUInt64LE(supply, 36);
  data[44] = 0;
  data[45] = 1;
  return data;
}

function packToken(mintBuf, ownerBuf, amount) {
  const data = Buffer.alloc(165);
  mintBuf.copy(data, 0);
  ownerBuf.copy(data, 32);
  data.writeBigUInt64LE(amount, 64);
  data[108] = 1;
  return data;
}

function packVault(authorityBuf, mintA, mintB, ra, rb, bump) {
  const body = Buffer.alloc(120);
  authorityBuf.copy(body, 0);
  mintA.copy(body, 32);
  mintB.copy(body, 64);
  body.writeBigUInt64LE(ra, 96);
  body.writeBigUInt64LE(rb, 104);
  body[112] = bump;
  return Buffer.concat([disc("VaultState"), body]);
}

function packGlobal(authorityBuf, vkey, pause) {
  const body = Buffer.alloc(65);
  authorityBuf.copy(body, 0);
  vkey.copy(body, 32);
  body[64] = pause ? 1 : 0;
  return Buffer.concat([disc("GlobalConfig"), body]);
}

function accountJson(pubkey, owner, data) {
  return {
    pubkey,
    account: {
      lamports: 1_000_000_000,
      data: [data.toString("base64"), "base64"],
      owner,
      executable: false,
      rentEpoch: 0,
      space: data.length,
    },
  };
}

async function main() {
  const pathEnv = `${process.env.PATH ?? ""}:${homedir()}/.local/share/solana/install/active_release/bin`;
  process.env.PATH = pathEnv;
  // Avoid sandbox / alternate target dirs so deploy .so lands in repo `target/deploy`.
  delete process.env.CARGO_TARGET_DIR;

  console.log("==> anchor build --ignore-keys");
  let r = spawnSync(
    "anchor",
    ["build", "--ignore-keys"],
    { cwd: REPO_ROOT, stdio: "inherit", env: process.env },
  );
  if (r.status !== 0) process.exit(r.status ?? 1);
  if (!existsSync(SO)) {
    console.error("missing", SO);
    process.exit(1);
  }

  // Use web3 from sdk/node_modules
  const { PublicKey, Keypair } = await import(
    join(SDK_ROOT, "node_modules/@solana/web3.js/lib/index.cjs.js")
  ).catch(async () => import("@solana/web3.js"));

  const walletPath =
    process.env.ANCHOR_WALLET ?? join(homedir(), ".config/solana/id.json");
  const authority = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(readFileSync(walletPath, "utf8"))),
  );
  const programId = new PublicKey(PROGRAM_ID);
  const [globalConfig] = PublicKey.findProgramAddressSync(
    [Buffer.from("global-config")],
    programId,
  );
  const [vault, vaultBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("spot_vault")],
    programId,
  );
  const mintA = new PublicKey(Buffer.alloc(32, 3));
  const mintB = new PublicKey(Buffer.alloc(32, 4));
  const vaultTokenA = Keypair.generate().publicKey;
  const vaultTokenB = Keypair.generate().publicKey;
  const userTokenA = Keypair.generate().publicKey;
  const userTokenB = Keypair.generate().publicKey;

  const reserveA = 1000n;
  const reserveB = 1000n;
  const userA = 1000n;
  const userB = 0n;

  rmSync(ACCOUNTS_DIR, { recursive: true, force: true });
  mkdirSync(ACCOUNTS_DIR, { recursive: true });

  const TOKEN = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
  const write = (pk, owner, data, name) => {
    const p = join(ACCOUNTS_DIR, `${name}.json`);
    writeFileSync(
      p,
      JSON.stringify(accountJson(pk.toBase58(), owner, data), null, 2),
    );
    return p;
  };

  const authBuf = authority.publicKey.toBuffer();
  const paths = [
    write(
      globalConfig,
      PROGRAM_ID,
      packGlobal(authBuf, VKEY_HASH, false),
      "global_config",
    ),
    write(
      vault,
      PROGRAM_ID,
      packVault(
        authBuf,
        mintA.toBuffer(),
        mintB.toBuffer(),
        reserveA,
        reserveB,
        vaultBump,
      ),
      "vault",
    ),
    write(mintA, TOKEN, packMint(reserveA + userA), "mint_a"),
    write(mintB, TOKEN, packMint(reserveB + userB), "mint_b"),
    write(
      vaultTokenA,
      TOKEN,
      packToken(mintA.toBuffer(), vault.toBuffer(), reserveA),
      "vault_token_a",
    ),
    write(
      vaultTokenB,
      TOKEN,
      packToken(mintB.toBuffer(), vault.toBuffer(), reserveB),
      "vault_token_b",
    ),
    write(
      userTokenA,
      TOKEN,
      packToken(mintA.toBuffer(), authBuf, userA),
      "user_token_a",
    ),
    write(
      userTokenB,
      TOKEN,
      packToken(mintB.toBuffer(), authBuf, userB),
      "user_token_b",
    ),
  ];

  writeFileSync(
    join(ACCOUNTS_DIR, "bundle.json"),
    JSON.stringify(
      {
        programId: PROGRAM_ID,
        authority: authority.publicKey.toBase58(),
        globalConfig: globalConfig.toBase58(),
        vault: vault.toBase58(),
        vaultBump,
        mintA: mintA.toBase58(),
        mintB: mintB.toBase58(),
        vaultTokenA: vaultTokenA.toBase58(),
        vaultTokenB: vaultTokenB.toBase58(),
        userTokenA: userTokenA.toBase58(),
        userTokenB: userTokenB.toBase58(),
      },
      null,
      2,
    ),
  );

  rmSync(LEDGER, { recursive: true, force: true });

  const accountArgs = [];
  for (const p of paths) {
    const j = JSON.parse(readFileSync(p, "utf8"));
    accountArgs.push("--account", j.pubkey, p);
  }

  const validatorArgs = [
    "--reset",
    "--ledger",
    LEDGER,
    "--bind-address",
    "127.0.0.1",
    "--rpc-port",
    "8899",
    "--bpf-program",
    PROGRAM_ID,
    SO,
    ...accountArgs,
  ];

  console.log("==> solana-test-validator (background)");
  const validator = spawn("solana-test-validator", validatorArgs, {
    stdio: ["ignore", "pipe", "pipe"],
    env: process.env,
  });
  let ready = false;
  const onData = (buf) => {
    const s = buf.toString();
    process.stdout.write(s);
    if (s.includes("JSON RPC URL") || s.includes("Processing NodeReady")) {
      ready = true;
    }
  };
  validator.stdout.on("data", onData);
  validator.stderr.on("data", onData);

  const deadline = Date.now() + 60_000;
  while (!ready && Date.now() < deadline) {
    await new Promise((r) => setTimeout(r, 500));
    // also try RPC
    const ping = spawnSync(
      "solana",
      ["cluster-version", "--url", "http://127.0.0.1:8899"],
      { encoding: "utf8" },
    );
    if (ping.status === 0) {
      ready = true;
      break;
    }
  }
  if (!ready) {
    validator.kill("SIGTERM");
    console.error("validator failed to become ready");
    process.exit(1);
  }

  // Fund authority
  spawnSync(
    "solana",
    [
      "airdrop",
      "100",
      authority.publicKey.toBase58(),
      "--url",
      "http://127.0.0.1:8899",
    ],
    { stdio: "inherit", env: process.env },
  );

  console.log("==> npm test (unit) + vitest e2e");
  r = spawnSync("npm", ["test"], { cwd: SDK_ROOT, stdio: "inherit", env: process.env });
  if (r.status !== 0) {
    validator.kill("SIGTERM");
    process.exit(r.status ?? 1);
  }

  r = spawnSync(
    "npx",
    ["vitest", "run", "--config", "vitest.e2e.config.ts"],
    {
      cwd: SDK_ROOT,
      stdio: "inherit",
      env: {
        ...process.env,
        RUN_E2E: "1",
        E2E_ACCOUNTS_DIR: ACCOUNTS_DIR,
        ANCHOR_PROVIDER_URL: "http://127.0.0.1:8899",
      },
    },
  );

  console.log("==> stopping validator");
  validator.kill("SIGTERM");
  try {
    rmSync(LEDGER, { recursive: true, force: true });
  } catch {
    /* ignore */
  }
  process.exit(r.status ?? 1);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
