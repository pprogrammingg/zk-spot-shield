/// <reference types="vite/client" />

interface PhantomProvider {
  isPhantom?: boolean;
  publicKey?: { toBase58(): string };
  connect(): Promise<{ publicKey: { toBase58(): string } }>;
  signTransaction<T>(tx: T): Promise<T>;
}

interface Window {
  solana?: PhantomProvider;
}
