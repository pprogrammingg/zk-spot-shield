export const DEMO_SESSION_KEY = "zk-spot-shield-demo";

export type DemoSession = {
  proofB64: string;
  journalB64: string;
  programIdStr: string;
  vaultTokenA: string;
  vaultTokenB: string;
  userTokenA: string;
  userTokenB: string;
  txSig: string | null;
};

export function emptySession(): DemoSession {
  return {
    proofB64: "",
    journalB64: "",
    programIdStr: "",
    vaultTokenA: "",
    vaultTokenB: "",
    userTokenA: "",
    userTokenB: "",
    txSig: null,
  };
}

export function readDemoSession(
  store: Pick<Storage, "getItem"> | null = globalThis.sessionStorage,
): DemoSession | null {
  if (!store) return null;
  try {
    const raw = store.getItem(DEMO_SESSION_KEY);
    if (!raw) return null;
    const v = JSON.parse(raw) as Partial<DemoSession>;
    if (typeof v.journalB64 !== "string" || v.journalB64.length === 0) return null;
    return {
      ...emptySession(),
      ...v,
      txSig: typeof v.txSig === "string" && v.txSig ? v.txSig : null,
    };
  } catch {
    return null;
  }
}

export function writeDemoSession(
  session: DemoSession,
  store: Pick<Storage, "setItem"> | null = globalThis.sessionStorage,
): void {
  if (!store) return;
  store.setItem(DEMO_SESSION_KEY, JSON.stringify(session));
}
