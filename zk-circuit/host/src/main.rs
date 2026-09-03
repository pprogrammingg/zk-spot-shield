//! SP1 host driver: run the guest inside the zkVM **without** proving.
//!
//! SP1 is a zkVM: you write a normal-looking Rust program (the **guest**), compile it
//! to a RISC-V ELF, then this **host** feeds it private inputs. `execute` just runs
//! that program and returns the values it `commit`s (the journal). Later, `prove`
//! produces a Groth16 proof that "this guest ran correctly on some private inputs
//! and produced this journal" — Solana verifies that proof, not the RISC-V code.

// `Prover` must be in scope so `client.execute(...)` resolves (trait method).
use sp1_sdk::{Elf, Prover, ProverClient, SP1Stdin};
use zk_circuit_io::PublicOutputs;

mod fixtures;

// Guest binary from `cargo prove build` (RISC-V, not a normal macOS/Linux exe).
const ELF: Elf = Elf::Static(include_bytes!("../../../target/elf/guest"));

// SP1 6.x client/execute are async; this macro provides the tokio runtime.
#[tokio::main]
async fn main() {
    // Fake note + Merkle path that matches the guest's Poseidon helpers.
    let inputs = fixtures::happy_path_inputs();

    // Bytes the guest will `sp1_zkvm::io::read()` as `PrivateInputs`.
    let mut stdin = SP1Stdin::new();
    stdin.write(&inputs);

    // local CPU by default; `SP1_PROVER` can switch mock/network/etc.
    let client = ProverClient::from_env().await;

    // Run guest logic only (no Groth16). Panics in the guest surface here.
    let (mut public_values, _report) = client
        .execute(ELF, stdin)
        .await
        .expect("guest execution failed");

    // Journal: whatever the guest `commit`ted (`PublicOutputs`).
    let public: PublicOutputs = public_values.read();

    assert_eq!(public.merkle_root, inputs.expected_root);
    assert_eq!(public.requested_swap_amount, inputs.requested_swap_amount);
    assert_ne!(public.nullifier, [0u8; 32]);

    println!("execute ok — merkle_root={:?}", public.merkle_root);
}