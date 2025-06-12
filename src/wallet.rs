use crate::client::*;
use crate::errors::*;
use crate::rest_model::*;
use chrono::DateTime;
use chrono::{Duration, Utc};
use hex::encode as hex_encode;
use ring::hmac;
use std::collections::HashMap;
use std::ops::Sub;

static SAPI_V1_UNIVERSAL_TRANSFER: &str = "/sapi/v1/sub-account/universalTransfer";
static SAPI_V1_SYSTEM_STATUS: &str = "/sapi/v1/system/status";
static SAPI_V1_CAPITAL_CONFIG_GETALL: &str = "/sapi/v1/capital/config/getall";
static SAPI_V1_ACCOUNTSNAPSHOT: &str = "/sapi/v1/accountSnapshot";
static SAPI_V1_ACCOUNT_DISABLEFASTWITHDRAWSWITCH: &str = "/sapi/v1/account/disableFastWithdrawSwitch";
static SAPI_V1_ACCOUNT_ENABLEFASTWITHDRAWSWITCH: &str = "/sapi/v1/account/enableFastWithdrawSwitch";
static SAPI_V1_CAPITAL_WITHDRAW_APPLY: &str = "/sapi/v1/capital/withdraw/apply";
static SAPI_V1_CAPITAL_DEPOSIT_HISREC: &str = "/sapi/v1/capital/deposit/hisrec";
static SAPI_V1_CAPITAL_WITHDRAW_HISTORY: &str = "/sapi/v1/capital/withdraw/history";
static SAPI_V1_CAPITAL_DEPOSIT_ADDRESS: &str = "/sapi/v1/capital/deposit/address";
static SAPI_V1_ACCOUNT_STATUS: &str = "/sapi/v1/account/status";
static SAPI_V1_ACCOUNT_APITRADINGSTATUS: &str = "/sapi/v1/account/apiTradingStatus";
static SAPI_V1_ASSET_DRIBBLET: &str = "/sapi/v1/asset/dribblet";
static SAPI_V1_ASSET_DUSTBTC: &str = "/sapi/v1/asset/dust-btc";
static SAPI_V1_ASSET_DUST: &str = "/sapi/v1/asset/dust";
static SAPI_V1_ASSET_ASSETDIVIDEND: &str = "/sapi/v1/asset/assetDividend";
static SAPI_V1_ASSET_ASSETDETAIL: &str = "/sapi/v1/asset/assetDetail";
static SAPI_V1_ASSET_TRADEFEE: &str = "/sapi/v1/asset/tradeFee";
static SAPI_V1_ASSET_TRADEFEE_US: &str = "/sapi/v1/asset/query/trading-fee";
static SAPI_V1_ASSET_TRANSFER: &str = "/sapi/v1/asset/transfer";
static SAPI_V1_ASSET_GETFUNDINGASSET: &str = "/sapi/v1/asset/get-funding-asset";
static SAPI_V1_ASSET_APIRESTRICTIONS: &str = "/sapi/v1/account/apiRestrictions";
static SAPI_V1_ASSET_ONGOING_ORDERS: &str = "/sapi/v2/loan/flexible/ongoing/orders";
static SAPI_V1_VIP_LOAN_ONGOING_ORDERS: &str = "/sapi/v1/loan/vip/ongoing/orders";
static SAPI_V2_LOAN_FLEXIBLE_ADJUST_LTV: &str = "/sapi/v2/loan/flexible/adjust/ltv";

static DEFAULT_WALLET_HISTORY_QUERY_INTERVAL_DAYS: i64 = 90;

/// This struct acts as a gateway for all wallet endpoints.
/// Preferably use the trait [`crate::api::Binance`] to get an instance.
#[derive(Clone)]
pub struct Wallet {
    pub client: Client,
    pub recv_window: u64,
    pub binance_us_api: bool,
}

impl Wallet {
    /// Fetch system status.
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let system_status = tokio_test::block_on(wallet.system_status());
    /// assert!(system_status.is_ok(), "{:?}", system_status);
    /// ```
    pub async fn system_status(&self) -> Result<SystemStatus> {
        self.client.get_p(SAPI_V1_SYSTEM_STATUS, None).await
    }

