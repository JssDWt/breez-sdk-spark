use crate::input::ParseError;

use std::fmt;
use thiserror::Error;

#[derive(Debug)]
pub enum ServiceConnectivityErrorKind {
    Builder,
    Redirect,
    Status,
    Timeout,
    Request,
    Connect,
    Body,
    Decode,
    Json,
    Other,
}
impl fmt::Display for ServiceConnectivityErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Error)]
#[error("{kind}: {err}")]
pub struct ServiceConnectivityError {
    pub kind: ServiceConnectivityErrorKind,
    pub err: String,
}
impl ServiceConnectivityError {
    pub fn new(kind: ServiceConnectivityErrorKind, err: String) -> Self {
        ServiceConnectivityError { kind, err }
    }
}
impl From<reqwest::Error> for ServiceConnectivityError {
    fn from(err: reqwest::Error) -> Self {
        #[allow(unused_mut)]
        let mut kind = if err.is_builder() {
            ServiceConnectivityErrorKind::Builder
        } else if err.is_redirect() {
            ServiceConnectivityErrorKind::Redirect
        } else if err.is_status() {
            ServiceConnectivityErrorKind::Status
        } else if err.is_timeout() {
            ServiceConnectivityErrorKind::Timeout
        } else if err.is_request() {
            ServiceConnectivityErrorKind::Request
        } else if err.is_body() {
            ServiceConnectivityErrorKind::Body
        } else if err.is_decode() {
            ServiceConnectivityErrorKind::Decode
        } else {
            ServiceConnectivityErrorKind::Other
        };
        #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
        if err.is_connect() {
            kind = ServiceConnectivityErrorKind::Connect;
        }
        Self {
            kind,
            err: err.to_string(),
        }
    }
}

pub enum ParseAndPickError {
    Parse(ParseError),
    Pick(PickPaymentMethodError),
}
impl From<ParseError> for ParseAndPickError {
    fn from(err: ParseError) -> Self {
        ParseAndPickError::Parse(err)
    }
}
impl From<PickPaymentMethodError> for ParseAndPickError {
    fn from(err: PickPaymentMethodError) -> Self {
        ParseAndPickError::Pick(err)
    }
}
pub enum PickPaymentMethodError {
    Unsupported,
}
pub enum PrepareSendBitcoinError {
    InvalidAddress,
    InvalidNetwork,
}
pub enum PrepareSendLightningError {}
pub enum PrepareSendLiquidAddressError {}
pub enum PrepareSendLnurlPayError {}
pub enum PrepareReceivePaymentError {}
pub enum ReceivePaymentError {}
pub enum SendBitcoinError {}
pub enum SendLightningError {}
pub enum SendLiquidAddressError {}
pub enum SendLnurlPayError {}
