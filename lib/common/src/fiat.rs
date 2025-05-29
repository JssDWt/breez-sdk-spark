use serde::{Deserialize, Serialize};

/// Details about a supported currency in the fiat rate feed
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct CurrencyInfo {
    pub name: String,
    pub fraction_size: u32,
    pub spacing: Option<u32>,
    pub symbol: Option<Symbol>,
    pub uniq_symbol: Option<Symbol>,
    #[serde(default)]
    pub localized_name: Vec<LocalizedName>,
    #[serde(default)]
    pub locale_overrides: Vec<LocaleOverrides>,
}

/// Wrapper around the [`CurrencyInfo`] of a fiat currency
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct FiatCurrency {
    pub id: String,
    pub info: CurrencyInfo,
}

/// Localized name of a currency
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LocalizedName {
    pub locale: String,
    pub name: String,
}

/// Locale-specific settings for the representation of a currency
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LocaleOverrides {
    pub locale: String,
    pub spacing: Option<u32>,
    pub symbol: Symbol,
}

/// Denominator in an exchange rate
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Rate {
    pub coin: String,
    pub value: f64,
}

/// Settings for the symbol representation of a currency
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Symbol {
    pub grapheme: Option<String>,
    pub template: Option<String>,
    pub rtl: Option<bool>,
    pub position: Option<u32>,
}
