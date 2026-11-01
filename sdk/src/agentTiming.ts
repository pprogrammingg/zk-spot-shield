/** Wall-clock laps from “buy Token B” until mint B hits the wallet ATA. */

export type BuyLap = { name: string; ms: number };

export function formatClock(at: number): string {
  const d = new Date(at);
  const pad = (n: number, w = 2) => String(n).padStart(w, "0");
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}.${pad(d.getMilliseconds(), 3)}`;
}

export function formatDuration(ms: number): string {
  if (ms < 1000) return `${Math.max(0, Math.round(ms))}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

export function formatBuyWalletTiming(args: {
  startedAt: number;
  laps: BuyLap[];
  walletMs: number;
}): string {
  const landed = args.startedAt + args.walletMs;
  const lines = [
    `Token B in wallet at ${formatClock(landed)}  total ${formatDuration(args.walletMs)}`,
    `  started       ${formatClock(args.startedAt)}`,
  ];
  for (const lap of args.laps) {
    lines.push(`  ${lap.name.padEnd(13)} ${formatDuration(lap.ms)}`);
  }
  return lines.join("\n");
}

export class BuyTimer {
  readonly startedAt: number;
  readonly laps: BuyLap[] = [];
  private last: number;

  constructor(startedAt: number = Date.now()) {
    this.startedAt = startedAt;
    this.last = startedAt;
  }

  mark(name: string, at: number = Date.now()): number {
    const ms = at - this.last;
    this.laps.push({ name, ms });
    this.last = at;
    return ms;
  }

  elapsed(at: number = Date.now()): number {
    return at - this.startedAt;
  }

  walletLines(at: number = Date.now()): string {
    return formatBuyWalletTiming({
      startedAt: this.startedAt,
      laps: this.laps,
      walletMs: this.elapsed(at),
    });
  }
}
