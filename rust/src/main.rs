#![allow(unused)]
use bitcoin::address::NetworkUnchecked;
use bitcoin::hex::DisplayHex;
use bitcoin::Address;
use bitcoincore_rpc::bitcoin::{Amount, Network};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;

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

fn check_wallets() {
    let rpc = Client::new(
        // RPC_URL,
        &format!("{}/wallet/{}", RPC_URL, "Trader"),
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )
    .unwrap();

    let received = rpc
        .list_received_by_address(None, Some(1), None, None)
        .unwrap();
    for item in received {
        println!("Received Address: {:?}", item.address);
    }

    let rpc = Client::new(
        // RPC_URL,
        &format!("{}/wallet/{}", RPC_URL, "Miner"),
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )
    .unwrap();

    let received = rpc
        .list_received_by_address(None, Some(1), None, None)
        .unwrap();
    for item in received {
        println!("Received Address: {:?}", item.address);
    }
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
        .get_new_address(None, None)
        .unwrap()
        .require_network(Network::Regtest)
        .unwrap();

    println!("miner_address {} ", miner_address);

    // miner.generate_to_address(103, &miner_address)?;

    println!("Miner bal {} ", miner.get_balance(None, None)?);

    // Load Trader wallet and generate a new address
    let trader_address = trader
        .get_new_address(None, None)
        .unwrap()
        .require_network(Network::Regtest)
        .unwrap();

    println!("trader_address {} ", trader_address);

    // Send 20 BTC from Miner to Trader
    let tx_id = send(&miner, &trader_address.to_string(), 20.0)?;

    // Check transaction in mempool

    println!("trader bal {} ", trader.get_balance(None, None)?);

    // Mine 1 block to confirm the transaction

    // Extract all required transaction details

    // Write the data to ../out.txt in the specified format given in readme.md

    Ok(())
}
