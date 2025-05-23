use crate::utils::default_true;
use serde::{Deserialize, Serialize};

/// [SuccessAction] where contents are ready to be consumed by the caller
///
/// Contents are identical to [SuccessAction], except for AES where the ciphertext is decrypted.
#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub enum SuccessActionProcessed {
    /// See [SuccessAction::Aes] for received payload
    ///
    /// See [AesSuccessActionDataDecrypted] for decrypted payload
    Aes { result: AesSuccessActionDataResult },

    /// See [SuccessAction::Message]
    Message { data: MessageSuccessActionData },

    /// See [SuccessAction::Url]
    Url { data: UrlSuccessActionData },
}

/// Result of decryption of [AesSuccessActionData] payload
#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub enum AesSuccessActionDataResult {
    Decrypted { data: AesSuccessActionDataDecrypted },
    ErrorStatus { reason: String },
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct AesSuccessActionDataDecrypted {
    /// Contents description, up to 144 characters
    pub description: String,

    /// Decrypted content
    pub plaintext: String,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct MessageSuccessActionData {
    pub message: String,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct UrlSuccessActionData {
    /// Contents description, up to 144 characters
    pub description: String,

    /// URL of the success action
    pub url: String,

    /// Indicates the success URL domain matches the LNURL callback domain.
    ///
    /// See <https://github.com/lnurl/luds/blob/luds/09.md>
    #[serde(default = "default_true")]
    pub matches_callback_domain: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LnurlPayErrorData {
    pub payment_hash: String,
    pub reason: String,
}
