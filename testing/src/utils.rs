use std::io::{self, Write};
use std::process::Command;

/// general helper functions
pub mod utils {
    use super::*;    
    /// returns file as Vec<u8>
    /// if file does not exist calls build method then returns once built
    /// panics if read attempt fails once build method has been called
    pub fn get_wasm(path_to_wasm: &str) -> Vec<u8> {
        match std::fs::read(path_to_wasm) {
            Err(_) => {
                build_contract();  
                match std::fs::read(path_to_wasm) {
                    Ok(wasm) => {
                        println!("Contract was built, returning wasm");
                        wasm
                    },
                    Err(_) => panic!("could not retrive wasm")
                }
            },
            Ok(wasm) => {
                println!("Contract is already built, returning wasm");
                wasm
            }
        }
    }

    /// iterates through a commands list that builds the wasm
    /// then copies to contracts/res directory
    pub fn build_contract() {
        let commands = vec![
            ("echo", "    Building Contract .wasm file"),
            ("echo", ""),
            ("cargo", "build --target wasm32-unknown-unknown --release"),
            ("cp", "target/wasm32-unknown-unknown/release/decentralised_content_subscription_near.wasm contracts/res/decentralised_content_subscription_near.wasm"),
            ("echo", "")
        ];
        for tup in commands.iter() {
            let output = Command::new(tup.0)
                        .args(tup.1.split(" "), )
                        .output()
                        .expect("failed to execute process");
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
        }
    }

    /// deletes .near-credentials directory to reduce clutter
    pub fn remove_near_credentials() {
        let output = Command::new("rm")
                    .arg("-rf")
                    .arg(".near-credentials/")
                    .output()
                    .expect("failed to execute process");
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
    }
}