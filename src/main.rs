pub mod error;

use std::process::Command;
use std::str::FromStr;
use std::time::Duration;
use std::{env, fs, thread};

use axum::routing::get;
use axum::Router;
use bdk_wallet::bitcoin::bip32::{DerivationPath, Xpub};
use bdk_wallet::bitcoin::key::Secp256k1;
use bdk_wallet::bitcoin::{Address, CompressedPublicKey};
use bdk_wallet::serde_json::{self, Value};
use bdk_wallet::{
    bitcoin::{bip32::Xpriv, Network},
    keys::{bip39::WordCount, GeneratableKey, GeneratedKey},
    miniscript::{self},
};
use bip39::{Language, Mnemonic};
use dotenv::dotenv;
use error::LavaErrors;
use regex::Regex;
use reqwest::Client;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{keypair_from_seed, Keypair},
    signer::Signer,
};

fn create_a_new_mnemonic() -> Result<String, LavaErrors> {
    let mnemonic: GeneratedKey<Mnemonic, miniscript::Tap> =
        Mnemonic::generate((WordCount::Words12, Language::English)).unwrap();
    println!("Mnemonic: {}", mnemonic.to_string());
    Ok(mnemonic.to_string())
}

fn create_bitcoin_address(seed: &[u8; 64], path: &str) -> Result<Address, LavaErrors> {
    let master_key = Xpriv::new_master(Network::Testnet, seed).unwrap();

    let derivation_path = DerivationPath::from_str(path).unwrap();
    let child_key = master_key
        .derive_priv(&Secp256k1::new(), &derivation_path)
        .unwrap();

    let secp = Secp256k1::new();
    let public_key = Xpub::from_priv(&secp, &child_key);

    println!(
        "Public key: {}",
        CompressedPublicKey::from_str(public_key.public_key.to_string().as_str()).unwrap()
    );

    let native_segwit_address = Address::p2wpkh(
        &CompressedPublicKey::from_str(public_key.public_key.to_string().as_str()).unwrap(),
        Network::Testnet,
    );

    Ok(native_segwit_address)
}

fn generate_a_solana_pubkey(mnemonic: String) -> Result<Pubkey, LavaErrors> {
    let mnemonic = Mnemonic::parse(mnemonic).unwrap();

    let seed_bytes = mnemonic.to_seed(""); // Empty passphrase

    // Generate Solana address
    let solana_path = "m/44'/501'/0'/0'";
    let solana_keypair = derive_solana_keypair(&seed_bytes, solana_path).unwrap();
    let solana_address = solana_keypair.pubkey();

    Ok(solana_address)
}

fn derive_solana_keypair(seed: &[u8], path: &str) -> Result<Keypair, LavaErrors> {
    use ed25519_dalek_bip32::{DerivationPath, ExtendedSecretKey};

    let derivation_path = DerivationPath::from_str(path).unwrap();
    let extended_key = ExtendedSecretKey::from_seed(seed).unwrap();
    let derived_key = extended_key.derive(&derivation_path).unwrap();

    let mut seed_bytes = [0u8; 32];
    seed_bytes.copy_from_slice(&derived_key.secret_key.to_bytes()[..32]);

    let keypair = keypair_from_seed(&seed_bytes).unwrap();

    Ok(keypair)
}

