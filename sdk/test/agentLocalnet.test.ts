import { describe, expect, it } from "vitest";
import { DEVNET_FIXED, agentCluster } from "../src/agentLocalnet.js";

describe("agent cluster pins", () => {
  it("Devnet vault mint A is DNNk… not toy 0x03", () => {
    expect(DEVNET_FIXED.mintA).toBe("DNNkZidPWCHxoyuaQ3h5N8FUCWaJZddEYvF7JyHHKbVE");
    expect(DEVNET_FIXED.mintB.startsWith("Hfgiz")).toBe(true);
    expect(agentCluster()).toBe("localnet");
  });
});
