/**
 * Public A→B spot (contrast with `buyMintB` / settle_shielded_spot).
 * Flow: 402 quote → public_spot_swap → POST /402/pay-public (unlock=wallet).
 */

import { PublicKey, Transaction } from "@solana/web3.js";
import { BuyTimer, formatClock, formatDuration } from "./agentTiming.js";
import { agentCluster, fetch402Quote, type AgentCtx } from "./agentLocalnet.js";
import {
  buildPublicSpotSwapIx,
  confirmSignature,
  createProgram,
  findGlobalConfigPda,
  findVaultPda,
} from "./instructions.js";
import { bytes32ToHex } from "./merkle.js";
import { createLocalhostProvider, loadKeypairWallet } from "./provider.js";
import { payPublicUrl, type PaymentQuote } from "./pay402.js";
import { decimalsForCluster, formatTokenAmount } from "./tokenAmount.js";

export type NonShieldedPayResult = {
  quote: PaymentQuote;
  swapSig: string;
  pay: { paid: boolean; signature: string; payer: string; amount: number };
};

export async function post402PayPublic(
  relayerUrl: string,
  body: { signature: string; payer: string; amount: number | string },
): Promise<{ paid: boolean; signature: string; payer: string; amount: number }> {
  const res = await fetch(payPublicUrl(relayerUrl), {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      signature: body.signature,
      payer: body.payer,
      amount: Number(body.amount),
    }),
  });
  const text = await res.text();
  if (!res.ok) {
    throw new Error(`POST /402/pay-public ${res.status}: ${text}`);
  }
  return JSON.parse(text) as {
    paid: boolean;
    signature: string;
    payer: string;
    amount: number;
  };
}

/** Same quote as shielded buy; payer wallet is visible on-chain. */
export async function nonShieldedPay(
  ctx: AgentCtx,
  amount: bigint,
  log: (line: string) => void,
): Promise<NonShieldedPayResult> {
  if (amount <= 0n) throw new Error("amount must be > 0");

  const timer = new BuyTimer();
  const dec = decimalsForCluster(agentCluster());
  log(
    `non-shielded swap ${formatTokenAmount(amount, dec)} A→B  started ${formatClock(timer.startedAt)}`,
  );

  const mintHex = bytes32ToHex(
    Uint8Array.from(new PublicKey(ctx.bundle.mintA).toBytes()),
  );
  const quote = await fetch402Quote(ctx.relayerUrl, amount, mintHex);
  const quoteMint = quote.mint_hex.replace(/^0x/i, "").toLowerCase();
  const wantMint = mintHex.replace(/^0x/i, "").toLowerCase();
  if (quoteMint !== wantMint) {
    throw new Error(`402 mint ${quote.mint_hex} ≠ vault mint A ${mintHex}`);
  }
  const quoteAmt = BigInt(quote.amount);
  if (quoteAmt !== amount) {
    throw new Error(`402 amount ${quoteAmt} ≠ requested ${amount}`);
  }
  timer.mark("402 quote");
  log(
    `402 PAYMENT REQUIRED  ${formatTokenAmount(quote.amount, dec)} mint ${quote.mint_hex.slice(0, 12)}…  unlock=wallet (public path)`,
  );

  const provider = createLocalhostProvider({
    setGlobal: false,
    url: ctx.rpcUrl,
  });
  const program = createProgram(provider);
  const wallet = loadKeypairWallet();
  const [globalConfig] = await findGlobalConfigPda(program.programId);
  const [vault] = await findVaultPda(program.programId);

  log("==> public_spot_swap (no Groth16)");
  const ix = await buildPublicSpotSwapIx(
    program,
    {
      payer: wallet.publicKey,
      globalConfig,
      vault,
      vaultTokenA: new PublicKey(ctx.bundle.vaultTokenA),
      vaultTokenB: new PublicKey(ctx.bundle.vaultTokenB),
      userTokenA: new PublicKey(ctx.bundle.userTokenA),
      userTokenB: new PublicKey(ctx.bundle.userTokenB),
    },
    quoteAmt,
  );
  const swapSig = await provider.sendAndConfirm(new Transaction().add(ix));
  await confirmSignature(provider.connection, swapSig);
  const swapMs = timer.mark("public swap");
  log(`==> public swap done in ${formatDuration(swapMs)}  sig ${swapSig}`);
  log(timer.walletLines());

  const pay = await post402PayPublic(ctx.relayerUrl, {
    signature: swapSig,
    payer: wallet.publicKey.toBase58(),
    amount: Number(quoteAmt),
  });
  log(
    `402 PAID  unlock=wallet  payer ${pay.payer.slice(0, 8)}…  sig ${pay.signature.slice(0, 12)}…`,
  );

  return { quote, swapSig, pay };
}
