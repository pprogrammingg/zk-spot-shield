#!/usr/bin/env bash
# SHA-256 of the built program .so (see notes/budgets.md).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

found=0
for p in target/deploy/zk_spot_shield.so target/sbpf-solana-solana/release/zk_spot_shield.so; do
 if [[ -f "$p" ]]; then
 found=1
 echo -n "$p "
 shasum -a 256 "$p"
 fi
done

if [[ "$found" -eq 0 ]]; then
 echo "no zk_spot_shield.so — run: export PATH=\"\$HOME/.local/share/solana/install/active_release/bin:\$PATH\" && anchor build --ignore-keys" >&2
 exit 1
fi
