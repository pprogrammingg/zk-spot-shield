import { useMemo, useState } from "react";
import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import { DEFAULT_PROGRAM_ID, DEFAULT_RPC_LIST } from "./config.ts";
import {
  bytesToHex,
  decodeBase64,
  decodeGlobalConfig,
  decodeVaultState,
  parseJournal,
} from "./decode.ts";
import {
  findCleanFundsRootPda,
  findGlobalConfigPda,
  findNullifierPda,
  findVaultPda,
} from "./pda.ts";
import { connectWithFallback } from "./rpc.ts";
import { keypairSigner, submitMyself, submitViaRelayer, toRelayerIntent } from "./submit.ts";
import type { WalletSigner } from "./submit.ts";
import "./App.css";

type ChainView = {
  rpc: string;
  pauseFlag: boolean;
  reserveA: string;
  reserveB: string;
  mintA: string;
  mintB: string;
  config: string;
  vault: string;
};

function explorerCluster(rpc: string): string {
  if (rpc.includes("devnet")) return "devnet";
  if (rpc.includes("127.0.0.1") || rpc.includes("localhost")) return "custom";
  return "devnet";
}

function accountUrl(pk: string, rpc: string): string {
  const cluster = explorerCluster(rpc);
  if (cluster === "custom") {
    return `https://explorer.solana.com/address/${pk}?cluster=custom&customUrl=${encodeURIComponent(rpc)}`;
  }
  return `https://explorer.solana.com/address/${pk}?cluster=${cluster}`;
}

function txUrl(sig: string, rpc: string): string {
  const cluster = explorerCluster(rpc);
  if (cluster === "custom") {
    return `https://explorer.solana.com/tx/${sig}?cluster=custom&customUrl=${encodeURIComponent(rpc)}`;
  }
  return `https://explorer.solana.com/tx/${sig}?cluster=${cluster}`;
}

