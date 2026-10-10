/**
 * Day 34: prove bridge — return Groth16 proof + journal bytes for settle.
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
};

/** Read committed `groth16.bin` + `journal.bin` (Day 12 wrap). */
export function loadHappyProofFixture(
  fixtureDir: string = HAPPY_FIXTURE_DIR,
): ProofBundle {
  const proofPath = join(fixtureDir, "groth16.bin");
  const journalPath = join(fixtureDir, "journal.bin");
  if (!existsSync(proofPath) || !existsSync(journalPath)) {
    throw new Error(
      `missing happy fixtures under ${fixtureDir} (need groth16.bin + journal.bin)`,
    );
  }
  const proof = readFileSync(proofPath);
  const journal = readFileSync(journalPath);
  if (proof.length !== 356) {
    throw new Error(`expected groth16.bin length 356, got ${proof.length}`);
  }
  if (journal.length !== 104) {
    throw new Error(`expected journal.bin length 104, got ${journal.length}`);
  }
  return { proof, journal, source: "fixture" };
}

/**
 * Prove bridge entrypoint.
 * - Default / CI: fixtures
 * - `invokeHost: true` or `PROVE_BRIDGE_HOST=1`: spawn host network prove, then reload fixtures
 *   (host copies artifacts into `zk-circuit/fixtures/happy/`).
 */
export async function getSettleProof(
  opts: LoadProofOptions = {},
): Promise<ProofBundle> {
  const invoke =
    opts.invokeHost === true || process.env.PROVE_BRIDGE_HOST === "1";
  if (!invoke) {
    return loadHappyProofFixture(opts.fixtureDir);
  }
  await runHostGroth16Prove(opts.repoRoot ?? REPO_ROOT);
  return loadHappyProofFixture(opts.fixtureDir ?? HAPPY_FIXTURE_DIR);
}

function runHostGroth16Prove(repoRoot: string): Promise<void> {
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
        env: {
          ...process.env,
          SP1_USE_NETWORK: "1",
          SP1_PROVER: "network",
          SP1_GROTH16: "1",
          RUST_LOG: process.env.RUST_LOG ?? "info",
        },
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
} {
  if (journal.length !== 104) {
    throw new Error(`journal must be 104 bytes, got ${journal.length}`);
  }
  return {
    requestedSwapAmount: journal.readBigUInt64LE(0),
    assetIdMint: journal.subarray(8, 40),
    nullifier: journal.subarray(40, 72),
    merkleRoot: journal.subarray(72, 104),
  };
}
