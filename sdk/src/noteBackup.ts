/**
 * Encrypted note backup. Passphrase → AES-256-GCM blob the user can store
 * off-device. Decrypt locally to restore secret / balance / address / leaf index
 * (including a change-note remainder). Not an NFT. Not on-chain.
 */

export type NoteBackupPlain = {
  secret: Uint8Array;
  userAddress: Uint8Array;
  balance: bigint;
  assetIdMint: Uint8Array;
  leafIndex: number;
};

const BACKUP_VERSION = 1;
const PBKDF2_ITERS = 210_000;
const SALT_LEN = 16;
const IV_LEN = 12;

function cryptoApi(): Crypto {
  const c = globalThis.crypto;
  if (!c?.subtle) {
    throw new Error("Web Crypto is required for note backup");
  }
  return c;
}

function bytesToB64(bytes: Uint8Array): string {
  return Buffer.from(bytes).toString("base64");
}

function b64ToBytes(s: string): Uint8Array {
  return new Uint8Array(Buffer.from(s, "base64"));
}

function encodePlain(note: NoteBackupPlain): Uint8Array {
  if (note.secret.length !== 32 || note.userAddress.length !== 32) {
    throw new Error("secret and userAddress must be 32 bytes");
  }
  if (note.assetIdMint.length !== 32) {
    throw new Error("assetIdMint must be 32 bytes");
  }
  const json = JSON.stringify({
    v: BACKUP_VERSION,
    secret: bytesToB64(note.secret),
    userAddress: bytesToB64(note.userAddress),
    balance: note.balance.toString(),
    assetIdMint: bytesToB64(note.assetIdMint),
    leafIndex: note.leafIndex,
  });
  return new TextEncoder().encode(json);
}

function decodePlain(bytes: Uint8Array): NoteBackupPlain {
  const raw = JSON.parse(new TextDecoder().decode(bytes)) as {
    v: number;
    secret: string;
    userAddress: string;
    balance: string;
    assetIdMint: string;
    leafIndex: number;
  };
  if (raw.v !== BACKUP_VERSION) {
    throw new Error(`unsupported backup version ${raw.v}`);
  }
  return {
    secret: b64ToBytes(raw.secret),
    userAddress: b64ToBytes(raw.userAddress),
    balance: BigInt(raw.balance),
    assetIdMint: b64ToBytes(raw.assetIdMint),
    leafIndex: raw.leafIndex,
  };
}

async function deriveKey(
  passphrase: string,
  salt: Uint8Array,
): Promise<CryptoKey> {
  const subtle = cryptoApi().subtle;
  const base = await subtle.importKey(
    "raw",
    new TextEncoder().encode(passphrase),
    "PBKDF2",
    false,
    ["deriveKey"],
  );
  return subtle.deriveKey(
    {
      name: "PBKDF2",
      salt,
      iterations: PBKDF2_ITERS,
      hash: "SHA-256",
    },
    base,
    { name: "AES-GCM", length: 256 },
    false,
    ["encrypt", "decrypt"],
  );
}

/** Encrypt a note (deposit or change remainder) to a portable JSON blob. */
export async function exportEncryptedNote(
  note: NoteBackupPlain,
  passphrase: string,
): Promise<string> {
  if (!passphrase) throw new Error("passphrase required");
  const c = cryptoApi();
  const salt = c.getRandomValues(new Uint8Array(SALT_LEN));
  const iv = c.getRandomValues(new Uint8Array(IV_LEN));
  const key = await deriveKey(passphrase, salt);
  const ciphertext = new Uint8Array(
    await c.subtle.encrypt({ name: "AES-GCM", iv }, key, encodePlain(note)),
  );
  return JSON.stringify({
    v: BACKUP_VERSION,
    kdf: "pbkdf2-sha256",
    iter: PBKDF2_ITERS,
    salt: bytesToB64(salt),
    iv: bytesToB64(iv),
    ct: bytesToB64(ciphertext),
  });
}

/** Decrypt a blob from {@link exportEncryptedNote}. */
export async function importEncryptedNote(
  blob: string,
  passphrase: string,
): Promise<NoteBackupPlain> {
  if (!passphrase) throw new Error("passphrase required");
  const parsed = JSON.parse(blob) as {
    v: number;
    salt: string;
    iv: string;
    ct: string;
  };
  if (parsed.v !== BACKUP_VERSION) {
    throw new Error(`unsupported backup version ${parsed.v}`);
  }
  const salt = b64ToBytes(parsed.salt);
  const iv = b64ToBytes(parsed.iv);
  const ct = b64ToBytes(parsed.ct);
  const key = await deriveKey(passphrase, salt);
  let plain: ArrayBuffer;
  try {
    plain = await cryptoApi().subtle.decrypt(
      { name: "AES-GCM", iv },
      key,
      ct,
    );
  } catch {
    throw new Error("note backup decrypt failed");
  }
  return decodePlain(new Uint8Array(plain));
}
