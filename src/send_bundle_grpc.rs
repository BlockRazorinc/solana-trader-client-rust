use base64::{engine::general_purpose, Engine as _};
use bincode;
use server::server_client::ServerClient;
use server::{GetTipAccountsRequest, HealthRequest, SendBundleRequest};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_system_interface::instruction::transfer;
use solana_trader_client_rust::random_tip_account;
use std::error::Error;
use std::time::Duration;
use tokio::time::interval;
use tonic::metadata::AsciiMetadataValue;
use tonic::transport::Channel;

pub mod server {
    tonic::include_proto!("serverpb");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This example endpoint may change. Always use the current endpoint published on the BlockRazor website.
    let blzendpoint = "http://frankfurt.solana-grpc.blockrazor.xyz:80";
    // replace your solana rpc endpoint
    let mainnetrpc = "";
    // replace your authKey
    let authkey = "";
    // relace your private key
    let privatekey = "";
    // tip amount
    let tipamount = 100_000;

    let channel = Channel::from_shared(blzendpoint.to_string())
        .map_err(|e| Box::<dyn Error>::from(format!("Invalid URI: {}", e)))?
        .connect()
        .await
        .map_err(|e| Box::<dyn Error>::from(format!("Connection error: {}", e)))?;

    let apikeyvalue = AsciiMetadataValue::try_from(authkey)
        .map_err(|e| Box::<dyn Error>::from(format!("Invalid API key format: {}", e)))?;

    let mut client = ServerClient::new(channel.clone());
    let tip_accounts = client
        .get_tip_accounts(tonic::Request::new(GetTipAccountsRequest {}))
        .await?
        .into_inner()
        .tip_accounts;
    let tip_account = random_tip_account(&tip_accounts)?;
    let mut request = tonic::Request::new(HealthRequest {});
    request.metadata_mut().insert("apikey", apikeyvalue.clone());
    let response = client.get_health(request).await?;
    println!("health check response: {:?}", response.into_inner().status);

    // Start a background goroutine to periodically send /health requests
    // For low-frequency users, this keeps the grpc connection alive (warm)
    let mut health_client = ServerClient::new(channel);
    let health_apikey = apikeyvalue.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            let mut request = tonic::Request::new(HealthRequest {});
            request
                .metadata_mut()
                .insert("apikey", health_apikey.clone());
            match health_client.get_health(request).await {
                Ok(response) => {
                    println!(
                        "[Health Check] Response: {:?}",
                        response.into_inner().status
                    );
                }
                Err(e) => {
                    eprintln!("[Health Check] Failed: {}", e);
                }
            }
        }
    });

    let from = Keypair::from_base58_string(privatekey);
    let frompubkey = Signer::pubkey(&from);
    let topubkey = tip_account;

    let rpcurl = String::from(mainnetrpc);
    let connection = RpcClient::new_with_commitment(rpcurl, CommitmentConfig::confirmed());

    let ix = transfer(&frompubkey, &topubkey, tipamount);
    let recent_blockhash = connection
        .get_latest_blockhash()
        .expect("Failed to get recent blockhash.");
    let tx =
        Transaction::new_signed_with_payer(&[ix], Some(&frompubkey), &[&from], recent_blockhash);

    // tx base64
    let serialized = bincode::serialize(&tx)?;
    let base64_encoded = general_purpose::STANDARD.encode(serialized);
    let mut bundle_request: tonic::Request<SendBundleRequest> =
        tonic::Request::new(SendBundleRequest {
            transactions: vec![base64_encoded],
        });
    bundle_request
        .metadata_mut()
        .insert("apikey", apikeyvalue.clone());
    let response = client.send_bundle(bundle_request).await?;
    println!("SEND BUNDLE RESPONSE={:?}", response);

    Ok(())
}
