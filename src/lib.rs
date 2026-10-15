use rand::Rng;
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use std::error::Error;
use std::str::FromStr;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TipAccountsResponse {
    tip_accounts: Vec<String>,
}

pub async fn get_http_tip_account(
    client: &reqwest::Client,
    endpoint: &str,
) -> Result<Pubkey, Box<dyn Error>> {
    let response = client
        .get(endpoint)
        .send()
        .await?
        .error_for_status()?
        .json::<TipAccountsResponse>()
        .await?;
    random_tip_account(&response.tip_accounts)
}

pub fn random_tip_account(accounts: &[String]) -> Result<Pubkey, Box<dyn Error>> {
    if accounts.is_empty() {
        return Err("getTipAccounts returned an empty account list".into());
    }
    let index = rand::thread_rng().gen_range(0..accounts.len());
    Ok(Pubkey::from_str(&accounts[index])?)
}
