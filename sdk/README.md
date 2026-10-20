# `@zk-spot-shield/sdk`

TypeScript client for deposit-backed, vault-only settle. The vault pays the other mint 1:1. The Solana fee payer may be a relayer, not the note owner.

By default, settle uses committed Groth16 fixtures (no `$PROVE`).

## Usage

```ts
import {
  DepositTree,
  generateProofInputs,
  requestShieldedSwap,
  getVaultReserves,
  encodeSettleIntent,
  createProgram,
  createLocalhostProvider,
  hexToBytes32,
} from "@zk-spot-shield/sdk";

const tree = await DepositTree.create();
const inputs = await generateProofInputs(
  {
    secret: hexToBytes32("0x" + "01".repeat(32)),
    userAddress: hexToBytes32("0x" + "02".repeat(32)),
    balance: 1000,
    assetIdMint: hexToBytes32("0x" + "03".repeat(32)),
  },
  tree,
  100,
);
// `inputs` is the circuit witness. Default settle still uses committed fixtures.

const program = createProgram(createLocalhostProvider({ setGlobal: false }));
const plan = await requestShieldedSwap({
  program,
  feePayer: relayerPubkey, // not the note owner
  vaultTokenA,
  vaultTokenB,
  userTokenA: depositorAtaA,
  userTokenB: depositorAtaB,
  recipientTokenA: recipientAtaA, // optional: payout ≠ depositor
  recipientTokenB: recipientAtaB,
});

const intent = encodeSettleIntent(plan);
await fetch("http://127.0.0.1:8787/submit-intent", {
  method: "POST",
  headers: { "content-type": "application/json" },
  body: JSON.stringify({
    intent_base64: Buffer.from(JSON.stringify(intent)).toString("base64"),
  }),
});

const reserves = await getVaultReserves(connection, plan.accounts.vault);
```

`plan.accounts.userTokenA` / `userTokenB` are the **payout** token accounts. Hiding the recipient still needs a fresh token account and relayer-only submit.

## HTTP 402

```ts
import { journalToPayBody, quoteUrl, resourceUrl } from "@zk-spot-shield/sdk";

const base = "http://127.0.0.1:8787";
const unpaid = await fetch(quoteUrl(base)); // 402 + quote
const ack = await fetch(`${base}/402/pay`, {
  method: "POST",
  headers: { "content-type": "application/json" },
  body: JSON.stringify(journalToPayBody(plan.journal)),
});
const { nullifier_hex } = await ack.json();
const paid = await fetch(resourceUrl(base, nullifier_hex)); // 200
```

Unlock is the settle **nullifier**, not a wallet address.
