import { useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import { DEFAULT_PROGRAM_ID } from "./config.ts";
import {
  AGENT_STATUS_POLL_MS,
  DEVNET_VAULT,
  chainPollMs,
  parseClusterParam,
  rpcForCluster,
  type Cluster,
} from "./cluster.ts";
import {
  bytesToHex,
  decodeBase64,
  decodeGlobalConfig,
  decodeSplAmount,
  decodeVaultState,
  parseJournal,
  type JournalView,
} from "./decode.ts";
import {
  discoverAgentOrigin,
  fetchAgentStatus,
  parseUiBuyAmount,
  pickSpendableNoteIndex,
  postAgentBuy,
  postAgentDeposit,
  type AgentStatus,
} from "./agentBuy.ts";
import { loadAgentNotesPublic, loadDemoPack, type AgentNotePublic } from "./demo.ts";
import { readDemoSession, writeDemoSession } from "./session.ts";
import { accountUrl, localConfirmCmd, txUrl } from "./explorer.ts";
import { keypairFromJsonText } from "./keypair.ts";
import { NoteCard } from "./NoteCard.tsx";
import { isSpendSettled, noteOldBalance } from "./noteStory.ts";
import {
  findCleanFundsRootPda,
  findDepositLeafPda,
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
  vkeyHash: string;
};

type TokenBalances = {
  vaultA: string;
  vaultB: string;
  userA: string;
  userB: string;
};

async function splAmount(
  connection: Connection,
  pk: string,
): Promise<string> {
  if (!pk.trim()) return "—";
  const acc = await connection.getAccountInfo(new PublicKey(pk));
  if (!acc) return "missing";
  return decodeSplAmount(acc.data).toString();
}

function changeNoteJson(pub: JournalView, sig: string | null): string {
  return JSON.stringify(
    {
      kind: "zk-spot-shield-change-note",
      spent: pub.requestedSwapAmount.toString(),
      changeAmount: pub.changeAmount.toString(),
      changeLeafHex: bytesToHex(pub.changeLeaf),
      nullifierHex: bytesToHex(pub.nullifier),
      merkleRootHex: bytesToHex(pub.merkleRoot),
      mintHex: bytesToHex(pub.assetIdMint),
      settleSig: sig,
      note: "Remainder is spendable after the protocol admin registers a Merkle root that includes changeLeaf.",
    },
    null,
    2,
  );
}

function initialCluster(): Cluster {
  if (typeof window === "undefined") return "localnet";
  return parseClusterParam(window.location.search) ?? "localnet";
}

export default function App() {
  const saved = readDemoSession();
  const [cluster, setCluster] = useState<Cluster>(initialCluster);
  const bootDevnet = cluster === "devnet";
  const [rpcList, setRpcList] = useState(
    () => rpcForCluster(initialCluster()),
  );
  const [onChainOpen, setOnChainOpen] = useState(
    () => typeof window !== "undefined" && window.location.hash === "#on-chain",
  );
  const [programIdStr, setProgramIdStr] = useState(
    bootDevnet
      ? DEVNET_VAULT.programId
      : saved?.programIdStr || DEFAULT_PROGRAM_ID,
  );
  const [relayerUrl, setRelayerUrl] = useState("");
  const [vaultTokenA, setVaultTokenA] = useState(
    bootDevnet ? DEVNET_VAULT.vaultTokenA : saved?.vaultTokenA ?? "",
  );
  const [vaultTokenB, setVaultTokenB] = useState(
    bootDevnet ? DEVNET_VAULT.vaultTokenB : saved?.vaultTokenB ?? "",
  );
  const [userTokenA, setUserTokenA] = useState(
    bootDevnet ? "" : saved?.userTokenA ?? "",
  );
  const [userTokenB, setUserTokenB] = useState(
    bootDevnet ? "" : saved?.userTokenB ?? "",
  );
  const [proofB64, setProofB64] = useState(
    bootDevnet ? "" : saved?.proofB64 ?? "",
  );
  const [journalB64, setJournalB64] = useState(
    bootDevnet ? "" : saved?.journalB64 ?? "",
  );
  const [status, setStatus] = useState(
    bootDevnet
      ? "Devnet. Polling the agent for notes and ATAs."
      : saved?.journalB64
        ? "Restored the last loaded note (Vite reloads wipe in-memory state)."
        : "Idle.",
  );
  const [chain, setChain] = useState<ChainView | null>(null);
  const [balances, setBalances] = useState<TokenBalances | null>(null);
  const [walletLabel, setWalletLabel] = useState("not connected");
  const [localKp, setLocalKp] = useState<Keypair | null>(null);
  const [txSig, setTxSig] = useState<string | null>(
    bootDevnet ? null : saved?.txSig ?? null,
  );
  const [pay402, setPay402] = useState("");
  const [agentNotes, setAgentNotes] = useState<AgentNotePublic[]>([]);
  const [buyAmount, setBuyAmount] = useState("");
  const [buying, setBuying] = useState(false);
  const [lockedLeaf, setLockedLeaf] = useState<number | null>(null);
  const [depositAmount, setDepositAmount] = useState("");
  const [depositing, setDepositing] = useState(false);
  const pollRef = useRef({
    rpcList,
    programIdStr,
    vaultTokenA,
    vaultTokenB,
    userTokenA,
    userTokenB,
  });
  pollRef.current = {
    rpcList,
    programIdStr,
    vaultTokenA,
    vaultTokenB,
    userTokenA,
    userTokenB,
  };

  const journalPreview = useMemo(() => {
    if (!journalB64.trim()) return null;
    try {
      return parseJournal(decodeBase64(journalB64));
    } catch {
      return null;
    }
  }, [journalB64]);

  const settled = Boolean(
    txSig ||
      (journalPreview &&
        isSpendSettled(balances?.userB, journalPreview.requestedSwapAmount)),
  );

  useEffect(() => {
    if (!journalB64) return;
    writeDemoSession({
      proofB64,
      journalB64,
      programIdStr,
      vaultTokenA,
      vaultTokenB,
      userTokenA,
      userTokenB,
      txSig,
    });
  }, [
    proofB64,
    journalB64,
    programIdStr,
    vaultTokenA,
    vaultTokenB,
    userTokenA,
    userTokenB,
    txSig,
  ]);

  useEffect(() => {
    if (cluster === "localnet") {
      loadDemo(rpcForCluster("localnet")).catch((e) => setStatus(String(e)));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function applyStatusBundle(st: AgentStatus) {
    setProgramIdStr(st.programId);
    setVaultTokenA(st.vaultTokenA);
    setVaultTokenB(st.vaultTokenB);
    setUserTokenA(st.userTokenA);
    setUserTokenB(st.userTokenB);
    setAgentNotes(st.notes);
    pollRef.current = {
      ...pollRef.current,
      programIdStr: st.programId,
      vaultTokenA: st.vaultTokenA,
      vaultTokenB: st.vaultTokenB,
      userTokenA: st.userTokenA,
      userTokenB: st.userTokenB,
    };
  }

  useEffect(() => {
    let stop = false;
    const pullStatus = async () => {
      if (stop || document.hidden) return;
      try {
        const origin = await discoverAgentOrigin();
        if (origin) {
          const st = await fetchAgentStatus(origin);
          if (st && st.cluster === cluster && !stop) {
            applyStatusBundle(st);
          } else if (!st) {
            const notes = await loadAgentNotesPublic();
            if (!stop) setAgentNotes(notes);
          }
        } else {
          const notes = await loadAgentNotesPublic();
          if (!stop) setAgentNotes(notes);
        }
      } catch {
        /* agent optional */
      }
    };
    const pullChain = async () => {
      if (stop || document.hidden) return;
      const p = pollRef.current;
      try {
        const { url, connection } = await connectWithFallback(p.rpcList);
        const programId = new PublicKey(p.programIdStr);
        const config = findGlobalConfigPda(programId);
        const vault = findVaultPda(programId);
        const cfgAcc = await connection.getAccountInfo(config);
        const vaultAcc = await connection.getAccountInfo(vault);
        if (!cfgAcc || !vaultAcc || stop) return;
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
          vkeyHash: bytesToHex(cfg.vkeyHash),
        });
        await loadTokenBalances(connection, {
          vaultTokenA: p.vaultTokenA,
          vaultTokenB: p.vaultTokenB,
          userTokenA: p.userTokenA,
          userTokenB: p.userTokenB,
        });
      } catch {
        /* validator / RPC blip */
      }
    };
    void pullStatus();
    void pullChain();
    const statusId = window.setInterval(() => void pullStatus(), AGENT_STATUS_POLL_MS);
    const chainId = window.setInterval(() => void pullChain(), chainPollMs(cluster));
    return () => {
      stop = true;
      window.clearInterval(statusId);
      window.clearInterval(chainId);
    };
  }, [cluster]);

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

  async function loadTokenBalances(
    connection: Connection,
    atas: {
      vaultTokenA: string;
      vaultTokenB: string;
      userTokenA: string;
      userTokenB: string;
    },
  ) {
    const [vaultA, vaultB, userA, userB] = await Promise.all([
      splAmount(connection, atas.vaultTokenA),
      splAmount(connection, atas.vaultTokenB),
      splAmount(connection, atas.userTokenA),
      splAmount(connection, atas.userTokenB),
    ]);
    setBalances({ vaultA, vaultB, userA, userB });
  }

  function syncClusterInUrl(next: Cluster) {
    if (typeof window === "undefined") return;
    const u = new URL(window.location.href);
    u.searchParams.set("cluster", next);
    window.history.replaceState(null, "", u);
  }

  async function applyCluster(next: Cluster) {
    setCluster(next);
    const rpc = rpcForCluster(next);
    setRpcList(rpc);
    syncClusterInUrl(next);
    if (next === "localnet") {
      await loadDemo(rpc);
      return;
    }
    setProofB64("");
    setJournalB64("");
    setTxSig(null);
    setAgentNotes([]);
    setProgramIdStr(DEVNET_VAULT.programId);
    const atas = {
      vaultTokenA: DEVNET_VAULT.vaultTokenA,
      vaultTokenB: DEVNET_VAULT.vaultTokenB,
      userTokenA: "",
      userTokenB: "",
    };
    setVaultTokenA(atas.vaultTokenA);
    setVaultTokenB(atas.vaultTokenB);
    setUserTokenA("");
    setUserTokenB("");
    pollRef.current = {
      rpcList: rpc,
      programIdStr: DEVNET_VAULT.programId,
      vaultTokenA: atas.vaultTokenA,
      vaultTokenB: atas.vaultTokenB,
      userTokenA: "",
      userTokenB: "",
    };
    setStatus("Devnet. Polling the agent for notes and ATAs.");
    await refresh(rpc, atas, next);
  }

  async function refresh(
    rpcOverride?: string,
    atasOverride?: {
      vaultTokenA: string;
      vaultTokenB: string;
      userTokenA: string;
      userTokenB: string;
    },
    clusterOverride?: Cluster,
  ) {
    setStatus("Fetching on-chain state…");
    let atas = atasOverride ?? {
      vaultTokenA,
      vaultTokenB,
      userTokenA,
      userTokenB,
    };
    const want = clusterOverride ?? cluster;
    try {
      const origin = await discoverAgentOrigin();
      if (origin) {
        const st = await fetchAgentStatus(origin);
        if (st && st.cluster === want) {
          applyStatusBundle(st);
          atas = {
            vaultTokenA: st.vaultTokenA,
            vaultTokenB: st.vaultTokenB,
            userTokenA: st.userTokenA,
            userTokenB: st.userTokenB,
          };
        }
      }
    } catch {
      /* agent optional */
    }
    const { url, connection } = await connectWithFallback(
      rpcOverride ?? rpcList,
    );
    const programId = new PublicKey(pollRef.current.programIdStr);
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
      vkeyHash: bytesToHex(cfg.vkeyHash),
    });
    await loadTokenBalances(connection, atas);
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

  function applyKeypairText(text: string) {
    const kp = keypairFromJsonText(text);
    setLocalKp(kp);
    setWalletLabel(`${kp.publicKey.toBase58()} (local keypair — localnet only)`);
    setStatus("Loaded localnet fee-payer keypair. Do not use a mainnet secret here.");
  }

  function onKeyfile(file: File | undefined) {
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      try {
        applyKeypairText(String(reader.result));
      } catch (e) {
        setStatus(String(e));
      }
    };
    reader.readAsText(file);
  }

  async function loadDemo(rpcOverride?: string) {
    setStatus("Loading localnet demo pack…");
    const pack = await loadDemoPack();
    setProgramIdStr(pack.accounts.programId);
    setVaultTokenA(pack.accounts.vaultTokenA);
    setVaultTokenB(pack.accounts.vaultTokenB);
    setUserTokenA(pack.accounts.userTokenA);
    setUserTokenB(pack.accounts.userTokenB);
    setProofB64(pack.proofB64);
    setJournalB64(pack.journalB64);
    setTxSig(null);
    setPay402("");
    const { url, connection } = await connectWithFallback(
      rpcOverride ?? rpcList,
    );
    const programId = new PublicKey(pack.accounts.programId);
    const config = findGlobalConfigPda(programId);
    const vault = findVaultPda(programId);
    const cfgAcc = await connection.getAccountInfo(config);
    const vaultAcc = await connection.getAccountInfo(vault);
    if (!cfgAcc || !vaultAcc) {
      throw new Error("Config or vault missing. Is npm run demo:localnet still running?");
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
      vkeyHash: bytesToHex(cfg.vkeyHash),
    });
    await loadTokenBalances(connection, pack.accounts);
    setStatus(
      "Localnet. User A/B in On-chain data are public SPL. Note cards are the fixture pack.",
    );
  }

  function settleKeys(pub: JournalView, payer: PublicKey) {
    const programId = new PublicKey(programIdStr);
    return {
      payer,
      globalConfig: findGlobalConfigPda(programId),
      vault: findVaultPda(programId),
      cleanFundsRoot: findCleanFundsRootPda(programId, pub.merkleRoot),
      nullifierAccount: findNullifierPda(programId, pub.nullifier),
      vaultTokenA: new PublicKey(vaultTokenA),
      vaultTokenB: new PublicKey(vaultTokenB),
      userTokenA: new PublicKey(userTokenA),
      userTokenB: new PublicKey(userTokenB),
      ...(pub.changeAmount > 0n
        ? { changeLeaf: findDepositLeafPda(programId, pub.changeLeaf) }
        : {}),
    };
  }

  async function doSelfSubmit() {
    const proof = decodeBase64(proofB64);
    const journal = decodeBase64(journalB64);
    const pub = parseJournal(journal);
    const { connection, wallet, rpc } = await signer();
    const programId = new PublicKey(programIdStr);
    const keys = settleKeys(pub, wallet.publicKey);
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
    setStatus(`Settled ${sig} via ${rpc}. You received ${pub.requestedSwapAmount.toString()} mint B; the ${pub.changeAmount.toString()} remainder is the gold card.`);
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
    const keys = settleKeys(pub, wallet.publicKey);
    const job = await submitViaRelayer(
      relayerUrl,
      toRelayerIntent(keys, proof, journal, Number(pub.requestedSwapAmount)),
    );
    setStatus(`Relayer job ${job.job_id} (${job.status}). Queue may be dry-run.`);
  }

  async function quote402() {
    const base = relayerUrl.trim().replace(/\/$/, "");
    if (!base) throw new Error("Set Relayer URL (http://127.0.0.1:8787) first.");
    const res = await fetch(`${base}/402/resource`);
    const body = await res.text();
    setPay402(`${res.status} ${body}`);
    setStatus(res.status === 402 ? "402 quote (expected until you pay)." : `402 resource ${res.status}.`);
  }

  async function pay402WithJournal() {
    const base = relayerUrl.trim().replace(/\/$/, "");
    if (!base) throw new Error("Set Relayer URL (http://127.0.0.1:8787) first.");
    if (!journalB64.trim()) throw new Error("Load the demo journal first.");
    const res = await fetch(`${base}/402/pay`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ journal_base64: journalB64.trim() }),
    });
    const body = await res.text();
    setPay402(`${res.status} ${body}`);
    if (!res.ok) {
      throw new Error(`402 pay failed: ${res.status} ${body}`);
    }
    const pub = journalPreview;
    const nf = pub ? `0x${bytesToHex(pub.nullifier)}` : "";
    const unlocked = await fetch(`${base}/402/resource?nullifier=${encodeURIComponent(nf)}`);
    const unlockedBody = await unlocked.text();
    setPay402(`${res.status} paid. Resource ${unlocked.status}: ${unlockedBody}`);
    setStatus("402 unlocked with the settle journal (fixture path — no live prove).");
  }

  async function agentOriginOrThrow(): Promise<string> {
    const origin = await discoverAgentOrigin();
    if (!origin) {
      throw new Error("Start the agent first: cd sdk && npm run agent");
    }
    return origin;
  }

  async function depositTokenA(e: FormEvent) {
    e.preventDefault();
    const amount = parseUiBuyAmount(depositAmount);
    setDepositing(true);
    setStatus(`Depositing ${amount} Token A (no Groth16)…`);
    try {
      const origin = await agentOriginOrThrow();
      const result = await postAgentDeposit(origin, amount);
      setTxSig(result.sig);
      setStatus(
        `Deposited ${result.balance} A at idx ${result.leafIndex}. sig ${result.sig}`,
      );
      await refresh();
    } finally {
      setDepositing(false);
    }
  }

  async function buyTokenB(e: FormEvent) {
    e.preventDefault();
    const amount = parseUiBuyAmount(buyAmount);
    const picked =
      agentNotes.length === 0
        ? null
        : pickSpendableNoteIndex(agentNotes, BigInt(amount));
    if (agentNotes.length > 0 && picked === null) {
      setStatus(`no note covers ${amount}. Deposit more A.`);
      return;
    }
    setLockedLeaf(picked);
    setBuying(true);
    setStatus(
      `Buying ${amount} Token B (1:1). Groth16 runs in the agent — keep this tab open.`,
    );
    try {
      const origin = await agentOriginOrThrow();
      const result = await postAgentBuy(origin, amount);
      setTxSig(result.settleSig);
      const change = result.changeBalance
        ? ` Change note ${result.changeBalance} A.`
        : " Spend-all (no change).";
      setStatus(
        `Settled ${result.settleSig}. Spent note ${result.spentIndex}.${change}`,
      );
      await refresh();
    } finally {
      setBuying(false);
      setLockedLeaf(null);
    }
  }

  function downloadChange() {
    if (!journalPreview || journalPreview.changeAmount === 0n) {
      throw new Error("No change amount in the journal.");
    }
    const blob = new Blob([changeNoteJson(journalPreview, txSig)], {
      type: "application/json",
    });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = "zk-spot-shield-change-900.json";
    a.click();
    URL.revokeObjectURL(a.href);
    setStatus("Downloaded a client copy of the 900 note. The gold card is the note; this file is a backup.");
  }

  const err = status.toLowerCase().includes("error") || status.toLowerCase().includes("failed");
  const cfgHref = chain ? accountUrl(chain.config, chain.rpc) : null;
  const vaultHref = chain ? accountUrl(chain.vault, chain.rpc) : null;
  const sigHref = txSig && chain ? txUrl(txSig, chain.rpc) : null;

  return (
    <main className="page">
      <header>
        <div>
          <h1>ZK Spot Shield</h1>
          <p className="lede">
            Private mint A notes. Type an amount to buy Token B 1:1 — same path as the agent CLI.
          </p>
        </div>
        <div className="cluster" role="group" aria-label="Cluster">
          <button
            type="button"
            aria-pressed={cluster === "localnet"}
            onClick={() => applyCluster("localnet").catch((e) => setStatus(String(e)))}
          >
            Localnet
          </button>
          <button
            type="button"
            aria-pressed={cluster === "devnet"}
            onClick={() => applyCluster("devnet").catch((e) => setStatus(String(e)))}
          >
            Devnet
          </button>
        </div>
      </header>

      {agentNotes.length > 0 ? (
        <section className="notes" aria-label="CLI notes">
          {agentNotes.map((n) => (
            <NoteCard
              key={`${n.leafIndex}-${n.leafPrefix}`}
              kind="spend"
              amount={n.balance}
              settled={n.spent}
              locked={lockedLeaf === n.leafIndex}
            />
          ))}
        </section>
      ) : cluster === "localnet" && journalPreview ? (
        <section className="notes">
          <NoteCard
            kind="spend"
            amount={noteOldBalance(journalPreview).toString()}
            settled={settled}
            locked={buying && !settled}
          />
          {journalPreview.changeAmount > 0n && settled && (
            <NoteCard
              kind="change"
              amount={journalPreview.changeAmount.toString()}
              settled={settled}
            >
              <div className="row">
                <button type="button" onClick={() => downloadChange().catch((e) => setStatus(String(e)))}>
                  Download backup
                </button>
              </div>
            </NoteCard>
          )}
        </section>
      ) : (
        <p className="muted">
          {cluster === "devnet"
            ? "Deposit Token A below. Notes come from the running agent."
            : "Deposit Token A below, or load the localnet demo."}
        </p>
      )}

      <form
        className="trade"
        onSubmit={(e) => depositTokenA(e).catch((err) => setStatus(String(err)))}
      >
        <label>
          Amount of Token A to deposit
          <div className="buy-row">
            <input
              type="text"
              inputMode="numeric"
              pattern="[0-9]*"
              autoComplete="off"
              placeholder="e.g. 33"
              value={depositAmount}
              disabled={buying || depositing}
              onChange={(e) => setDepositAmount(e.target.value)}
            />
            <button type="submit" className="primary" disabled={buying || depositing}>
              {depositing ? "Depositing…" : "Deposit Token A"}
            </button>
          </div>
        </label>
      </form>

      <form className="trade" onSubmit={(e) => buyTokenB(e).catch((err) => setStatus(String(err)))}>
        <label>
          Amount of Token A to use to Buy Token B (1:1 ratio)
          <div className="buy-row">
            <input
              type="text"
              inputMode="numeric"
              pattern="[0-9]*"
              autoComplete="off"
              placeholder="e.g. 33"
              value={buyAmount}
              disabled={buying || depositing}
              onChange={(e) => setBuyAmount(e.target.value)}
            />
            <button type="submit" className="primary" disabled={buying || depositing}>
              {buying ? "Buying…" : "Buy Token B"}
            </button>
          </div>
        </label>
        {buying ? (
          <p className="muted help">Groth16 prove is running in the agent terminal. This can take a minute.</p>
        ) : null}
      </form>

      <section className="row">
        <button
          type="button"
          disabled={cluster !== "localnet"}
          onClick={() => loadDemo().catch((e) => setStatus(String(e)))}
        >
          Load localnet demo
        </button>
      </section>

      <p className={err ? "err" : "status"}>{status}</p>
      {txSig && chain && (
        <p className="mono">
          {sigHref ? (
            <a href={sigHref} target="_blank" rel="noreferrer">
              Explorer tx
            </a>
          ) : (
            <>
              Localnet signature: {txSig}
              <br />
              <span className="muted">{localConfirmCmd(txSig, chain.rpc)}</span>
            </>
          )}
        </p>
      )}

      <details className="fold">
        <summary>Fee payer — {walletLabel}</summary>
        <p className="muted help">
          Solana CLI default wallet is in a hidden folder. Choose File → <kbd>⌘⇧G</kbd> →{" "}
          <code>~/.config/solana/id.json</code>, or paste the JSON array. Localnet only.
        </p>
        <label className="file">
          Choose keypair JSON
          <input
            type="file"
            accept=".json,application/json"
            onChange={(e) => onKeyfile(e.target.files?.[0])}
          />
        </label>
        <label>
          Or paste keypair JSON
          <textarea
            rows={2}
            placeholder="[1, 2, 3, …]"
            onChange={(e) => {
              const t = e.target.value.trim();
              if (!t.startsWith("[")) return;
              try {
                applyKeypairText(t);
              } catch {
                /* keep typing */
              }
            }}
          />
        </label>
        <div className="row">
          <button type="button" onClick={() => connectInjected().catch((e) => setStatus(String(e)))}>
            Connect wallet
          </button>
        </div>
      </details>

      <details
        className="fold"
        id="on-chain"
        open={onChainOpen}
        onToggle={(e) => setOnChainOpen((e.target as HTMLDetailsElement).open)}
      >
        <summary>
          On-chain data
          <button
            type="button"
            className="icon-btn"
            title="Refresh"
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              refresh().catch((err) => setStatus(String(err)));
            }}
          >
            ↻
          </button>
        </summary>
        {chain ? (
          <dl className="stats">
            <dt>RPC</dt>
            <dd>{chain.rpc}</dd>
            <dt>Paused</dt>
            <dd>{chain.pauseFlag ? "yes" : "no"}</dd>
            <dt>Vault A / B</dt>
            <dd>
              {balances
                ? `${balances.vaultA} / ${balances.vaultB}`
                : `${chain.reserveA} / ${chain.reserveB}`}
            </dd>
            <dt>User Balance A / B</dt>
            <dd>
              {balances
                ? `${balances.userA} / ${balances.userB}`
                : "— / —"}
            </dd>
            <dt>Notes</dt>
            <dd>
              {agentNotes.length === 0
                ? "none yet — CLI Deposit Token A"
                : agentNotes.map((n) => (
                    <div key={`${n.leafIndex}-${n.leafPrefix}`}>
                      idx {n.leafIndex} · {n.balance} A · {n.spent ? "spent" : "open"}
                    </div>
                  ))}
            </dd>
            <dt>Vault A account</dt>
            <dd className="mono">{vaultTokenA || "—"}</dd>
            <dt>Vault B account</dt>
            <dd className="mono">{vaultTokenB || "—"}</dd>
            <dt>User A account</dt>
            <dd className="mono">{userTokenA || "—"}</dd>
            <dt>User B account</dt>
            <dd className="mono">{userTokenB || "—"}</dd>
            <dt>Mint A / B</dt>
            <dd className="mono">
              {chain.mintA} / {chain.mintB}
            </dd>
            <dt>vkey</dt>
            <dd className="mono">0x{chain.vkeyHash.slice(0, 16)}…</dd>
            <dt>Explorer</dt>
            <dd>
              {cfgHref && vaultHref ? (
                <>
                  <a href={cfgHref} target="_blank" rel="noreferrer">
                    GlobalConfig
                  </a>
                  {" · "}
                  <a href={vaultHref} target="_blank" rel="noreferrer">
                    Vault
                  </a>
                </>
              ) : (
                <span className="muted">Localnet — public explorer cannot reach 127.0.0.1.</span>
              )}
            </dd>
          </dl>
        ) : (
          <p className="muted">Load the demo or press ↻.</p>
        )}
        <div className="grid">
          <label>
            RPC list
            <textarea
              rows={2}
              value={rpcList}
              onChange={(e) => setRpcList(e.target.value)}
            />
          </label>
          <label>
            Program id
            <input value={programIdStr} onChange={(e) => setProgramIdStr(e.target.value)} />
          </label>
          <label>
            Vault ATA A
            <input value={vaultTokenA} onChange={(e) => setVaultTokenA(e.target.value)} />
          </label>
          <label>
            Vault ATA B
            <input value={vaultTokenB} onChange={(e) => setVaultTokenB(e.target.value)} />
          </label>
          <label>
            User Balance A
            <input value={userTokenA} onChange={(e) => setUserTokenA(e.target.value)} />
          </label>
          <label>
            User Balance B
            <input value={userTokenB} onChange={(e) => setUserTokenB(e.target.value)} />
          </label>
        </div>
      </details>

      <details className="fold">
        <summary>Proof and journal bytes</summary>
        <p className="muted help">
          Packed happy-path bytes (spend 100 / keep 900). Buy Token B above uses the agent, not this
          fixture.
        </p>
        <div className="row">
          <button type="button" onClick={() => doSelfSubmit().catch((e) => setStatus(String(e)))}>
            Submit fixture settle
          </button>
        </div>
        <label className="span2">
          Groth16 proof (base64, 356 bytes)
          <textarea rows={3} value={proofB64} onChange={(e) => setProofB64(e.target.value)} />
        </label>
        <label className="span2">
          Journal (base64, 144 bytes; 104 = spend-all)
          <textarea rows={2} value={journalB64} onChange={(e) => setJournalB64(e.target.value)} />
        </label>
      </details>

      <details className="fold">
        <summary>Optional: HTTP 402 (same journal, not on-chain)</summary>
        <p className="muted help">
          Separate from settle. Start <code>cargo run -p zk-spot-shield-relayer</code>, set the URL,
          Quote then Pay. Unlock is keyed by nullifier.
        </p>
        <label>
          Relayer URL
          <input
            value={relayerUrl}
            onChange={(e) => setRelayerUrl(e.target.value)}
            placeholder="http://127.0.0.1:8787"
          />
        </label>
        <div className="row">
          <button type="button" onClick={() => doRelayer().catch((e) => setStatus(String(e)))}>
            Ask relayer
          </button>
          <button type="button" onClick={() => quote402().catch((e) => setStatus(String(e)))}>
            Quote 402
          </button>
          <button type="button" onClick={() => pay402WithJournal().catch((e) => setStatus(String(e)))}>
            Pay 402 with journal
          </button>
        </div>
        {pay402 && <pre className="mono pay402-out">{pay402}</pre>}
      </details>
    </main>
  );
}
