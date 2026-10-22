# Instruction signers

Who must sign each of the nine instructions. Sizes: [`budgets.md`](./budgets.md).

| Instruction | Must sign | Other writable | Constraint |
| --- | --- | --- | --- |
| `initialize_global_config` | `payer` | new config PDA | payer funds rent |
| `initialize_vault` | `payer` | new vault PDA | payer funds rent |
| `set_vault_mints` | `authority` | vault | `has_one = authority` on `GlobalConfig`; mints once |
| `set_vkey_hash` | `authority` | config | `has_one = authority`; rejects the all-zero hash |
| `register_clean_funds_root` | `authority` | new root PDA | `has_one = authority` |
| `deposit_shielded` | `depositor` | vault, leaf, token accounts | `user_token.owner == depositor` |
| `settle_shielded_spot` | `payer` (fee payer) | vault, nullifier, token accounts | vault PDA signs the SPL payout; user token accounts need not match payer |
| `pause` / `unpause` | `authority` | config | `has_one = authority` |

Settle `payer` does **not** need to own the user token accounts. Deposit still requires the token owner to sign. Test: `instruction_signer_inventory` in `constants.rs`.
