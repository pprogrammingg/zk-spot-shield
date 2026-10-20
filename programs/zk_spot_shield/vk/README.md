# SP1 Groth16 verifying key (on-chain)

`groth16_vk_v6_1_0.bin` is the SP1 circuits **v6.1.0** Groth16 verifying key
(`~/.sp1/circuits/groth16/v6.1.0/groth16_vk.bin`, also in `sp1-verifier` artifacts).

The 356-byte `proof.bytes()` is verified against this file and `GlobalConfig.vkey_hash`
(the program’s `vk.bytes32()`). Guest/ELF change → re-prove → update `VKEY_HASH`.
Circuit version bump → replace this file and `SP1_V6_1_0_VK_ROOT` in `verify_sp1.rs`.
