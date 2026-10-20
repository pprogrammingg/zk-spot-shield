# `@zk-spot-shield/sdk`

TypeScript client for deposit-backed **vault-only** settle (Architecture phases **C–E**).

- **C** Prove: Merkle path + fixture Groth16 (no `$PROVE` by default)
- **D** Settle: vault pays the other mint; fee payer may be a **relayer**
- **E** Relayer: encode `SettleIntent` for `POST /submit-intent`

Package name on npm-style imports stays `@zk-spot-shield/sdk` (roadmap alias `@zk-shield/sdk`).

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
  feePayer: relayerPubkey, // ≠ note owner
  vaultTokenA,
  vaultTokenB,
  userTokenA: depositorAtaA,
  userTokenB: depositorAtaB,
  recipientTokenA: recipientAtaA, // optional: recipient ≠ depositor
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

Recipient ATAs in `plan.accounts.userTokenA/B` are the **payout** accounts. Full unlink (fresh ATA + relayer-only submit so metas do not dox the note owner) may land with UI Days 66+.

## HTTP 402 (Day 50)

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

Unlock is the settle **nullifier**, not a wallet pubkey.
