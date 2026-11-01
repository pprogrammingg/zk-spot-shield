/**
 * Prove bridge — return Groth16 proof + journal bytes for settle.
 *
 * Default: load committed happy-path fixtures (CI-safe, no `$PROVE`).
 * Optional: spawn the Rust host with network Groth16 when explicitly requested.
 */

import { spawn } from "node:child_process";
import { readFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
/** Repo root: `sdk/src` → `../..` */
export const REPO_ROOT = join(HERE, "../..");

export const HAPPY_FIXTURE_DIR = join(
  REPO_ROOT,
  "zk-circuit/fixtures/happy",
);

/** @deprecated Happy pack is the 144-byte wrap. */
export const CHANGE_FIXTURE_DIR = HAPPY_FIXTURE_DIR;

export type ProofBundle = {
  proof: Buffer;
  journal: Buffer;
  /** Where the bytes came from. */
  source: "fixture" | "host";
};

export type LoadProofOptions = {
  /**
   * When true, run `cargo run -p zk-circuit-host --features network --release`
   * with `SP1_USE_NETWORK=1 SP1_GROTH16=1` (needs `$PROVE` / root `.env`).
   * Default false — use committed fixtures.
   */
  invokeHost?: boolean;
  /** Override fixture directory. */
  fixtureDir?: string;
  /** Host cwd (repo root). */
  repoRoot?: string;
  /** Gitignored witness JSON (`leaves_hex` + `leaf_index` for a multi-leaf tree). */
  witnessJson?: string;
  /**
   * Host writes groth16/journal here. When set, skip copying into
   * `fixtures/happy/` and load this pack instead.
   */
  artifactDir?: string;
};

/** Read `groth16.bin` + `journal.bin` (104 spend-all or 144 change-note). */
export function loadProofFixture(fixtureDir: string): ProofBundle {
  const proofPath = join(fixtureDir, "groth16.bin");
  const journalPath = join(fixtureDir, "journal.bin");
  if (!existsSync(proofPath) || !existsSync(journalPath)) {
    throw new Error(
      `missing proof fixtures under ${fixtureDir} (need groth16.bin + journal.bin)`,
    );
  }
  const proof = readFileSync(proofPath);
  const journal = readFileSync(journalPath);
  if (proof.length !== 356) {
    throw new Error(`expected groth16.bin length 356, got ${proof.length}`);
  }
  if (journal.length !== 104 && journal.length !== 144) {
    throw new Error(
      `expected journal.bin length 104 or 144, got ${journal.length}`,
    );
  }
  return { proof, journal, source: "fixture" };
}

/** Read committed toy pack: 144-byte journal (spend 100 / change 900). */
export function loadHappyProofFixture(
  fixtureDir: string = HAPPY_FIXTURE_DIR,
): ProofBundle {
  const bundle = loadProofFixture(fixtureDir);
  if (fixtureDir === HAPPY_FIXTURE_DIR && bundle.journal.length !== 144) {
    throw new Error(
      `expected happy journal.bin length 144, got ${bundle.journal.length}`,
    );
  }
  return bundle;
}

/** @deprecated Alias of {@link loadHappyProofFixture}. */
export function loadChangeProofFixture(
  fixtureDir: string = HAPPY_FIXTURE_DIR,
): ProofBundle {
  return loadHappyProofFixture(fixtureDir);
}

/**
 * Prove bridge entrypoint.
 * - Default / CI: fixtures
 * - `invokeHost: true` or `PROVE_BRIDGE_HOST=1`: spawn host network prove
 *   (happy pack unless `artifactDir` / `witnessJson` are set).
 */
export async function getSettleProof(
  opts: LoadProofOptions = {},
): Promise<ProofBundle> {
  const invoke =
    opts.invokeHost === true || process.env.PROVE_BRIDGE_HOST === "1";
  if (!invoke) {
    return loadHappyProofFixture(opts.fixtureDir);
  }
  await runHostGroth16Prove(opts);
  if (opts.artifactDir) {
    const proofPath = join(opts.artifactDir, "groth16.bin");
    if (!existsSync(proofPath)) {
      throw new Error(
        "host skipped Groth16 — set SP1_USE_NETWORK=1 SP1_GROTH16=1 in root .env ($PROVE is Succinct credits, not a Solana key)",
      );
    }
    const bundle = loadProofFixture(opts.artifactDir);
    return { ...bundle, source: "host" };
  }
  return loadHappyProofFixture(opts.fixtureDir ?? HAPPY_FIXTURE_DIR);
}

function runHostGroth16Prove(opts: LoadProofOptions): Promise<void> {
  const repoRoot = opts.repoRoot ?? REPO_ROOT;
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    SP1_USE_NETWORK: "1",
    SP1_PROVER: "network",
    SP1_GROTH16: "1",
    RUST_LOG: process.env.RUST_LOG ?? "info",
  };
  if (opts.witnessJson) env.WITNESS_JSON = opts.witnessJson;
  if (opts.artifactDir) {
    env.ARTIFACT_DIR = opts.artifactDir;
    env.SKIP_HAPPY_FIXTURE = "1";
  }
  return new Promise((resolve, reject) => {
    const child = spawn(
      "cargo",
      [
        "run",
        "-p",
        "zk-circuit-host",
        "--features",
        "network",
        "--release",
      ],
      {
        cwd: repoRoot,
        env,
        stdio: "inherit",
      },
    );
    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0) resolve();
      else reject(new Error(`zk-circuit-host exited with code ${code}`));
    });
  });
}

/** Parse journal public fields (same layout as on-chain `JournalPublic`). */
export function parseJournalPublic(journal: Buffer): {
  requestedSwapAmount: bigint;
  assetIdMint: Buffer;
  nullifier: Buffer;
  merkleRoot: Buffer;
  changeAmount: bigint;
  changeLeaf: Buffer;
} {
  if (journal.length !== 104 && journal.length !== 144) {
    throw new Error(`journal must be 104 or 144 bytes, got ${journal.length}`);
  }
  const changeAmount =
    journal.length === 144 ? journal.readBigUInt64LE(104) : 0n;
  const changeLeaf =
    journal.length === 144 ? journal.subarray(112, 144) : Buffer.alloc(32);
  if (changeAmount > 0n && changeLeaf.every((b) => b === 0)) {
    throw new Error("change_amount > 0 requires a non-zero change_leaf");
  }
  return {
    requestedSwapAmount: journal.readBigUInt64LE(0),
    assetIdMint: journal.subarray(8, 40),
    nullifier: journal.subarray(40, 72),
    merkleRoot: journal.subarray(72, 104),
    changeAmount,
    changeLeaf,
  };
}
