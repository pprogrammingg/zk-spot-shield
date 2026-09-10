//! Host driver: default is **execute only** (fits a 16 GB laptop).
//!
//! RUST_LOG=info cargo run -p zk-circuit-host --release
//! ```
//!
//! Groth16 wrap OOMs this laptop. Day 12 is **remote** (`SP1_PROVER=network`); not free (`$PROVE`):
//!
//! ```bash
//! SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info cargo run -p zk-circuit-host --release
//! ```

use std::fs;
use std::time::Instant;

use sp1_sdk::{
    utils, Elf, HashableKey, ProveRequest, Prover, ProvingKey, ProverClient, SP1Stdin,
};

use zk_circuit_host::fixtures;
use zk_circuit_io::PublicOutputs;

const ELF: Elf = Elf::Static(include_bytes!("../../../target/elf/guest"));

#[tokio::main]
async fn main() {
    utils::setup_logger();

    let inputs = fixtures::happy_path_inputs();
    let mut stdin = SP1Stdin::new();
    stdin.write(&inputs);

    let client = ProverClient::from_env().await;

    let (mut exec_pv, report) = client
        .execute(ELF, stdin.clone())
        .await
        .expect("guest execution failed");
    let exec_public: PublicOutputs = exec_pv.read();
    assert_eq!(exec_public.merkle_root, inputs.expected_root);
    assert_eq!(exec_public.requested_swap_amount, inputs.requested_swap_amount);
    assert_ne!(exec_public.nullifier, [0u8; 32]);
    println!(
        "execute ok — cycles={} merkle_root={:?}",
        report.total_instruction_count(),
        exec_public.merkle_root
    );

    if std::env::var("SP1_GROTH16").ok().as_deref() != Some("1") {
        println!("skip groth16 (set SP1_GROTH16=1 on a machine that can wrap)");
        return;
    }

    let pk = client.setup(ELF).await.expect("setup failed");
    let vk = pk.verifying_key();
    println!("vk.bytes32() = {}", vk.bytes32());

    let t0 = Instant::now();
    let proof = client
        .prove(&pk, stdin)
        .groth16()
        .await
        .expect("groth16 prove failed");
    let prove_secs = t0.elapsed().as_secs_f64();

    client
        .verify(&proof, vk, None)
        .expect("local groth16 verify failed");

    let mut pv = proof.public_values.clone();
    let public: PublicOutputs = pv.read();
    assert_eq!(public.merkle_root, inputs.expected_root);
    assert_eq!(public.requested_swap_amount, inputs.requested_swap_amount);
    assert_ne!(public.nullifier, [0u8; 32]);

    let groth16_bytes = proof.bytes();
    println!(
        "groth16 ok — proof.bytes().len()={} prove_secs={:.1} merkle_root={:?}",
        groth16_bytes.len(),
        prove_secs,
        public.merkle_root
    );
    assert!(
        !groth16_bytes.is_empty(),
        "empty proof.bytes(): missing .groth16()?"
    );

    fs::create_dir_all("sp1-artifacts").expect("mkdir sp1-artifacts");
    proof
        .save("sp1-artifacts/proof.bin")
        .expect("save proof.bin");
    fs::write("sp1-artifacts/journal.bin", proof.public_values.as_slice())
        .expect("write journal");
    fs::write("sp1-artifacts/groth16.bin", &groth16_bytes).expect("write groth16");
    fs::write("sp1-artifacts/vkey.bytes32.txt", vk.bytes32()).expect("write vk");
    println!("wrote sp1-artifacts/proof.bin journal.bin groth16.bin vkey.bytes32.txt");
}
