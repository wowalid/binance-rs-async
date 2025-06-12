extern crate lazy_static;
#[macro_use]
extern crate serde;
extern crate serde_qs as qs;

use binance::rest_model::DepositQuestionnaireRequest;
use binance::rest_model::NaturalPersonPii;
use binance::rest_model::PiiName;
use binance::rest_model::StandardPii;
use binance::rest_model::TravelRuleDepositHistoryQuery;
use binance::rest_model::UaeQuestionnaire;
use chrono::Utc;
pub use util::bool_to_string;
pub use util::bool_to_string_some;

use binance::rest_model::SubAccountDepositHistoryQuery;

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
    let client = { reqwest::Client::builder().build().unwrap() };

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

    let travel_rule_query = TravelRuleDepositHistoryQuery {
        tr_id: None,
        coin: None,
        travel_rule_status: None, // Pending
        pending_questionnaire: None,
        start_time: None,
        end_time: None,
        offset: Some(0),
        limit: Some(1000),
        timestamp: Utc::now().timestamp_millis() as u64,
        ..Default::default()
    };

    for deposit_record in wallet
        .get_travel_rule_deposit_history(travel_rule_query.clone())
        .await
        .unwrap()
    {
        let deposit_id = deposit_record.tran_id.clone();

        let questionnaire = UaeQuestionnaire {
            deposit_originator: 1, // Myself
            org_type: None,
            org_name: None,
            country: None,
            city: None,
            receive_from: 1, // Private Wallet
            vasp: None,
            vasp_name: None,
        };

        let request = DepositQuestionnaireRequest {
            tran_id: deposit_id.to_string(),
            questionnaire,

            timestamp: Utc::now().timestamp_millis() as i64,
        };

        if deposit_record.require_questionnaire {
            println!("questionnaire {:?}", request);
            let response = wallet.submit_uae_deposit_questionnaire(request).await.unwrap();
            println!("Deposit ID: {:?}, requires questionnaire", response);
        }
    }
}
