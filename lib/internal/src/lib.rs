use breez_sdk_input::PaymentMethodType;
pub mod utils;

pub trait BreezServicesImpl: Send + Sync {
    /// Returns the payment methods supported by this implementation, ordered by preference.
    fn get_payment_methods(&self) -> Vec<PaymentMethodType>;
}
