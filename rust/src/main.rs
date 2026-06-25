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
        if let Err(_) = rpc.load_wallet(wallet_name) {
            // If loading fails, create it fresh
            rpc.create_wallet(wallet_name, Some(false), Some(false), None, None)?;
        }
    }
    Ok(Client::new(
        &format!("{}/wallet/{}", RPC_URL, wallet_name),
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?)
}

fn main() -> bitcoincore_rpc::Result<()> {
    // check_wallets();
    // return Ok(());

    // Connect to Bitcoin Core RPC
    let rpc = Client::new(
        RPC_URL,
        // &format!("{}/wallet/{}", RPC_URL, "Trader"),
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    // let received = rpc
    //     .list_received_by_address(None, Some(1), None, None)
    //     .unwrap();
    // for item in received {
    //     println!("Received Address: {:?}", item.address);
    // }

    // Get blockchain info
    // let blockchain_info = rpc.get_blockchain_info()?;
    // println!("Blockchain Info: {:?}", blockchain_info);

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

    // miner.generate_to_address(103, &miner_address)?;

    println!("Miner bal {} ", miner.get_balance(None, None)?);

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
    let wallet_tx = miner.get_transaction(&tx_id, None);
    // println!("wallet_tx details: {:#?}", wallet_tx);

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

    let miner_input_amount = input_vout.value;
    println!("miner_input_amount {}", miner_input_amount);

    // Write the data to ../out.txt in the specified format given in readme.md

    // let mut out_file = std::fs::File::create("../out.txt")?;
    // use std::io::Write;
    // writeln!(out_file, "tx_id {}", tx_id)?;
    // writeln!(out_file, "miner_input_address {}", miner_input_address)?;
    // writeln!(out_file, "miner_input_amount {}", miner_input_amount)?;
    // writeln!(out_file, "trader_address {}", trader_address)?;

    // writeln!(out_file, "miner_address {}", miner_address)?;

    Ok(())
}
