    use std::process::Command;
    use std::path::Path;
    use std::env;

    fn main() {
        println!("cargo:rerun-if-changed=src/circuits");

        let out_dir = env::var("OUT_DIR").unwrap();
        let build_dir = Path::new(&out_dir).join("zksnark");
        std::fs::create_dir_all(&build_dir).unwrap();

        #[cfg(feature = "std")]
        compile_circuits(&build_dir);
    }

    #[cfg(feature = "std")]
    fn compile_circuits(build_dir: &Path) {
        // Compile circuit
        let status = Command::new("circom")
            .args(&[
                "src/circuits/enhanced_transaction.circom",
                "--r1cs",
                "--wasm",
                "--sym",
                "--c",
                &format!("--output={}", build_dir.display()),
            ])
            .status()
            .expect("Failed to compile circuit");

        assert!(status.success(), "Circuit compilation failed");

        // Setup proving system
        let pot_file_path = format!("{}/src/circuits/pot16_final.ptau", env::current_dir().unwrap().display());

        let status = Command::new("snarkjs")
            .args(&[
                "groth16",
                "setup",
                &format!("{}/enhanced_transaction.r1cs", build_dir.display()),
                &pot_file_path, // Correctly using the absolute path to pot16_final.ptau
                &format!("{}/circuit_final.zkey", build_dir.display()),
            ])
            .status()
            .expect("Failed to setup proving system");

        assert!(status.success(), "Proving system setup failed");

        // Export verification key
        let status = Command::new("snarkjs")
            .args(&[
                "zkey",
                "export",
                "verificationkey",
                &format!("{}/circuit_final.zkey", build_dir.display()),
                &format!("{}/verification_key.json", build_dir.display()),
            ])
            .status()
            .expect("Failed to export verification key");

        assert!(status.success(), "Verification key export failed");
    }
