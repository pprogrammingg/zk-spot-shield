# Account layout and signers

Engineer audit of zero-copy sizes and who must sign each instruction.

## Zero-copy / `repr(C)`

| Account | Layout | Size | Padding |
| --- | --- | --- | --- |
| `VaultState` | `zero_copy` `repr(C)` | 120 | `_padding: [u8; 7]` after `bump` |
| `DepositLeafAccount` | `zero_copy` `repr(C)` | 104 | already 8-byte aligned |
| `NullifierAccount` | `zero_copy` `repr(C)` | 32 | none |
| `CleanFundsRoot` | `zero_copy` `repr(C)` | 32 | none |
| `GlobalConfig` | `Account` + `InitSpace` | 65 data | `pause_flag` is a bool |

Settle loads vault and root with `AccountLoader`. Size tests live in `state.rs`.

## Instruction signers

| Instruction | Must sign | Other writable | Constraint |
| --- | --- | --- | --- |
| `initialize_global_config` | `payer` | new config PDA | payer funds rent |
| `initialize_vault` | `payer` | new vault PDA | payer funds rent |
| `register_clean_funds_root` | `authority` | new root PDA | `has_one = authority` |
| `deposit_shielded` | `depositor` | vault, leaf, token accounts | `user_token.owner == depositor` |
| `settle_shielded_spot` | `payer` (fee payer) | vault, nullifier, token accounts | vault PDA signs the SPL payout; user token accounts need not match payer |
| `pause` / `unpause` | `authority` | config | `has_one = authority` |

Settle `payer` does **not** need to own the user token accounts. Deposit still requires the token owner to sign.
