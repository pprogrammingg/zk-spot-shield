import { encodeBase64 } from "./decode.ts";

export type DemoAccounts = {
  programId: string;
  authority: string;
  vaultTokenA: string;
  vaultTokenB: string;
  userTokenA: string;
  userTokenB: string;
};

export type DemoPack = {
  accounts: DemoAccounts;
  proofB64: string;
  journalB64: string;
};

export function demoAssetUrl(name: string): string {
  const base = import.meta.env.BASE_URL ?? "./";
  const prefix = base.endsWith("/") ? base : `${base}/`;
  return `${prefix}demo/${name}`;
}

function isDemoAccounts(v: unknown): v is DemoAccounts {
  if (!v || typeof v !== "object") return false;
  const o = v as Record<string, unknown>;
  return (
    typeof o.programId === "string" &&
    typeof o.authority === "string" &&
    typeof o.vaultTokenA === "string" &&
    typeof o.vaultTokenB === "string" &&
    typeof o.userTokenA === "string" &&
    typeof o.userTokenB === "string"
  );
}

export type AgentNotePublic = {
  leafIndex: number;
  balance: string;
  spent: boolean;
  leafPrefix: string;
};

export async function loadAgentNotesPublic(
  fetchFn: typeof fetch = fetch,
): Promise<AgentNotePublic[]> {
  const res = await fetchFn(`${demoAssetUrl("agent-notes.json")}?t=${Date.now()}`);
  if (!res.ok) return [];
  const v = (await res.json()) as { notes?: unknown };
  if (!Array.isArray(v.notes)) return [];
  return v.notes.filter((n): n is AgentNotePublic => {
    if (!n || typeof n !== "object") return false;
    const o = n as Record<string, unknown>;
    return (
      typeof o.leafIndex === "number" &&
      typeof o.balance === "string" &&
      typeof o.spent === "boolean" &&
      typeof o.leafPrefix === "string"
    );
  });
}

export async function loadDemoPack(
  fetchFn: typeof fetch = fetch,
): Promise<DemoPack> {
  const [accountsRes, proofRes, journalRes] = await Promise.all([
    fetchFn(demoAssetUrl("accounts.json")),
    fetchFn(demoAssetUrl("groth16.bin")),
    fetchFn(demoAssetUrl("journal.bin")),
  ]);
  if (!accountsRes.ok) {
    throw new Error(
      "Demo ATAs missing. From sdk/: npm run demo:localnet (leave it running), then Load localnet demo.",
    );
  }
  if (!proofRes.ok || !journalRes.ok) {
    throw new Error("Demo proof/journal missing. Re-run npm run demo:localnet.");
  }
  const accounts = (await accountsRes.json()) as unknown;
  if (!isDemoAccounts(accounts)) {
    throw new Error("ui/public/demo/accounts.json is not a demo bundle.");
  }
  const proof = new Uint8Array(await proofRes.arrayBuffer());
  const journal = new Uint8Array(await journalRes.arrayBuffer());
  if (proof.length !== 356) {
    throw new Error(`demo groth16.bin must be 356 bytes, got ${proof.length}`);
  }
  if (journal.length !== 144) {
    throw new Error(`demo journal.bin must be 144 bytes, got ${journal.length}`);
  }
  return {
    accounts,
    proofB64: encodeBase64(proof),
    journalB64: encodeBase64(journal),
  };
}
