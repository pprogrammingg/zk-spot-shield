import { Keypair } from "@solana/web3.js";
import { Wallet } from "@coral-xyz/anchor";
import { describe, expect, it } from "vitest";
import {
  LOCALNET_URL,
  createLocalhostProvider,
  isLocalhostUrl,
} from "../src/provider.js";

describe("localhost Anchor provider", () => {
  it("defaults to loopback localnet URL", () => {
    expect(LOCALNET_URL).toBe("http://127.0.0.1:8899");
    expect(isLocalhostUrl(LOCALNET_URL)).toBe(true);
    expect(isLocalhostUrl("https://api.devnet.solana.com")).toBe(false);
  });

  it("createLocalhostProvider wires Connection + Wallet without needing a live validator", () => {
    const wallet = new Wallet(Keypair.generate());
    const provider = createLocalhostProvider({
      wallet,
      setGlobal: false,
    });
    expect(provider.connection.rpcEndpoint).toBe(LOCALNET_URL);
    expect(provider.wallet.publicKey.equals(wallet.publicKey)).toBe(true);
    expect(isLocalhostUrl(provider.connection.rpcEndpoint)).toBe(true);
  });

  it("respects url override", () => {
    const wallet = new Wallet(Keypair.generate());
    const provider = createLocalhostProvider({
      wallet,
      url: "http://localhost:8899",
      setGlobal: false,
    });
    expect(provider.connection.rpcEndpoint).toBe("http://localhost:8899");
    expect(isLocalhostUrl(provider.connection.rpcEndpoint)).toBe(true);
  });
});