export default function App() {
  const [rpcList, setRpcList] = useState(DEFAULT_RPC_LIST);
  const [programIdStr, setProgramIdStr] = useState(DEFAULT_PROGRAM_ID);
  const [relayerUrl, setRelayerUrl] = useState("");
  const [vaultTokenA, setVaultTokenA] = useState("");
  const [vaultTokenB, setVaultTokenB] = useState("");
  const [userTokenA, setUserTokenA] = useState("");
  const [userTokenB, setUserTokenB] = useState("");
  const [proofB64, setProofB64] = useState("");
  const [journalB64, setJournalB64] = useState("");
  const [status, setStatus] = useState("Idle.");
  const [chain, setChain] = useState<ChainView | null>(null);
  const [walletLabel, setWalletLabel] = useState("not connected");
  const [localKp, setLocalKp] = useState<Keypair | null>(null);
  const [txSig, setTxSig] = useState<string | null>(null);

  const journalPreview = useMemo(() => {
    if (!journalB64.trim()) return null;
    try {
      return parseJournal(decodeBase64(journalB64));
    } catch {
      return null;
    }
  }, [journalB64]);

  async function signer(): Promise<{ connection: Connection; wallet: WalletSigner; rpc: string }> {
    const { url, connection } = await connectWithFallback(rpcList);
    if (localKp) {
      return { connection, wallet: keypairSigner(localKp), rpc: url };
    }
    const injected = window.solana;
    if (!injected?.connect) {
      throw new Error("No wallet. Connect Phantom or load a localnet keypair JSON.");
    }
    await injected.connect();
    const pk = injected.publicKey;
    if (!pk) throw new Error("wallet connected without a pubkey");
    const wallet: WalletSigner = {
      publicKey: new PublicKey(pk.toBase58()),
      signTransaction: (tx: Transaction) => injected.signTransaction(tx),
    };
    return { connection, wallet, rpc: url };
  }

  async function refresh() {
    setStatus("Fetching on-chain state…");
    const { url, connection } = await connectWithFallback(rpcList);
    const programId = new PublicKey(programIdStr);
    const config = findGlobalConfigPda(programId);
    const vault = findVaultPda(programId);
    const cfgAcc = await connection.getAccountInfo(config);
    const vaultAcc = await connection.getAccountInfo(vault);
    if (!cfgAcc || !vaultAcc) {
      throw new Error("Config or vault missing on this cluster. Init first.");
    }
    const cfg = decodeGlobalConfig(cfgAcc.data);
    const v = decodeVaultState(vaultAcc.data);
    setChain({
      rpc: url,
      pauseFlag: cfg.pauseFlag,
      reserveA: v.reserveA.toString(),
      reserveB: v.reserveB.toString(),
      mintA: v.mintA.toBase58(),
      mintB: v.mintB.toBase58(),
      config: config.toBase58(),
      vault: vault.toBase58(),
    });
    setStatus(`RPC ${url}. Pause=${cfg.pauseFlag}.`);
  }

  async function connectInjected() {
    const injected = window.solana;
    if (!injected?.connect) {
      throw new Error("No injected wallet (Phantom). For localnet, load a keypair JSON.");
    }
    const res = await injected.connect();
    setLocalKp(null);
    setWalletLabel(res.publicKey.toBase58());
    setStatus("Injected wallet connected.");
  }

  function onKeyfile(file: File | undefined) {
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      const nums = JSON.parse(String(reader.result)) as number[];
      const kp = Keypair.fromSecretKey(Uint8Array.from(nums));
      setLocalKp(kp);
      setWalletLabel(`${kp.publicKey.toBase58()} (local keypair — localnet only)`);
      setStatus("Loaded localnet fee-payer keypair. Do not use a mainnet secret here.");
    };
    reader.readAsText(file);
  }

  async function doSelfSubmit() {
    const proof = decodeBase64(proofB64);
    const journal = decodeBase64(journalB64);
    const pub = parseJournal(journal);
    const { connection, wallet, rpc } = await signer();
    const programId = new PublicKey(programIdStr);
    const keys = {
      payer: wallet.publicKey,
      globalConfig: findGlobalConfigPda(programId),
      vault: findVaultPda(programId),
      cleanFundsRoot: findCleanFundsRootPda(programId, pub.merkleRoot),
      nullifierAccount: findNullifierPda(programId, pub.nullifier),
      vaultTokenA: new PublicKey(vaultTokenA),
      vaultTokenB: new PublicKey(vaultTokenB),
      userTokenA: new PublicKey(userTokenA),
      userTokenB: new PublicKey(userTokenB),
    };
    setStatus("Sending settle (self-submit)…");
    const sig = await submitMyself(
      connection,
      programId,
      keys,
      proof,
      journal,
      wallet,
    );
    setTxSig(sig);
    setStatus(`Submitted ${sig} via ${rpc}`);
    await refresh();
  }

  async function doRelayer() {
    if (!relayerUrl.trim()) {
      throw new Error("Relayer URL is empty. Self-submit does not need a relayer.");
    }
    const proof = decodeBase64(proofB64);
    const journal = decodeBase64(journalB64);
    const pub = parseJournal(journal);
    const { wallet } = await signer();
    const programId = new PublicKey(programIdStr);
    const keys = {
      payer: wallet.publicKey,
      globalConfig: findGlobalConfigPda(programId),
      vault: findVaultPda(programId),
      cleanFundsRoot: findCleanFundsRootPda(programId, pub.merkleRoot),
      nullifierAccount: findNullifierPda(programId, pub.nullifier),
      vaultTokenA: new PublicKey(vaultTokenA),
      vaultTokenB: new PublicKey(vaultTokenB),
      userTokenA: new PublicKey(userTokenA),
      userTokenB: new PublicKey(userTokenB),
    };
    const job = await submitViaRelayer(
      relayerUrl,
      toRelayerIntent(keys, proof, journal, Number(pub.requestedSwapAmount)),
    );
    setStatus(`Relayer job ${job.job_id} (${job.status}). Queue may be dry-run.`);
  }

  return (
    <main className="page">
      <header>
        <h1>ZK Spot Shield</h1>
        <p className="lede">
          Vault-only 1:1 spot settle after Groth16. This is a <strong>shielded settle</strong>, not a
          mixer — a deposit still shows the depositor on-chain.
        </p>
      </header>

      <section className="grid">
        <label>
          RPC list (fallback)
          <textarea
            rows={3}
            value={rpcList}
            onChange={(e) => setRpcList(e.target.value)}
            placeholder="http://127.0.0.1:8899, https://api.devnet.solana.com"
          />
        </label>
        <label>
          Program id
          <input value={programIdStr} onChange={(e) => setProgramIdStr(e.target.value)} />
        </label>
        <label>
          Relayer URL (optional)
          <input
            value={relayerUrl}
            onChange={(e) => setRelayerUrl(e.target.value)}
            placeholder="http://127.0.0.1:8787"
          />
        </label>
      </section>

      <section className="row">
        <button type="button" onClick={() => connectInjected().catch((e) => setStatus(String(e)))}>
          Connect wallet
        </button>
        <label className="file">
          Localnet keypair JSON
          <input
            type="file"
            accept=".json,application/json"
            onChange={(e) => onKeyfile(e.target.files?.[0])}
          />
        </label>
        <span className="muted">{walletLabel}</span>
      </section>

      <section>
        <button type="button" onClick={() => refresh().catch((e) => setStatus(String(e)))}>
          Refresh on-chain
        </button>
        {chain && (
          <dl className="stats">
            <dt>RPC</dt>
            <dd>{chain.rpc}</dd>
            <dt>Paused</dt>
            <dd>{chain.pauseFlag ? "yes" : "no"}</dd>
            <dt>Reserve A / B</dt>
            <dd>
              {chain.reserveA} / {chain.reserveB}
            </dd>
            <dt>Mint A / B</dt>
            <dd className="mono">
              {chain.mintA} / {chain.mintB}
            </dd>
            <dt>Explorer</dt>
            <dd>
              <a href={accountUrl(chain.config, chain.rpc)} target="_blank" rel="noreferrer">
                GlobalConfig
              </a>
              {" · "}
              <a href={accountUrl(chain.vault, chain.rpc)} target="_blank" rel="noreferrer">
                Vault
              </a>
            </dd>
          </dl>
        )}
      </section>

      <section className="grid">
        <label>
          Vault ATA A
          <input value={vaultTokenA} onChange={(e) => setVaultTokenA(e.target.value)} />
        </label>
        <label>
          Vault ATA B
          <input value={vaultTokenB} onChange={(e) => setVaultTokenB(e.target.value)} />
        </label>
        <label>
          Recipient ATA A
          <input value={userTokenA} onChange={(e) => setUserTokenA(e.target.value)} />
        </label>
        <label>
          Recipient ATA B
          <input value={userTokenB} onChange={(e) => setUserTokenB(e.target.value)} />
        </label>
        <label className="span2">
          Groth16 proof (base64, 356 bytes decoded)
          <textarea rows={4} value={proofB64} onChange={(e) => setProofB64(e.target.value)} />
        </label>
        <label className="span2">
          Journal (base64, 104 bytes decoded)
          <textarea rows={3} value={journalB64} onChange={(e) => setJournalB64(e.target.value)} />
        </label>
      </section>

      {journalPreview && (
        <p className="muted">
          Journal amount {journalPreview.requestedSwapAmount.toString()} · nullifier{" "}
          {bytesToHex(journalPreview.nullifier).slice(0, 16)}…
        </p>
      )}

      <section className="row">
        <button
          type="button"
          className="primary"
          onClick={() => doSelfSubmit().catch((e) => setStatus(String(e)))}
        >
          Submit myself
        </button>
        <button type="button" onClick={() => doRelayer().catch((e) => setStatus(String(e)))}>
          Ask relayer
        </button>
      </section>

      <p className={status.toLowerCase().includes("error") ? "err" : "status"}>{status}</p>
      {txSig && chain && (
        <p>
          <a href={txUrl(txSig, chain.rpc)} target="_blank" rel="noreferrer">
            Explorer tx
          </a>
        </p>
      )}
    </main>
  );
}
