use anyhow::anyhow;
use breez_sdk_spark::{
    BreezSdk, InputType, ListPaymentsRequest, PrepareReceivePaymentRequest,
    PrepareSendPaymentRequest, ReceiveMethod, ReceivePaymentRequest, SendPaymentRequest,
};
use clap::Parser;
use rustyline::{
    Completer, Editor, Helper, Hinter, Validator, highlight::Highlighter, hint::HistoryHinter,
    history::DefaultHistory,
};
use std::borrow::Cow::{self, Owned};

#[derive(Clone, Parser)]
pub enum Command {
    /// Exit the interactive shell (interactive mode only)
    #[command(hide = true)]
    Exit,

    /// Get balance information
    GetInfo,

    /// Get the payment with the given ID
    GetPayment {
        /// The ID of the payment to retrieve
        payment_id: String,
    },

    /// Lists payments
    ListPayments {
        /// Number of payments to show
        #[arg(short, long, default_value = "10")]
        limit: u32,

        /// Number of payments to skip
        #[arg(short, long, default_value = "0")]
        offset: u32,
    },

    /// Pay the given payment request
    Pay {
        /// The payment request to pay
        #[arg(short = 'r', long)]
        payment_request: String,

        /// Optional amount to pay in satoshis
        #[arg(short = 'a', long)]
        amount: Option<u64>,

        /// Optional message for the payment
        #[arg(short = 'm', long)]
        message: Option<String>,
    },

    /// Receive via onchain address
    ReceiveOnchain,

    /// Create a Lightning invoice
    ReceiveLightning {
        /// The amount to receive in satoshis
        #[arg(short, long)]
        amount: u64,

        /// Optional description/memo for the invoice
        #[arg(short, long)]
        memo: Option<String>,
    },
}

#[derive(Helper, Completer, Hinter, Validator)]
pub struct CliHelper {
    #[rustyline(Hinter)]
    pub hinter: HistoryHinter,
}

impl Highlighter for CliHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }
}

