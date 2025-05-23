use serde::{Deserialize, Serialize};

/// Wrapped in a [LnUrlError], this represents a LNURL-endpoint error.
#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct LnurlErrorData {
    pub reason: String,
}
