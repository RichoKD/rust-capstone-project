#![allow(unused)]
use bitcoin::address::NetworkUnchecked;
use bitcoin::hex::DisplayHex;
use bitcoin::Address;
use bitcoincore_rpc::bitcoin::{Amount, Network, Txid};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::vec;

// Node access params
const RPC_URL: &str = "http://127.0.0.1:18443"; // Default regtest RPC port
const RPC_USER: &str = "alice";
const RPC_PASS: &str = "password";

// You can use calls not provided in RPC lib API using the generic `call` function.
// An example of using the `send` RPC call, which doesn't have exposed API.
// You can also use serde_json `Deserialize` derivation to capture the returned json result.
fn send(rpc: &Client, addr: &str, amount: f64) -> bitcoincore_rpc::Result<String> {
    let args = [
        json!([{addr : amount }]), // recipient address + amount
        json!(null),               // conf target
        json!(null),               // estimate mode
        json!(null),               // fee rate in sats/vb
        json!(null),               // Empty option object
    ];

    #[derive(Deserialize)]
    struct SendResult {
        complete: bool,
        txid: String,
    }
    let send_result = rpc.call::<SendResult>("send", &args)?;
    assert!(send_result.complete);
    Ok(send_result.txid)
}

// Load wallet if exists and create new if not
fn wallet_loader(rpc: &Client, wallet_name: &str) -> bitcoincore_rpc::Result<Client> {
    let wallet_list = rpc.list_wallets()?;
    if !wallet_list.contains(&wallet_name.to_string()) {
        // Attempt to load it first in case it exists but isn't loaded
        if rpc.load_wallet(wallet_name).is_err() {
            // If loading fails, create it fresh
            rpc.create_wallet(wallet_name, Some(false), Some(false), None, None)?;
        }
    }
    Client::new(
        &format!("{}/wallet/{}", RPC_URL, wallet_name),
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )
}

fn main() -> bitcoincore_rpc::Result<()> {
    // Connect to Bitcoin Core RPC
    let rpc = Client::new(
        RPC_URL,
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    // Create/Load the wallets, named 'Miner' and 'Trader'. Have logic to optionally create/load them if they do not exist or not loaded already.
    let miner = wallet_loader(&rpc, "Miner")?;
    let trader = wallet_loader(&rpc, "Trader")?;

    let wallets = miner.list_wallets()?;
    println!("Wallets: {:?}", wallets);

    // Generate spendable balances in the Miner wallet. How many blocks needs to be mined?
    let miner_address = miner
        .get_new_address(Some("Mining Reward"), None)
        .unwrap()
        .require_network(Network::Regtest)
        .unwrap();

    println!("miner_address {} ", miner_address);

    miner.generate_to_address(103, &miner_address)?;
    // 100 confirmations requirements is a security mechanism exist to prevent
    // duoble spending and prevent miner reward loss from blockchain forks

    println!("Miner balance {} ", miner.get_balance(None, None)?);

    // Load Trader wallet and generate a new address
    let trader_address = trader
        .get_new_address(Some("Received"), None)
        .unwrap()
        .require_network(Network::Regtest)
        .unwrap();

    println!("trader_address {} ", trader_address);

    // Send 20 BTC from Miner to Trader
    let tx_id: Txid = send(&miner, &trader_address.to_string(), 20.0)?
        .parse()
        .unwrap();

    println!("tx_id {}", tx_id);

    // Check transaction in mempool
    let tx = rpc.get_mempool_entry(&tx_id)?;
    println!("Unconfirmed transaction: {:#?}", tx);

    // Mine 1 block to confirm the transaction
    miner.generate_to_address(1, &miner_address)?;
    println!("========================================================");

    // Extract all required transaction details
    let raw_tx = miner.get_raw_transaction_info(&tx_id, None)?;
    println!("tx details: {:#?}", raw_tx.vin);
    let wallet_tx = miner.get_transaction(&tx_id, None).unwrap();
    // println!("wallet_tx details: {:#?}", wallet_tx);

    let block_hash = wallet_tx.info.blockhash.expect("tx must be confirmed");

    let block_info = rpc.get_block_info(&block_hash)?;
    let block_height = block_info.height;

    let tx_fees = wallet_tx.fee.expect("fee must be present").to_btc();

    let vin = &raw_tx.vin[0];
    let previous_txid = vin.txid.unwrap();
    let previous_vout = vin.vout.unwrap() as usize;
    let previous_raw_tx = rpc.get_raw_transaction_info(&previous_txid, None)?;
    let input_vout = &previous_raw_tx.vout[previous_vout];

    let miner_input_address = input_vout
        .script_pub_key
        .address
        .clone()
        .expect("input address")
        .assume_checked()
        .to_string();

    println!("miner_input_address {}", miner_input_address);

    let miner_input_amount = input_vout.value.to_btc();
    println!("miner_input_amount {}", miner_input_amount);

    let trader_address_string = trader_address.to_string();

    let mut trader_output_address = String::new();
    let mut trader_output_amount = 0.0f64;
    let mut miner_change_address = String::new();
    let mut miner_change_amount = 0.0f64;

    for vout in &raw_tx.vout {
        let addr = match vout.script_pub_key.address.clone() {
            Some(a) => a.assume_checked().to_string(),
            None => continue, // if invalid skip vout
        };

        if addr == trader_address_string {
            trader_output_address = addr;
            trader_output_amount = vout.value.to_btc();
        } else {
            miner_change_address = addr;
            miner_change_amount = vout.value.to_btc();
        }
    }

    // Write the data to ../out.txt in the specified format given in readme.md
    let mut out_file = File::create("../out.txt")?;
    writeln!(out_file, "{}", tx_id)?;
    writeln!(out_file, "{}", miner_input_address)?;
    writeln!(out_file, "{}", miner_input_amount)?;
    writeln!(out_file, "{}", trader_address)?;
    writeln!(out_file, "{}", trader_output_amount)?;
    writeln!(out_file, "{}", miner_change_address)?;
    writeln!(out_file, "{}", miner_change_amount)?;
    writeln!(out_file, "{}", tx_fees)?;
    writeln!(out_file, "{}", block_height)?;
    writeln!(out_file, "{}", block_hash)?;

    Ok(())
}
