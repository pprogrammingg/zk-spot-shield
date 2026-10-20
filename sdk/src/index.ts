/**
 * TS client: localhost provider, Poseidon Merkle, prove bridge, settle ix wrappers.
 */

export {
  LOCALNET_URL,
  createLocalhostProvider,
  defaultWalletPath,
  isLocalhostUrl,
  loadKeypairWallet,
  type LocalhostProviderOptions,
} from "./provider.js";

export {
  MERKLE_DEPTH,
  balanceToFieldBytes,
  bytes32ToHex,
  computeLeaf,
  computeNullifier,
  hashNodes,
  hexToBytes32,
  index0EmptyPath,
  verifyMerklePath,
  type MerklePathEntry,
} from "./merkle.js";

export {
  DepositTree,
  privateInputsFromDeposit,
  type DepositNote,
} from "./depositTree.js";

export {
  HAPPY_FIXTURE_DIR,
  REPO_ROOT,
  getSettleProof,
  loadHappyProofFixture,
  parseJournalPublic,
  type LoadProofOptions,
  type ProofBundle,
} from "./prove.js";

export {
  SETTLE_ALT_STATIC_LABELS,
  SETTLE_CU_LIMIT,
  SETTLE_CU_PRICE_MICRO_LAMPORTS,
  SEEDS,
  buildDepositIx,
  buildSettleIx,
  buildSettleTransaction,
  confirmSignature,
  createProgram,
  findCleanFundsRootPda,
  findDepositLeafPda,
  findGlobalConfigPda,
  findNullifierPda,
  findVaultPda,
  loadIdl,
  programIdFromIdl,
  settleAltStaticAddresses,
  settleComputeBudgetIxs,
  type DepositAccounts,
  type SettleAccounts,
} from "./instructions.js";

export {
  decodeVaultState,
  encodeSettleIntent,
  generateProofInputs,
  getVaultReserves,
  proofInputsToHex,
  requestShieldedSwap,
  subscribeCleanFundsRoot,
  vaultStateDiscriminator,
  type ProofInputs,
  type RelayerSettleIntent,
  type ShieldedSwapPlan,
  type ShieldedSwapRequest,
  type SubscribeCleanFundsRootOpts,
  type VaultReserves,
} from "./client.js";

export {
  journalToPayBody,
  payUrl,
  quoteUrl,
  resourceUrl,
  type PaidResource,
  type PayAck,
  type PaymentQuote,
} from "./pay402.js";
