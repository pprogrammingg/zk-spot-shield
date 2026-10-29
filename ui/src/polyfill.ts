import { Buffer } from "buffer";

const g = globalThis as typeof globalThis & { Buffer: typeof Buffer };
g.Buffer = Buffer;
if (typeof window !== "undefined") {
  (window as typeof window & { Buffer: typeof Buffer }).Buffer = Buffer;
}
