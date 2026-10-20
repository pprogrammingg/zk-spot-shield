import { Connection } from "@solana/web3.js";

export type RpcHealthFetch = (
  url: string,
  init: { method: string; headers: Record<string, string>; body: string },
) => Promise<{ ok: boolean; json: () => Promise<{ result?: unknown; error?: unknown }> }>;

export function parseRpcList(raw: string): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const part of raw.split(/[\s,]+/)) {
    const url = part.trim();
    if (!url || seen.has(url)) continue;
    seen.add(url);
    out.push(url);
  }
  return out;
}

const HEALTH_BODY = JSON.stringify({
  jsonrpc: "2.0",
  id: 1,
  method: "getHealth",
});

export async function firstHealthyRpc(
  urls: string[],
  fetchImpl: RpcHealthFetch,
): Promise<string> {
  if (urls.length === 0) {
    throw new Error("rpc list is empty");
  }
  const errors: string[] = [];
  for (const url of urls) {
    try {
      const res = await fetchImpl(url, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: HEALTH_BODY,
      });
      if (!res.ok) {
        errors.push(`${url}: http not ok`);
        continue;
      }
      const body = await res.json();
      if (body.error) {
        errors.push(`${url}: json-rpc error`);
        continue;
      }
      return url;
    } catch (e) {
      errors.push(`${url}: ${e instanceof Error ? e.message : String(e)}`);
    }
  }
  throw new Error(`all RPCs failed: ${errors.join("; ")}`);
}

export async function connectWithFallback(rawList: string): Promise<{
  url: string;
  connection: Connection;
}> {
  const urls = parseRpcList(rawList);
  const url = await firstHealthyRpc(urls, async (u, init) => {
    const res = await fetch(u, init);
    return {
      ok: res.ok,
      json: () => res.json() as Promise<{ result?: unknown; error?: unknown }>,
    };
  });
  return { url, connection: new Connection(url, "confirmed") };
}
