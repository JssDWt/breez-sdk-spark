use breez_sdk_common::{
    breez_server::BreezServer,
    buy::{BuyBitcoinProviderApi, moonpay::MoonpayProvider},
    utils::Arc,
};
use maybe_sync::{MaybeSend, MaybeSync};

use crate::{
    Network,
    error::BuyBitcoinError,
    model::{BuyBitcoinProvider, Config},
};

#[breez_sdk_macros::async_trait]
pub trait BuyBitcoinApi: MaybeSend + MaybeSync {
    /// Initiate buying Bitcoin and return a URL to the selected third party provider
    async fn buy_bitcoin(
        &self,
        provider: BuyBitcoinProvider,
        address: String,
        amount_sat: u64,
        redirect_url: Option<String>,
    ) -> Result<String, BuyBitcoinError>;
}

pub(crate) struct BuyBitcoinService {
    config: Config,
    moonpay_provider: Arc<dyn BuyBitcoinProviderApi>,
}

impl BuyBitcoinService {
    pub fn new(config: Config, breez_server: Arc<BreezServer>) -> Self {
        let moonpay_provider = Arc::new(MoonpayProvider::new(breez_server));
        Self {
            config,
            moonpay_provider,
        }
    }
}

#[breez_sdk_macros::async_trait]
impl BuyBitcoinApi for BuyBitcoinService {
    async fn buy_bitcoin(
        &self,
        provider: BuyBitcoinProvider,
        address: String,
        amount_sat: u64,
        redirect_url: Option<String>,
    ) -> Result<String, BuyBitcoinError> {
        if self.config.network != Network::Mainnet {
            return Err(BuyBitcoinError::InvalidNetwork);
        }

        match provider {
            BuyBitcoinProvider::Moonpay => self
                .moonpay_provider
                .buy_bitcoin(address, Some(amount_sat), None, redirect_url)
                .await
                .map_err(|e| BuyBitcoinError::ProviderError {
                    provider: BuyBitcoinProvider::Moonpay,
                    error: e.to_string(),
                }),
        }
    }
}
