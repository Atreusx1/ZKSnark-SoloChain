use std::process::Command;
use std::path::Path;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=src/circuits");

    // Get the output directory
    let out_dir = env::var("OUT_DIR").unwrap();
    let build_dir = Path::new(&out_dir).join("zksnark");
    std::fs::create_dir_all(&build_dir).unwrap();

    // Check circom installation
    let circom_version = Command::new("circom")
        .arg("--version")
        .output()
        .expect("Failed to check circom version. Is circom installed?");
    
    println!("Using circom version: {}", String::from_utf8_lossy(&circom_version.stdout));

    // Compile circuit
    let circuit_output = Command::new("circom")
        .args(&[
            "src/circuits/enhanced_transaction.circom",
            "--r1cs",
            "--wasm",
            "--sym",
            "--prime",
            "bn128",
            "-o",
            build_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to compile circuit");

    if !circuit_output.success() {
        panic!("Circuit compilation failed");
    }

    // Generate powers of tau
    let ptau_file = build_dir.join("pot12_0000.ptau");
    if !ptau_file.exists() {
        Command::new("snarkjs")
            .args(&[
                "powersoftau",
                "new",
                "bn128",
                "12",
                ptau_file.to_str().unwrap(),
                "-v"
            ])
            .status()
            .expect("Failed to generate initial powers of tau");

        Command::new("snarkjs")
            .args(&[
                "powersoftau",
                "contribute",
                ptau_file.to_str().unwrap(),
                build_dir.join("pot12_final.ptau").to_str().unwrap(),
                "--name=First contribution",
                "-v"
            ])
            .status()
            .expect("Failed to contribute to powers of tau");
    }

    // Setup for groth16
    Command::new("snarkjs")
        .args(&[
            "groth16",
            "setup",
            build_dir.join("enhanced_transaction.r1cs").to_str().unwrap(),
            build_dir.join("pot12_final.ptau").to_str().unwrap(),
            build_dir.join("circuit_final.zkey").to_str().unwrap(),
        ])
        .status()
        .expect("Failed to generate zkey");

    // Export verification key
    Command::new("snarkjs")
        .args(&[
            "zkey",
            "export",
            "verificationkey",
            build_dir.join("circuit_final.zkey").to_str().unwrap(),
            build_dir.join("verification_key.json").to_str().unwrap(),
        ])
        .status()
        .expect("Failed to export verification key");

    // Copy verification key to source directory
    std::fs::copy(
        build_dir.join("verification_key.json"),
        "src/verification_key.json",
    ).expect("Failed to copy verification key");
}