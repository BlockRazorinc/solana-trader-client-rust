use bincode;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_system_interface::instruction::transfer;
use solana_trader_client_rust::get_http_tip_account;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Deserialize, Debug)]
struct SendResponse {
    signature: String,
}

#[derive(Deserialize, Debug)]
struct HealthResponse {
    result: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration values
    // Example endpoints only. Always use the current endpoints published on the BlockRazor website.
    let http_endpoint = "http://frankfurt.solana.blockrazor.xyz:443/v2/sendBinaryTransaction";
    let health_endpoint = "http://frankfurt.solana.blockrazor.xyz:443/health";
    let tip_accounts_endpoint = "http://frankfurt.solana.blockrazor.xyz:443/getTipAccounts";
    let mainnetrpc = "";
    // replace your authKey
    let authkey = "";
    // relace your private key
    let privatekey = "";
    // relace your target public key
    let publickey = "";
    // transaction amount
    let amount: u64 = 200_000;
    // tip amount
    let tipamount: u64 = 100_000;
    // safe window
    let safe_window: u32 = 5;
    // revert protection
    let revert_protection = false;
    // send mode
    let mode = "sandwichMitigation";

    // Create a shared HTTP client with keep-alive support
    let client = reqwest::Client::new();

    // Perform initial health check to warm up the connection
    ping_health(&client, health_endpoint, authkey).await?;

    // Start background health pinger to keep the connection alive
    let health_endpoint_clone = health_endpoint.to_string();
    let authkey_clone = authkey.to_string();
    let client_clone = client.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = ping_health(&client_clone, &health_endpoint_clone, &authkey_clone).await
            {
                eprintln!("Health check failed: {:?}", e);
            }
            sleep(Duration::from_secs(30)).await;
        }
    });

    // Send Solana transaction
    send_transaction(
        &client,
        mainnetrpc,
        authkey,
        privatekey,
        publickey,
        tipamount,
        amount,
        mode,
        safe_window,
        revert_protection,
        &http_endpoint,
        tip_accounts_endpoint,
    )
    .await?;

    Ok(())
}

/// Perform POST /health to keep the HTTP connection alive.
async fn ping_health(
    client: &reqwest::Client,
    health_endpoint: &str,
    authkey: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_str(authkey)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let res = client
        .post(health_endpoint)
        .headers(headers)
        .json(&serde_json::json!({}))
        .send()
        .await?;

    let body = res.text().await?;
    let parsed: Result<HealthResponse, _> = serde_json::from_str(&body);
    if let Ok(hr) = parsed {
        println!("Health result: {}", hr.result);
    } else {
        return Err("Failed to parse health response".into());
    }
    Ok(())
}

/// Build and send a binary transaction via HTTP POST
async fn send_transaction(
    client: &reqwest::Client,
    mainnetrpc: &str,
    authkey: &str,
    privatekey: &str,
    publickey: &str,
    tipamount: u64,
    amount: u64,
    mode: &str,
    safe_window: u32,
    revert_protection: bool,
    http_endpoint: &str,
    tip_accounts_endpoint: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let from = Keypair::from_base58_string(privatekey);
    let receiver = Pubkey::from_str(publickey)?;
    let tip = get_http_tip_account(client, tip_accounts_endpoint).await?;

    let rpcurl = String::from(mainnetrpc);
    let connection = RpcClient::new_with_commitment(rpcurl, CommitmentConfig::confirmed());
    let recent_blockhash = connection
        .get_latest_blockhash()
        .expect("Failed to get recent blockhash.");

    let ix_tip = transfer(&from.pubkey(), &tip, tipamount);
    let ix_main = transfer(&from.pubkey(), &receiver, amount);

    let tx = Transaction::new_signed_with_payer(
        &[ix_tip, ix_main],
        Some(&from.pubkey()),
        &[&from],
        recent_blockhash,
    );

    let serialized = bincode::serialize(&tx)?;
    let res = client
        .post(http_endpoint)
        .query(&[
            ("auth", authkey.to_string()),
            ("mode", mode.to_string()),
            ("safeWindow", safe_window.to_string()),
            ("revertProtection", revert_protection.to_string()),
        ])
        .header(CONTENT_TYPE, "application/octet-stream")
        .body(serialized)
        .send()
        .await?;

    let text = res.text().await?;
    let parsed: Result<SendResponse, _> = serde_json::from_str(&text);
    match parsed {
        Ok(r) => {
            println!("Transaction Signature: {}", r.signature); // use field
        }
        Err(_) => println!("RAW RESPONSE: {}", text),
    }

    Ok(())
}
