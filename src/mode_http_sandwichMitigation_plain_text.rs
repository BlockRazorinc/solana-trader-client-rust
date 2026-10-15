use base64::{engine::general_purpose, Engine as _};
use reqwest::header::CONTENT_TYPE;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example endpoints only. Always use the current endpoints published on the BlockRazor website.
    let http_endpoint = "http://frankfurt.solana.blockrazor.xyz:443/v2/sendTransaction";
    let health_endpoint = "http://frankfurt.solana.blockrazor.xyz:443/health";
    let tip_accounts_endpoint = "http://frankfurt.solana.blockrazor.xyz:443/getTipAccounts";
    let mainnet_rpc = "";
    let auth_key = "";
    let private_key = "";
    let public_key = "";
    let client = reqwest::Client::new();

    ping_health(&client, health_endpoint, auth_key).await?;
    let health_client = client.clone();
    let health_endpoint = health_endpoint.to_string();
    let health_auth_key = auth_key.to_string();
    tokio::spawn(async move {
        loop {
            if let Err(error) =
                ping_health(&health_client, &health_endpoint, &health_auth_key).await
            {
                eprintln!("Health check failed: {error}");
            }
            sleep(Duration::from_secs(30)).await;
        }
    });

    send_transaction(
        &client,
        mainnet_rpc,
        auth_key,
        private_key,
        public_key,
        http_endpoint,
        tip_accounts_endpoint,
    )
    .await
}

async fn ping_health(
    client: &reqwest::Client,
    health_endpoint: &str,
    auth_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .post(health_endpoint)
        .header("apikey", auth_key)
        .json(&serde_json::json!({}))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

async fn send_transaction(
    client: &reqwest::Client,
    mainnet_rpc: &str,
    auth_key: &str,
    private_key: &str,
    public_key: &str,
    http_endpoint: &str,
    tip_accounts_endpoint: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let sender = Keypair::from_base58_string(private_key);
    let receiver = Pubkey::from_str(public_key)?;
    let tip_account = get_http_tip_account(client, tip_accounts_endpoint).await?;
    let blockhash =
        RpcClient::new_with_commitment(mainnet_rpc.to_string(), CommitmentConfig::confirmed())
            .get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[
            transfer(&sender.pubkey(), &tip_account, 100_000),
            transfer(&sender.pubkey(), &receiver, 200_000),
        ],
        Some(&sender.pubkey()),
        &[&sender],
        blockhash,
    );
    let encoded = general_purpose::STANDARD.encode(bincode::serialize(&transaction)?);
    let query = [
        ("auth", auth_key),
        ("mode", "sandwichMitigation"),
        ("safeWindow", "5"),
        ("revertProtection", "false"),
    ];
    let response = client
        .post(http_endpoint)
        .query(&query)
        .header(CONTENT_TYPE, "text/plain")
        .body(encoded)
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        return Err(format!(
            "send transaction failed: status={} body={}",
            status,
            text.trim()
        )
        .into());
    }
    let signature = text.trim();
    if signature.is_empty() {
        return Err("send transaction returned an empty signature".into());
    }
    println!("Transaction Signature: {}", signature);
    Ok(())
}
