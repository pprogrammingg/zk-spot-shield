# SP1 Groth16 verifying key (on-chain)

`groth16_vk_v6_1_0.bin` is the SP1 **circuits v6.1.0** Groth16 VK
(`~/.sp1/circuits/groth16/v6.1.0/groth16_vk.bin`, also in `sp1-verifier` vk-artifacts).

Day 12 `proof.bytes()` (356 B) are verified against this file + `GlobalConfig.vkey_hash`
(program `vk.bytes32()`). Guest/ELF change → re-prove → update `VKEY_HASH`; circuit
version bump → replace this VK and `SP1_V6_1_0_VK_ROOT` in `verify_sp1.rs`.