async fn test() {
    // Setup up environment variable
    dotenv().ok();
    let cli_path = env::var("CLI_PATH").expect("API_KEY not found in .env");

    let mnemonic = create_a_new_mnemonic().unwrap();
    let solana_address = generate_a_solana_pubkey(mnemonic.clone()).unwrap();
    let bitcoin_address = create_bitcoin_address(
        &Mnemonic::parse(mnemonic.clone()).unwrap().to_seed(""),
        "m/84'/1'/0'/0/0",
    )
    .unwrap();
    println!("Bitcoin Address: {}", bitcoin_address);
    println!("Solana Address: {}", solana_address);

    let client = reqwest::Client::new();
    println!(
        "Response from UPDATE BTC BALANCE: {:?}",
        update_btc_balance(&client, bitcoin_address).await.unwrap()
    );
    println!(
        "Response from UPDATE SOL BALANCE: {:?}",
        update_sol_balance(&client, solana_address).await.unwrap()
    );

    println!("Waiting for 20 seconds...");

    thread::sleep(Duration::from_secs(20));

    let cli_dir = format!("{}./loans-borrower-cli", cli_path);
    println!("CLI DIR: {}", cli_dir);

    let arg = format!(
        r#"MNEMONIC="{}" {} --testnet --disable-backup-contracts borrow init --loan-capital-asset solana-lava-usd --ltv-ratio-bp 5000 --loan-duration-days 4 --loan-amount 2 --finalize"#,
        mnemonic, cli_dir
    );

    println!("CLI DIR: {:?}", cli_dir);
    println!("ARG: {:?}", arg);

    let output = Command::new("sh").arg("-c").arg(arg).output().unwrap();

    println!("Output: {:?}", output);

    let formatted_output = format!("{:?}", output);
    let re = Regex::new(r"New contract ID: ([a-f0-9]{64})").unwrap();
    let mut c_id = None;
    if let Some(caps) = re.captures(&formatted_output) {
        if let Some(contract_id) = caps.get(1) {
            c_id = Some(contract_id.as_str());
            println!("Extracted Contract ID: {}", contract_id.as_str());
        }
    } else {
        println!("No contract ID found in the log.");
    }

    println!("Waiting for 10 seconds...");
    thread::sleep(Duration::from_secs(10));

    // Repay the loan
    let repay_arg = format!(
        r#"MNEMONIC="{}" {} --testnet --disable-backup-contracts borrow repay --contract-id {}"#,
        mnemonic,
        cli_dir,
        c_id.unwrap()
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(repay_arg)
        .output()
        .unwrap();

    println!("Repay Output: {:?}", output);

    println!("Waiting for 10 seconds...");
    thread::sleep(Duration::from_secs(10));

    // Detect loan repayment
    let loan_repay_arg = format!(
        r#"MNEMONIC="{}" {} --testnet --disable-backup-contracts get-contract --contract-id {} --verbose --output-file {}.json"#,
        mnemonic,
        cli_dir,
        c_id.unwrap(),
        c_id.unwrap(),
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(loan_repay_arg)
        .output()
        .unwrap();

    println!("Loan Repay Output: {:?}", output);

    let file_name = format!("{}.json", c_id.unwrap());
    let contents = fs::read_to_string(file_name).expect("Failed to read file");
    let json: Value = serde_json::from_str(&contents).expect("Failed to parse JSON");

    if json.get("Closed").is_some() {
        println!("Key Closed exists!")
    }

    if json
        .get("Closing")
        .and_then(|o| o.get("outcome"))
        .and_then(|o| o.get("repayment"))
        .and_then(|r| r.get("collateral_repayment_txid"))
        .is_some()
    {
        println!("Key 'outcome.repayment.collateral_repayment_txid' exists!");
    } else {
        println!("Key does not exist!");
    }

    println!("{}", json);
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/test", get(test));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on port: 3000");
    axum::serve(listener, app).await.unwrap();
}

async fn update_btc_balance(client: &Client, address: Address) -> Result<String, LavaErrors> {
    let body = format!(r#"{{ "address": "{}", "sats": 100000 }}"#, address);
    let res = client
        .post("https://faucet.testnet.lava.xyz/mint-mutinynet")
        .header("Content-Type", "application/json")
        .body(body);
    let response = res.send().await.unwrap().text().await.unwrap();
    Ok(response)
}

async fn update_sol_balance(client: &Client, pubkey: Pubkey) -> Result<String, LavaErrors> {
    let body = format!(r#"{{ "pubkey": "{}" }}"#, pubkey);
    let res = client
        .post("https://faucet.testnet.lava.xyz/transfer-lava-usd")
        .header("Content-Type", "application/json")
        .body(body);
    let response = res.send().await.unwrap().text().await.unwrap();
    Ok(response)
}