pub(crate) async fn execute_command(
    rl: &mut Editor<CliHelper, DefaultHistory>,
    command: Command,
    sdk: &BreezSdk,
) -> Result<bool, anyhow::Error> {
    match command {
        Command::Exit => Ok(false),
        Command::GetInfo => {
            let value = sdk.get_info().await?;
            print_value(&value)?;
            Ok(true)
        }
        Command::GetPayment { payment_id } => {
            let value = sdk.get_payment(&payment_id).await?;
            print_value(&value)?;
            Ok(true)
        }
        Command::ListPayments { limit, offset } => {
            let value = sdk
                .list_payments(ListPaymentsRequest {
                    limit: Some(limit),
                    offset: Some(offset),
                    ..Default::default()
                })
                .await?;
            print_value(&value)?;
            Ok(true)
        }
        Command::Pay {
            payment_request,
            amount,
            message,
        } => {
            let parsed = sdk.parse(&payment_request).await?;
            match parsed {
                InputType::BitcoinAddress(address) => {
                    let amount =
                        amount.ok_or(anyhow!("Amount is required for Bitcoin payments"))?;
                    println!("Bitcoin address: {}", address.details.address);
                    let recommended = sdk.fetch_recommended_fees().await?;
                    print_value(&recommended)?;
                    let rate = rl.readline_with_initial(
                        "fee rate (sat/vbyte)",
                        (&recommended.fastest_fee.to_string(), ""),
                    )?;
                    let prepared = sdk
                        .prepare_send_payment(
                            breez_sdk_spark::PrepareSendPaymentRequest::BitcoinAddress {
                                address,
                                amount_sat: amount,
                                fee_rate_sat_per_vbyte: Some(rate.parse()?),
                            },
                        )
                        .await?;
                    print_value(&prepared)?;
                    let result = sdk.send_payment(SendPaymentRequest { prepared }).await?;
                    print_value(&result)?;
                    Ok(true)
                }
                InputType::Bolt11Invoice(invoice) => {
                    println!("Bolt11 invoice: {}", invoice.details.invoice);
                    let amount_msat = match (invoice.min_amount_msat, invoice.max_amount_msat) {
                        (min, max) if min > 0 && min == max => min,
                        (min, max) => {
                            let line = rl.readline_with_initial(
                                &format!("amount (msat) between {} and {}", min, max),
                                (&min.to_string(), ""),
                            )?;
                            line.parse()?
                        }
                    };
                    let prepared = sdk
                        .prepare_send_payment(PrepareSendPaymentRequest::Bolt11Invoice {
                            invoice,
                            amount_msat,
                        })
                        .await?;
                    let result = sdk.send_payment(SendPaymentRequest { prepared }).await?;
                    print_value(&result)?;
                    Ok(true)
                }
                InputType::Bolt12Invoice(invoice) => {
                    println!("Bolt12 invoice: {}", invoice.details.invoice);
                    let prepared = sdk
                        .prepare_send_payment(PrepareSendPaymentRequest::Bolt12Invoice { invoice })
                        .await?;
                    let result = sdk.send_payment(SendPaymentRequest { prepared }).await?;
                    print_value(&result)?;
                    Ok(true)
                }
                InputType::Bolt12InvoiceRequest(bolt12_invoice_request) => {
                    return Err(anyhow!("Not a payment request"));
                }
                InputType::Bolt12Offer(offer) => {
                    println!("Bolt12 offer: {}", offer.details.offer);
                    let amount_msat = match (offer.min_amount_msat, offer.max_amount_msat) {
                        (min, max) if min > 0 && min == max => min,
                        (min, max) => {
                            let line = rl.readline_with_initial(
                                &format!("amount (msat) between {} and {}", min, max),
                                (&min.to_string(), ""),
                            )?;
                            line.parse()?
                        }
                    };
                    let prepared = sdk
                        .prepare_send_payment(PrepareSendPaymentRequest::Bolt12Offer {
                            offer,
                            amount_msat,
                            message,
                        })
                        .await?;
                    let result = sdk.send_payment(SendPaymentRequest { prepared }).await?;
                    print_value(&result)?;
                    Ok(true)
                }
                InputType::LightningAddress(address) => {
                    println!("Lightning address: {}", address.address);
                    let amount_msat = match (
                        address.pay_request.min_sendable,
                        address.pay_request.max_sendable,
                    ) {
                        (min, max) if min > 0 && min == max => min,
                        (min, max) => {
                            let line = rl.readline_with_initial(
                                &format!("amount (msat) between {} and {}", min, max),
                                (&min.to_string(), ""),
                            )?;
                            line.parse()?
                        }
                    };
                    let prepared = sdk
                        .prepare_send_payment(PrepareSendPaymentRequest::LightningAddress {
                            address,
                            amount_msat,
                            message,
                        })
                        .await?;
                    let result = sdk.send_payment(SendPaymentRequest { prepared }).await?;
                    print_value(&result)?;
                    Ok(true)
                }
                InputType::LiquidAddress(address) => {
                    println!("Liquid address: {}", address.details.address);
                    let amount = amount.ok_or(anyhow!("Amount is required for Liquid payments"))?;
                    let prepared = sdk
                        .prepare_send_payment(PrepareSendPaymentRequest::LiquidAddress {
                            address,
                            amount_sat: amount,
                        })
                        .await?;
                    print_value(&prepared)?;
                    let result = sdk.send_payment(SendPaymentRequest { prepared }).await?;
                    print_value(&result)?;
                    Ok(true)
                }
                InputType::LnurlAuth(lnurl_auth_request_data) => {
                    return Err(anyhow!("Not a payment request"));
                }
                InputType::LnurlPay(url) => {
                    println!("Lnurl pay: {}", url.url);
                    let amount_msat = match (url.min_sendable, url.max_sendable) {
                        (min, max) if min > 0 && min == max => min,
                        (min, max) => {
                            let line = rl.readline_with_initial(
                                &format!("amount (msat) between {} and {}", min, max),
                                (&min.to_string(), ""),
                            )?;
                            line.parse()?
                        }
                    };
                    let prepared = sdk
                        .prepare_send_payment(PrepareSendPaymentRequest::LnurlPay {
                            url,
                            amount_msat,
                            message,
                        })
                        .await?;
                    let result = sdk.send_payment(SendPaymentRequest { prepared }).await?;
                    print_value(&result)?;
                    Ok(true)
                }
                InputType::LnurlWithdraw(lnurl_withdraw_request_data) => {
                    return Err(anyhow!("Not a payment request"));
                }
                InputType::SilentPaymentAddress(address) => {
                    println!("Silent payment address: {}", address.details.address);
                    let amount = amount.ok_or(anyhow!(
                        "Amount is required for Silent Payment Address payments"
                    ))?;
                    let recommended = sdk.fetch_recommended_fees().await?;
                    print_value(&recommended)?;
                    let rate = rl.readline_with_initial(
                        "fee rate (sat/vbyte)",
                        (&recommended.fastest_fee.to_string(), ""),
                    )?;
                    let prepared = sdk
                        .prepare_send_payment(PrepareSendPaymentRequest::SilentPaymentAddress {
                            address,
                            amount_sat: amount,
                            fee_rate_sat_per_vbyte: Some(rate.parse()?),
                        })
                        .await?;
                    print_value(&prepared)?;
                    let result = sdk.send_payment(SendPaymentRequest { prepared }).await?;
                    print_value(&result)?;
                    Ok(true)
                }
                InputType::Url(_) => return Err(anyhow!("Not a payment request")),
            }
        }
        Command::ReceiveOnchain => {
            let line = rl.readline("amount (satoshis)")?;
            let amount: u64 = line.parse().map_err(|_| anyhow!("Invalid amount"))?;
            let line = rl.readline("message (optional)")?;
            let message = if line.is_empty() { None } else { Some(line) };
            let prepared = sdk
                .prepare_receive_payment(PrepareReceivePaymentRequest {
                    amount_msat: amount * 1000,
                    message,
                    receive_method: ReceiveMethod::BitcoinAddress,
                })
                .await?;
            let result = sdk
                .receive_payment(ReceivePaymentRequest { prepared })
                .await?;
            print_value(&result)?;
            Ok(true)
        }
        Command::ReceiveLightning { amount, memo } => {
            let prepared = sdk
                .prepare_receive_payment(PrepareReceivePaymentRequest {
                    amount_msat: amount * 1000,
                    message: memo,
                    receive_method: ReceiveMethod::Bolt11Invoice,
                })
                .await?;
            let result = sdk
                .receive_payment(ReceivePaymentRequest { prepared })
                .await?;
            print_value(&result)?;
            Ok(true)
        }
    }
}

fn print_value<T: serde::Serialize>(value: &T) -> Result<(), serde_json::Error> {
    let serialized = serialize(value)?;
    println!("{}", serialized);
    Ok(())
}

fn serialize<T: serde::Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
}
