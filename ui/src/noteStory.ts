import type { JournalView } from "./decode.ts";

/** Face value of the note this journal spends: spend + change (UTXO). */
export function noteOldBalance(pub: JournalView): bigint {
  return pub.requestedSwapAmount + pub.changeAmount;
}

export function isSpendSettled(
  userB: string | undefined,
  spend: bigint,
): boolean {
  return userB !== undefined && userB === spend.toString();
}