    /// Get information of coins (available for deposit and withdraw) for user.
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let records = tokio_test::block_on(wallet.all_coin_info());
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn all_coin_info(&self) -> Result<Vec<WalletCoinInfo>> {
        self.client
            .get_signed_p(SAPI_V1_CAPITAL_CONFIG_GETALL, Option::<String>::None, self.recv_window)
            .await
    }

    /// Daily account snapshot
    /// The query time period must be less then 30 days
    /// Support query within the last one month only
    /// If startTime and endTime not sent, return records of the last 7 days by default
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let query: AccountSnapshotQuery = AccountSnapshotQuery {start_time: None, end_time: None, limit: None, account_type: AccountSnapshotType::Spot};
    /// let records = tokio_test::block_on(wallet.daily_account_snapshot(query));
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn daily_account_snapshot(&self, query: AccountSnapshotQuery) -> Result<AccountSnapshot> {
        self.client
            .get_signed_p(SAPI_V1_ACCOUNTSNAPSHOT, Some(query), self.recv_window)
            .await
    }

    /// Submit a deposit questionnaire for UAE Travel Rule compliance.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// use chrono::Utc;
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let questionnaire = UaeQuestionnaire {
    ///     deposit_originator: 1, // Myself
    ///     org_type: None,
    ///     org_name: None,
    ///     country: None,
    ///     city: None,
    ///     receive_from: 1, // Private Wallet
    ///     vasp: None,
    ///     vasp_name: None,
    /// };
    /// let beneficiary_pii = StandardPii::NaturalPerson(NaturalPersonPii {
    ///     latin_names: vec![PiiName {
    ///         first_name: "John".to_string(),
    ///         middle_name: None,
    ///         last_name: Some("Doe".to_string()),
    ///     }],
    ///     local_names: None,
    ///     nationality: Some("ae".to_string()),
    ///     residence_country: "ae".to_string(),
    ///     national_identifier: None,
    ///     national_identifier_type: None,
    ///     national_identifier_issue_country: None,
    ///     date_of_birth: Some("1980-01-01".to_string()),
    ///     place_of_birth: None,
    ///     address: None,
    /// });
    /// let request = DepositQuestionnaireRequest {
    ///     sub_account_id: "12345".to_string(),
    ///     deposit_id: "67890".to_string(),
    ///     questionnaire,
    ///     beneficiary_pii,
    ///     network: Some("ETH".to_string()),
    ///     coin: Some("ETH".to_string()),
    ///     amount: Some(1.5),
    ///     address: Some("0x1234567890abcdef".to_string()),
    ///     address_tag: None,
    ///     timestamp: Utc::now().timestamp_millis() as u64,
    ///     signature: "signature_here".to_string(), // Replace with actual signature
    /// };
    /// let response = tokio_test::block_on(wallet.submit_uae_deposit_questionnaire(request));
    /// assert!(response.is_ok(), "{:?}", response);
    /// ```
    ///
    ///
    ///
    fn generate_signature(query_string: &str, api_secret: &str) -> String {
        let mac = hmac::Key::new(hmac::HMAC_SHA256, api_secret.as_bytes());
        let signature = hex_encode(hmac::sign(&mac, query_string.as_bytes()).as_ref());
        signature
    }

    pub async fn submit_uae_deposit_questionnaire(
        &self,
        request: DepositQuestionnaireRequest,
    ) -> Result<DepositQuestionnaireResponse> {
        // Validate required questionnaire fields
        if request.questionnaire.deposit_originator == 0 || request.questionnaire.receive_from == 0 {
            return Err(Error::Msg(
                "Questionnaire must include depositOriginator and receiveFrom".to_string(),
            ));
        }

        // Serialize UaeQuestionnaire to JSON string for questionnaire parameter
        let questionnaire_json = serde_json::to_string(&request.questionnaire)
            .map_err(|e| Error::Msg(format!("Failed to serialize questionnaire: {}", e)))?;

        // Construct payload manually to avoid serialization artifacts
        let payload = vec![
            ("tranId".to_string(), request.tran_id),
            ("questionnaire".to_string(), questionnaire_json),
            ("timestamp".to_string(), request.timestamp.to_string()),
        ];
        let endpoint = "/sapi/v1/localentity/deposit/provide-info";
        let recv_window = 15000; // Match provided URL

        self.client.put_signed_p(endpoint, payload, recv_window).await
    }

    /// Disable Fast Withdraw Switch
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let records = tokio_test::block_on(wallet.disable_fast_withdraw_switch());
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    ///
    ///
    ///
    pub async fn disable_fast_withdraw_switch(&self) -> Result<()> {
        self.client
            .post_signed_p(
                SAPI_V1_ACCOUNT_DISABLEFASTWITHDRAWSWITCH,
                Option::<String>::None,
                self.recv_window,
            )
            .await
    }

    /// Enable Fast Withdraw Switch
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let records = tokio_test::block_on(wallet.enable_fast_withdraw_switch());
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn enable_fast_withdraw_switch(&self) -> Result<()> {
        self.client
            .post_signed_p(
                SAPI_V1_ACCOUNT_ENABLEFASTWITHDRAWSWITCH,
                Option::<String>::None,
                self.recv_window,
            )
            .await
    }

    /// Apply for Withdrawal
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let query: CoinWithdrawalQuery = CoinWithdrawalQuery::default();
    /// let records = tokio_test::block_on(wallet.withdraw(query));
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn withdraw(&self, query: CoinWithdrawalQuery) -> Result<WithdrawId> {
        self.client
            .post_signed_p(SAPI_V1_CAPITAL_WITHDRAW_APPLY, Some(query), self.recv_window)
            .await
    }

    pub async fn get_loans(&self) -> Result<LoanResponse> {
        self.client
            .get_signed_p(SAPI_V1_ASSET_ONGOING_ORDERS, Option::<String>::None, self.recv_window)
            .await
    }

    pub async fn get_vip_loans(&self) -> Result<VipLoanResponse> {
        self.client
            .get_signed_p(
                SAPI_V1_VIP_LOAN_ONGOING_ORDERS,
                Option::<String>::None,
                self.recv_window,
            )
            .await
    }

    /// Get sub-account deposit history.
    ///
    /// The query time period must be within the last 7 days. If `start_time` and `end_time` are not provided,
    /// it defaults to the recent 7 days.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// use chrono::Utc;
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let query = SubAccountDepositHistoryQuery {
    ///     sub_account_id: Some("12345".to_string()),
    ///     coin: Some("USDT".to_string()),
    ///     status: Some(1), // Success
    ///     start_time: None,
    ///     end_time: None,
    ///     limit: Some(500),
    ///     offset: Some(0),
    ///     recv_window: Some(5000),
    ///     timestamp: Utc::now().timestamp_millis() as u64,
    /// };
    /// let response = tokio_test::block_on(wallet.get_sub_account_deposit_history(query));
    /// assert!(response.is_ok(), "{:?}", response);
    /// ```
    pub async fn get_sub_account_deposit_history(
        &self,
        query: SubAccountDepositHistoryQuery,
    ) -> Result<Vec<SubAccountDepositRecord>> {
        self.client
            .get_signed_p("/sapi/v1/broker/subAccount/depositHist", Some(query), self.recv_window)
            .await
    }

    pub async fn get_travel_rule_deposit_history(
        &self,
        query: TravelRuleDepositHistoryQuery,
    ) -> Result<Vec<TravelRuleDepositRecord>> {
        self.client
            .get_signed_p("/sapi/v1/localentity/deposit/history", Some(query), self.recv_window)
            .await
    }
    /// Deposit History
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let query: DepositHistoryQuery = DepositHistoryQuery::default();
    /// let records = tokio_test::block_on(wallet.deposit_history(&query));
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn deposit_history(&self, query: &DepositHistoryQuery) -> Result<Vec<DepositRecord>> {
        self.client
            .get_signed_p(SAPI_V1_CAPITAL_DEPOSIT_HISREC, Some(query), self.recv_window)
            .await
    }

    /// Withdraw History starting at start_from (defaults to now), ranging total_duration (defaults to 90 days), with intervals of 90 days.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let query: DepositHistoryQuery = DepositHistoryQuery::default();
    /// let records = tokio_test::block_on(wallet.deposit_history_quick(query, None, None));
    /// assert!(records.is_ok(), "{:?}", records);
    pub async fn deposit_history_quick(
        &self,
        mut query: DepositHistoryQuery,
        start_from: Option<DateTime<Utc>>,
        total_duration: Option<Duration>,
    ) -> Result<Vec<RecordHistory<DepositRecord>>> {
        let mut result = vec![];

        let total_duration =
            total_duration.unwrap_or_else(|| Duration::days(DEFAULT_WALLET_HISTORY_QUERY_INTERVAL_DAYS));
        let interval_duration = Duration::days(DEFAULT_WALLET_HISTORY_QUERY_INTERVAL_DAYS);
        let mut current_period_end: DateTime<Utc> = start_from.unwrap_or_else(Utc::now);
        let end_at = current_period_end.sub(total_duration);
        let mut current_period_start: DateTime<Utc> = current_period_end.sub(interval_duration);

        // auto query by step:
        while current_period_end > end_at {
            // modify query duration:
            query.start_time = Some(current_period_start.timestamp_millis() as u64);
            query.end_time = Some(current_period_end.timestamp_millis() as u64);

            // eprintln!("query: {:?}", query);
            let records = self.deposit_history(&query).await?;

            if !records.is_empty() {
                let item = RecordHistory::<DepositRecord> {
                    start_at: current_period_start,
                    end_at: current_period_end,
                    records,
                };
                result.push(item);
            }

            current_period_start = current_period_start.sub(interval_duration);
            current_period_end = current_period_end.sub(interval_duration);
        }

        Ok(result)
    }

    /// Withdraw History
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let query: WithdrawalHistoryQuery = WithdrawalHistoryQuery::default();
    /// let records = tokio_test::block_on(wallet.withdraw_history(&query));
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn withdraw_history(&self, query: &WithdrawalHistoryQuery) -> Result<Vec<WithdrawalRecord>> {
        self.client
            .get_signed_p(SAPI_V1_CAPITAL_WITHDRAW_HISTORY, Some(query), self.recv_window)
            .await
    }

    /// Withdraw History starting at start_from (defaults to now), ranging total_duration (defaults to 90 days), with intervals of 90 days.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use chrono::Duration;
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let query: WithdrawalHistoryQuery = WithdrawalHistoryQuery::default();
    /// let records = tokio_test::block_on(wallet.withdraw_history_quick(query, None, Some(Duration::weeks( 52 * 5))));
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn withdraw_history_quick(
        &self,
        mut query: WithdrawalHistoryQuery,
        start_from: Option<DateTime<Utc>>,
        total_duration: Option<Duration>,
    ) -> Result<Vec<RecordHistory<WithdrawalRecord>>> {
        let mut result = vec![];

        let total_duration =
            total_duration.unwrap_or_else(|| Duration::days(DEFAULT_WALLET_HISTORY_QUERY_INTERVAL_DAYS));
        let interval_duration = Duration::days(DEFAULT_WALLET_HISTORY_QUERY_INTERVAL_DAYS);
        let mut current_period_end: DateTime<Utc> = start_from.unwrap_or_else(Utc::now);
        let end_at = current_period_end.sub(total_duration);
        let mut current_period_start: DateTime<Utc> = current_period_end.sub(interval_duration);

        // auto query by step:
        while current_period_end > end_at {
            query.start_time = Some(current_period_start.timestamp_millis() as u64);
            query.end_time = Some(current_period_end.timestamp_millis() as u64);

            let records = self.withdraw_history(&query).await?;

            if !records.is_empty() {
                let item = RecordHistory::<WithdrawalRecord> {
                    start_at: current_period_start,
                    end_at: current_period_end,
                    records,
                };
                result.push(item);
            }

            current_period_start = current_period_start.sub(interval_duration);
            current_period_end = current_period_end.sub(interval_duration);
        }

        Ok(result)
    }

    /// Deposit address
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let query: DepositAddressQuery = DepositAddressQuery::default();
    /// let records = tokio_test::block_on(wallet.deposit_address(query));
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn deposit_address(&self, query: DepositAddressQuery) -> Result<DepositAddress> {
        self.client
            .get_signed_p(SAPI_V1_CAPITAL_DEPOSIT_ADDRESS, Some(query), self.recv_window)
            .await
    }

    pub async fn flexible_loan_adjust_ltv(
        &self,
        loan_coin: String,
        collateral_coin: String,
        adjustment_amount: f64,
        direction: AdjustmentDirection,
    ) -> Result<serde_json::Value> {
        let adjust_ltv = FlexibleLoanAdjustLTV {
            loan_coin,
            collateral_coin,
            adjustment_amount,
            direction,
        };
        self.client
            .post_signed_p(SAPI_V2_LOAN_FLEXIBLE_ADJUST_LTV, adjust_ltv, self.recv_window)
            .await
    }

    /// Universal Transfer
    ///
    /// from_symbol must be sent when transfer_type are IsolatedmarginMargin and IsolatedmarginIsolatedmargin
    /// to_symbol must be sent when transfer_type are MarginIsolatedmargin and IsolatedmarginIsolatedmargin
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let records = tokio_test::block_on(wallet.universal_transfer("BTC".to_string(), 1.0, None, None, UniversalTransferType::FundingMain));
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    ///
    ///
    ///
    pub async fn universal_transfer(
        &self,
        asset: String,
        amount: f64,
        from_symbol: Option<String>,
        to_symbol: Option<String>,
        transfer_type: UniversalTransferType,
    ) -> Result<TransactionId> {
        let transfer = UniversalTransfer {
            asset,
            amount,
            from_symbol: from_symbol.map(Into::<String>::into),
            to_symbol: to_symbol.map(Into::<String>::into),
            transfer_type,
        };
        self.client
            .post_signed_p(SAPI_V1_ASSET_TRANSFER, transfer, self.recv_window)
            .await
    }

    pub async fn universal_transfer_subaccount(
        &self,
        asset: String,
        amount: f64,
        from_email: String,
        to_email: String,
        from_account_type: String,
        to_account_type: String,
    ) -> Result<serde_json::Value> {
        let withdraw_payload = UniversalTransferSubAccount {
            asset,
            amount,
            from_email,
            to_email,
            from_account_type,
            to_account_type,
        };

        let response = match self
            .client
            .post_signed_p(SAPI_V1_UNIVERSAL_TRANSFER, withdraw_payload, self.recv_window)
            .await
        {
            Ok(res) => Ok(res),
            Err(e) => {
                println!("Error: {:?}", e);
                Err(e)
            }
        };

        println!("Response: {:?}", response);

        response
    }

    /// Universal Transfer
    ///
    /// from_symbol must be sent when transfer_type are IsolatedmarginMargin and IsolatedmarginIsolatedmargin
    /// to_symbol must be sent when transfer_type are MarginIsolatedmargin and IsolatedmarginIsolatedmargin
    /// Support query within the last 6 months only
    /// If query.start_time and query.end_time not sent, return records of the last 7 days by default
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let query: UniversalTransferHistoryQuery = UniversalTransferHistoryQuery { start_time: None, end_time: None, transfer_type: UniversalTransferType::FundingMain, current: None, from_symbol: None, to_symbol: None, size: None };
    /// let records = tokio_test::block_on(wallet.universal_transfer_history(query));
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn universal_transfer_history(
        &self,
        query: UniversalTransferHistoryQuery,
    ) -> Result<RecordsQueryResult<UniversalTransferRecord>> {
        self.client
            .get_signed_p(SAPI_V1_ASSET_TRANSFER, Some(query), self.recv_window)
            .await
    }

    /// Current account status
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let records = tokio_test::block_on(wallet.account_status());
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn account_status(&self) -> Result<AccountStatus> {
        self.client
            .get_signed_p(SAPI_V1_ACCOUNT_STATUS, Option::<String>::None, self.recv_window)
            .await
    }

    /// Current api trading status
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let records = tokio_test::block_on(wallet.api_trading_status());
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn api_trading_status(&self) -> Result<ApiTradingStatus> {
        self.client
            .get_signed_p(
                SAPI_V1_ACCOUNT_APITRADINGSTATUS,
                Option::<String>::None,
                self.recv_window,
            )
            .await
    }

    /// Dust Log
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let records = tokio_test::block_on(wallet.dust_log(None, None));
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn dust_log(&self, start_time: Option<u64>, end_time: Option<u64>) -> Result<DustLog> {
        let mut query = HashMap::new();
        query.insert("start_time", start_time);
        query.insert("end_time", end_time);
        self.client
            .get_signed_p(SAPI_V1_ASSET_DRIBBLET, Some(query), self.recv_window)
            .await
    }

    /// Assets convertible to BNB
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let records = tokio_test::block_on(wallet.convertible_assets());
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn convertible_assets(&self) -> Result<ConvertibleAssets> {
        self.client
            .post_signed_p(SAPI_V1_ASSET_DUSTBTC, Option::<String>::None, self.recv_window)
            .await
    }

    /// Dust Transfer
    ///
    /// Convert dust assets to bnb
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let records = tokio_test::block_on(wallet.dust_transfer(vec!["BTC".to_string()]));
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn dust_transfer(&self, assets: Vec<String>) -> Result<DustTransfer> {
        let mut params = HashMap::new();
        params.insert("assets", assets);
        self.client
            .post_signed_p(SAPI_V1_ASSET_DUST, Some(params), self.recv_window)
            .await
    }

    /// Asset Dividend Record
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let records = tokio_test::block_on(wallet.asset_dividends(AssetDividendQuery::default()));
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn asset_dividends(&self, query: AssetDividendQuery) -> Result<RecordsQueryResult<AssetDividend>> {
        self.client
            .get_signed_p(SAPI_V1_ASSET_ASSETDIVIDEND, Some(query), self.recv_window)
            .await
    }

    /// Asset Details
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let records = tokio_test::block_on(wallet.asset_detail(None));
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn asset_detail(&self, asset: Option<String>) -> Result<SupportedAssetDetails> {
        self.client
            .get_signed_p(SAPI_V1_ASSET_ASSETDETAIL, asset, self.recv_window)
            .await
    }

    /// Trade Fees
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let records = tokio_test::block_on(wallet.trade_fees(None));
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn trade_fees(&self, symbol: Option<String>) -> Result<TradeFees> {
        let mut query = HashMap::new();
        query.insert("symbol", symbol);
        self.client
            .get_signed_p(
                if self.binance_us_api {
                    SAPI_V1_ASSET_TRADEFEE_US
                } else {
                    SAPI_V1_ASSET_TRADEFEE
                },
                Some(query),
                self.recv_window,
            )
            .await
    }

    /// Funding Wallet
    ///
    /// Currently supports querying the following business assets：Binance Pay, Binance Card, Binance Gift Card, Stock Token
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let records = tokio_test::block_on(wallet.funding_wallet(None, None));
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn funding_wallet(
        &self,
        asset: Option<String>,
        need_btc_valuation: Option<bool>,
    ) -> Result<WalletFundings> {
        let mut query = HashMap::new();
        query.insert("asset", asset);
        query.insert("need_btc_valuation", need_btc_valuation.map(|b| format!("{b}")));
        self.client
            .post_signed_p(SAPI_V1_ASSET_GETFUNDINGASSET, Some(query), self.recv_window)
            .await
    }

    /// Api Key Permissions
    ///
    /// # Examples
    /// ```rust,no_run
    /// use binance::{api::*, wallet::*, config::*, rest_model::*};
    /// let wallet: Wallet = Binance::new_with_env(&Config::testnet());
    /// let records = tokio_test::block_on(wallet.api_key_permissions());
    /// assert!(records.is_ok(), "{:?}", records);
    /// ```
    pub async fn api_key_permissions(&self) -> Result<ApiKeyPermissions> {
        self.client
            .get_signed_p(SAPI_V1_ASSET_APIRESTRICTIONS, Option::<String>::None, self.recv_window)
            .await
    }
}
