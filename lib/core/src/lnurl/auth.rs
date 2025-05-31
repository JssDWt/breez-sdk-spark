use bitcoin::bip32::ChildNumber;
use breez_sdk_common::lnurl::error::LnurlResult;

pub struct LnurlAuthSigner {}

impl LnurlAuthSigner {
    pub fn new() -> Self {
        LnurlAuthSigner {}
    }
}

#[breez_sdk_macros::async_trait]
impl breez_sdk_common::lnurl::auth::LnurlAuthSigner for LnurlAuthSigner {
    async fn derive_bip32_pub_key(&self, _derivation_path: &[ChildNumber]) -> LnurlResult<Vec<u8>> {
        todo!()
    }
    async fn sign_ecdsa(
        &self,
        _msg: &[u8],
        _derivation_path: &[ChildNumber],
    ) -> LnurlResult<Vec<u8>> {
        todo!()
    }
    async fn hmac_sha256(
        &self,
        _key_derivation_path: &[ChildNumber],
        _input: &[u8],
    ) -> LnurlResult<Vec<u8>> {
        todo!()
    }
}
