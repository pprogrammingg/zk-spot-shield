/**
 * Mint + deposit Token B into the Devnet vault (fix VaultReserveUnderflow).
 *
 * Usage (from sdk/):
 *   ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
 *   ANCHOR_WALLET=$HOME/.config/solana/id.json \
 *   npx tsx scripts/devnet-fund-b.ts [humanB]
 *
 * Default humanB = 500 (10× a 50 A deposit). Mints use 6 decimals.
 */
import {
  mintAndDeposit,
  openDevnetCtx,
  readVault,
} from "../test/devnet/helpers.ts";

async function main(): Promise<void> {
  const humanB = Number(process.argv[2] ?? "500");
  if (!Number.isFinite(humanB) || humanB <= 0) {
    throw new Error(`bad humanB: ${process.argv[2]}`);
  }
  const raw = BigInt(Math.trunc(humanB)) * 1_000_000n;
  const ctx = await openDevnetCtx();
  const before = await readVault(ctx);
  console.log(
    JSON.stringify(
      {
        humanB,
        raw: raw.toString(),
        beforeB: before.reserveB.toString(),
      },
      null,
      2,
    ),
  );
  const out = await mintAndDeposit(ctx, "b", raw);
  console.log(JSON.stringify(out, null, 2));
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
