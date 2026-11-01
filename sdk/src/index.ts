/**
 * TS client: localhost provider, Poseidon Merkle, prove bridge, settle ix wrappers.
 */

export {
  DEVNET_URL,
  LOCALNET_URL,
  createLocalhostProvider,
  defaultWalletPath,
  isLocalhostUrl,
  loadKeypairWallet,
  type LocalhostProviderOptions,
} from "./provider.js";

export {
  BN254_SCALAR_FIELD,
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
  proofInputsForExistingNote,
  type DepositNote,
} from "./depositTree.js";

export {
  CHANGE_FIXTURE_DIR,
  HAPPY_FIXTURE_DIR,
  REPO_ROOT,
  getSettleProof,
  loadChangeProofFixture,
  loadHappyProofFixture,
  loadProofFixture,
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
  buildSetVkeyHashIx,
  buildSetVaultMintsIx,
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
  decodeGlobalConfig,
  decodeVaultState,
  encodeSettleIntent,
  generateProofInputs,
  getGlobalConfig,
  getVaultReserves,
  proofInputsToHex,
  requestShieldedSwap,
  subscribeCleanFundsRoot,
  vaultStateDiscriminator,
  type GlobalConfigView,
  type ProofInputs,
  type RelayerSettleIntent,
  type ShieldedSwapPlan,
  type ShieldedSwapRequest,
  type SubscribeCleanFundsRootOpts,
  type VaultReserves,
} from "./client.js";

export {
  firstHealthyRpc,
  parseRpcList,
  type RpcHealthFetch,
} from "./rpcFallback.js";

export {
  journalToPayBody,
  payUrl,
  quoteUrl,
  resourceUrl,
  type PaidResource,
  type PayAck,
  type PaymentQuote,
} from "./pay402.js";

export {
  exportEncryptedNote,
  importEncryptedNote,
  type NoteBackupPlain,
} from "./noteBackup.js";

export {
  AGENT_HELP,
  BACK_HINT,
  DEFAULT_UI_URL,
  balanceUiUrl,
  collapseRepeats,
  discoverUiOrigin,
  openBrowser,
  isBackToMenu,
  parseAgentCommand,
  parseAmount,
  parseYesNo,
  uiUrl,
  type AgentCommand,
} from "./agentCmd.js";

export {
  DEVNET_DECIMALS,
  LOCALNET_DECIMALS,
  decimalsForCluster,
  formatTokenAmount,
  parseTokenAmount,
} from "./tokenAmount.js";

export {
  AGENT_HTTP_PORTS,
  DEFAULT_AGENT_HTTP_HOST,
  DEFAULT_AGENT_HTTP_PORT,
  parseBuyBody,
  startAgentHttp,
  withAgentBuyGate,
  type AgentBuyErrJson,
  type AgentBuyFn,
  type AgentBuyOkJson,
  type AgentDepositFn,
  type AgentDepositOkJson,
  type AgentStatusFn,
  type AgentStatusJson,
  type AgentStatusNote,
} from "./agentHttp.js";

export {
  appendLeafHex,
  defaultStatePath,
  emptyAgentState,
  formatNotesTable,
  loadAgentState,
  markSpent,
  noteFromParts,
  pickSpendableNote,
  publicNotesSnapshot,
  publishUiNotes,
  saveAgentState,
  storedToDepositNote,
  type AgentState,
  type PublicNoteView,
  type StoredNote,
} from "./agentStore.js";

export {
  BuyTimer,
  formatBuyWalletTiming,
  formatClock,
  formatDuration,
  type BuyLap,
} from "./agentTiming.js";

export {
  buildWitnessFile,
  writeWitnessJson,
  type AgentWitnessFile,
} from "./agentWitness.js";
