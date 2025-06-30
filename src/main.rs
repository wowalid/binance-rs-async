extern crate lazy_static;
#[macro_use]
extern crate serde;
extern crate serde_qs as qs;

use binance::rest_model::DepositQuestionnaireRequest;
use binance::rest_model::NaturalPersonPii;
use binance::rest_model::PiiName;
use binance::rest_model::StandardPii;
use binance::rest_model::SubAccountDepositHistoryQuery;
use binance::rest_model::TravelRuleDepositHistoryQuery;
use binance::rest_model::TravelRuleWithdrawQuery;
use binance::rest_model::UaeQuestionnaire;
use binance::rest_model::WithdrawQuestionnaire;
use chrono::Utc;
pub use util::bool_to_string;
pub use util::bool_to_string_some;

pub mod client;
pub mod errors;
pub mod util;

pub mod account;
pub mod api;
pub mod config;
pub mod futures;
pub mod general;
pub mod margin;
pub mod market;
pub mod rest_model;
pub mod savings;
pub mod userstream;
pub mod wallet;
pub mod websockets;
pub mod ws_model;
#[tokio::main]
async fn main() {
    let client = { reqwest::Client::builder().proxy(proxy).build().unwrap() };
    let master_api_key = "";
    let master_api_secret = "";

    let mut default_config = binance::config::Config::default();
    default_config.recv_window = 15000;
    let mut wallet: binance::wallet::Wallet = binance::api::Binance::new_with_config(
        Some(master_api_key.to_string()),
        Some(master_api_secret.to_string()),
        &default_config,
    );

    wallet.client.inner = client.clone();

    let withdraw_questionnaire = WithdrawQuestionnaire {
        is_address_owner: 1,
        bnf_type: None,
        bnf_name: None,
        country: None,
        city: None,
        send_to: 1,
        vasp: None,
        vasp_name: None,
    };

    // Serialize UaeQuestionnaire to JSON string for questionnaire parameter
    let questionnaire_json = serde_json::to_string(&withdraw_questionnaire).unwrap();

    let withdraw_query = TravelRuleWithdrawQuery {
        coin: "USDT".to_string(),
        address: "0x8e5686ffbab85fe7d181967b1e2ab4ef3491d8b8".to_string(),
        amount: 20.0,
        questionnaire: questionnaire_json, // JSON string or null
        withdraw_order_id: None,
        network: Some("ETH".to_string()),
        address_tag: None,
        transaction_fee_flag: None,
        name: None,
        wallet_type: Some(1),
    };

    match wallet.local_withdraw(withdraw_query).await {
        Ok(response) => {
            println!("Withdrawal successful: {:?}", response);
        }
        Err(e) => {
            eprintln!("Error during withdrawal: {}", e);
        }
    };
}
