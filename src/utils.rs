use std::io::Read;
use std::{collections::HashMap, env, path::PathBuf};

use solana_sdk::{commitment_config::CommitmentConfig, signature::Signature};
use solana_transaction_status::TransactionStatus;
use cached::proc_macro::cached;
use ore_api::{
    consts::{
        CONFIG_ADDRESS, MINT_ADDRESS, PROOF, TOKEN_DECIMALS, TOKEN_DECIMALS_V1, TREASURY_ADDRESS,
    },
    state::{Config, Proof, Treasury},
};
use ore_utils::AccountDeserialize;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::{pubkey::Pubkey, sysvar};
use solana_sdk::clock::Clock;
use spl_associated_token_account::get_associated_token_address;

pub async fn _get_treasury(client: &RpcClient) -> Treasury {
    let data = client
        .get_account_data(&TREASURY_ADDRESS)
        .await
        .expect("Failed to get treasury account");
    *Treasury::try_from_bytes(&data).expect("Failed to parse treasury account")
}

pub async fn get_config(client: &RpcClient) -> Config {
    let data = client
        .get_account_data(&CONFIG_ADDRESS)
        .await
        .expect("Failed to get config account");
    *Config::try_from_bytes(&data).expect("Failed to parse config account")
}

pub async fn get_proof_with_authority(client: &RpcClient, authority: Pubkey) -> Proof {
    let proof_address = proof_pubkey(authority);
    get_proof(client, proof_address).await
}

pub async fn get_proof(client: &RpcClient, address: Pubkey) -> Proof {
    let data = client
        .get_account_data(&address)
        .await
        .expect("Failed to get miner account");
    *Proof::try_from_bytes(&data).expect("Failed to parse miner account")
}

pub async fn get_clock(client: &RpcClient) -> Clock {
    let data = client
        .get_account_data(&sysvar::clock::ID)
        .await
        .expect("Failed to get miner account");
    bincode::deserialize::<Clock>(&data).expect("Failed to deserialize clock")
}

pub fn amount_u64_to_string(amount: u64) -> String {
    amount_u64_to_f64(amount).to_string()
}

pub fn amount_u64_to_f64(amount: u64) -> f64 {
    (amount as f64) / 10f64.powf(TOKEN_DECIMALS as f64)
}

pub fn amount_f64_to_u64(amount: f64) -> u64 {
    (amount * 10f64.powf(TOKEN_DECIMALS as f64)) as u64
}

pub fn amount_f64_to_u64_v1(amount: f64) -> u64 {
    (amount * 10f64.powf(TOKEN_DECIMALS_V1 as f64)) as u64
}

pub fn ask_confirm(question: &str) -> bool {
    println!("{}", question);
    loop {
        let mut input = [0];
        let _ = std::io::stdin().read(&mut input);
        match input[0] as char {
            'y' | 'Y' => return true,
            'n' | 'N' => return false,
            _ => println!("y/n only please."),
        }
    }
}

#[cached]
pub fn proof_pubkey(authority: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[PROOF, authority.as_ref()], &ore_api::ID).0
}

#[cached]
pub fn treasury_tokens_pubkey() -> Pubkey {
    get_associated_token_address(&TREASURY_ADDRESS, &MINT_ADDRESS)
}


#[macro_export]
macro_rules! format_duration {
    ($d: expr) => {
        format_args!("{:.1}s", $d.as_secs_f64())
    };
}

#[macro_export]
macro_rules! format_reward {
    ($r: expr) => {
        format_args!("{:.}", utils::ore_ui_amount($r))
    };
}

#[macro_export]
macro_rules! wait_return {
    ($duration: expr) => {{
        tokio::time::sleep(std::time::Duration::from_millis($duration)).await;
        return;
    }};

    ($duration: expr, $return: expr) => {{
        tokio::time::sleep(std::time::Duration::from_millis($duration)).await;
        return $return;
    }};
}

#[macro_export]
macro_rules! wait_continue {
    ($duration: expr) => {{
        tokio::time::sleep(std::time::Duration::from_millis($duration)).await;
        continue;
    }};
}


pub fn find_landed_txs(signatures: &[Signature], statuses: Vec<Option<TransactionStatus>>) -> Vec<Signature> {
    let landed_tx = statuses
        .into_iter()
        .zip(signatures.iter())
        .filter_map(|(status, sig)| {
            if status?.satisfies_commitment(CommitmentConfig::confirmed()) {
                Some(*sig)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    landed_tx
}
