//! Host driver: default is **execute only** (fits a 16 GB laptop).
//!
//! RUST_LOG=info cargo run -p zk-circuit-host --release
//! ```
//!
//! Groth16 wrap OOMs this laptop. Wrap is **remote** (root `.env`: `SP1_USE_NETWORK=1`).
//! See `further_explanations/prove_network.md`. Not free (`$PROVE`):
//!
//! ```bash
//! SP1_USE_NETWORK=1 SP1_PROVER=network SP1_GROTH16=1 RUST_LOG=info \
//!   cargo run -p zk-circuit-host --features network --release
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use sp1_sdk::{utils, Elf, HashableKey, ProveRequest, Prover, ProverClient, ProvingKey, SP1Stdin};

use zk_circuit_host::{
    fixtures, format_vkey_rust_array, parse_vkey_bytes32, print_happy_fixture_vkey,
};
use zk_circuit_io::PublicOutputs;

const ELF: Elf = Elf::Static(include_bytes!("../../../target/elf/guest"));

#[tokio::main]
async fn main() {
    // Offline print of frozen fixture vkey (no SP1 / network).
    //   cargo run -p zk-circuit-host -- --print-vkey
    if std::env::args().any(|a| a == "--print-vkey") {
        print_happy_fixture_vkey();
        return;
    }
    // Local setup of the *current* guest ELF (no Groth16 / no `$PROVE`).
    //   cargo run -p zk-circuit-host --release -- --print-live-vkey
    if std::env::args().any(|a| a == "--print-live-vkey") {
        fixtures::load_root_dotenv();
        utils::setup_logger();
        let client = ProverClient::from_env().await;
        let pk = client.setup(ELF).await.expect("setup failed");
        let vk = pk.verifying_key();
        let vk_hex = vk.bytes32();
        println!("vk.bytes32() = {vk_hex}");
        let vk_bytes = parse_vkey_bytes32(&vk_hex).expect("vk.bytes32 parse");
        println!("VKEY_HASH (Rust):\n{}", format_vkey_rust_array(&vk_bytes));
        return;
    }

    fixtures::load_root_dotenv();
    utils::setup_logger();

    let witness_path = std::env::var("WITNESS_JSON").ok();
    let custom_witness = witness_path.is_some();
    let inputs = if let Some(ref p) = witness_path {
        println!("witness {}", p);
        fixtures::inputs_from_witness_json(Path::new(p))
    } else {
        fixtures::happy_path_inputs()
    };
    println!(
        "public mint={} amount={} root={}",
        fixtures::hex32(&inputs.asset_id_mint),
        inputs.requested_swap_amount,
        fixtures::hex32(&inputs.expected_root)
    );
    let mut stdin = SP1Stdin::new();
    stdin.write(&inputs);

    let client = ProverClient::from_env().await;

    let (mut exec_pv, report) = client
        .execute(ELF, stdin.clone())
        .await
        .expect("guest execution failed");
    let exec_public: PublicOutputs = exec_pv.read();
    assert_eq!(exec_public.merkle_root, inputs.expected_root);
    assert_eq!(
        exec_public.requested_swap_amount,
        inputs.requested_swap_amount
    );
    assert_ne!(exec_public.nullifier, [0u8; 32]);
    println!(
        "execute ok — cycles={} merkle_root={}",
        report.total_instruction_count(),
        fixtures::hex32(&exec_public.merkle_root)
    );

    let want_groth16 = std::env::var("SP1_GROTH16").ok().as_deref() == Some("1");
    if !fixtures::use_network() || !want_groth16 {
        println!(
            "skip groth16 (set SP1_USE_NETWORK=1 and SP1_GROTH16=1 in root .env, then cargo run --features network --release)"
        );
        println!("offline vkey: cargo run -p zk-circuit-host -- --print-vkey");
        return;
    }

    let pk = client.setup(ELF).await.expect("setup failed");
    let vk = pk.verifying_key();
    let vk_hex = vk.bytes32();
    println!("vk.bytes32() = {vk_hex}");
    let vk_bytes = parse_vkey_bytes32(&vk_hex).expect("vk.bytes32 parse");
    println!("VKEY_HASH (Rust):\n{}", format_vkey_rust_array(&vk_bytes));

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

    let artifact_dir: PathBuf = std::env::var("ARTIFACT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            if custom_witness {
                PathBuf::from("sp1-artifacts/devnet-settle")
            } else {
                PathBuf::from("sp1-artifacts")
            }
        });
    fs::create_dir_all(&artifact_dir).expect("mkdir artifacts");
    proof
        .save(artifact_dir.join("proof.bin"))
        .expect("save proof.bin");
    fs::write(
        artifact_dir.join("journal.bin"),
        proof.public_values.as_slice(),
    )
    .expect("write journal");
    fs::write(artifact_dir.join("groth16.bin"), &groth16_bytes).expect("write groth16");
    fs::write(artifact_dir.join("vkey.bytes32.txt"), vk.bytes32()).expect("write vk");
    println!(
        "wrote {}/{{proof,journal,groth16,vkey}}",
        artifact_dir.display()
    );

    if !custom_witness {
        let fixture = fixtures::happy_fixture_dir();
        fs::create_dir_all(&fixture).expect("mkdir fixtures/happy");
        fs::write(fixture.join("groth16.bin"), &groth16_bytes).expect("write fixture groth16");
        fs::write(fixture.join("journal.bin"), proof.public_values.as_slice())
            .expect("write fixture journal");
        fs::write(fixture.join("vkey.bytes32.txt"), vk.bytes32()).expect("write fixture vkey");
        println!(
            "copied public happy-path bytes to {} (tests; not a private key)",
            fixture.display()
        );
    }
}
