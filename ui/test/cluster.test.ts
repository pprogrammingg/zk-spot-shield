import { describe, expect, it } from "vitest";
import {
  AGENT_STATUS_POLL_MS,
  DEVNET_VAULT,
  chainPollMs,
  clusterFromRpc,
  parseClusterParam,
  rpcForCluster,
} from "../src/cluster.ts";

describe("cluster toggle", () => {
  it("parses ?cluster=", () => {
    expect(parseClusterParam("?cluster=localnet")).toBe("localnet");
    expect(parseClusterParam("cluster=devnet")).toBe("devnet");
    expect(parseClusterParam("")).toBeNull();
  });

  it("maps RPC", () => {
    expect(rpcForCluster("localnet")).toBe("http://127.0.0.1:8899");
    expect(rpcForCluster("devnet")).toContain("devnet");
    expect(clusterFromRpc("https://api.devnet.solana.com")).toBe("devnet");
    expect(clusterFromRpc("http://127.0.0.1:8899")).toBe("localnet");
  });

  it("Devnet chain poll is slower than localnet; agent status is loopback", () => {
    expect(chainPollMs("devnet")).toBe(15_000);
    expect(chainPollMs("localnet")).toBe(8_000);
    expect(AGENT_STATUS_POLL_MS).toBe(4_000);
    expect(chainPollMs("devnet")).toBeGreaterThan(AGENT_STATUS_POLL_MS);
  });

  it("Devnet vault ATAs are pinned (user ATAs come from the agent)", () => {
    expect(DEVNET_VAULT.programId.startsWith("74UW")).toBe(true);
    expect(DEVNET_VAULT.mintA.startsWith("DNNk")).toBe(true);
    expect(DEVNET_VAULT.vaultTokenA.startsWith("EVmR")).toBe(true);
    expect(DEVNET_VAULT.vaultTokenB.startsWith("8wBh")).toBe(true);
  });
});
