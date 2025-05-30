use anyhow::anyhow;
use breez_sdk_spark::{
    BitcoinPaymentMethod, BreezSdk, InputType, LightningPaymentMethod, ListPaymentsRequest,
    LnurlPaymentMethod, MilliSatoshi, PickedPaymentMethod,
    PrepareReceivePaymentRequest, PrepareSendBitcoinRequest, PrepareSendLightningRequest,
    PrepareSendLnurlPayRequest, ReceiveMethod, ReceivePaymentRequest, SendBitcoinRequest,
    SendLightningRequest, SendLnurlPayRequest, parse,
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
        #[arg(short = 'r', long)]
        amount: Option<u64>,
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
        } => {
            let parsed = parse(&payment_request).await?;
            let payment_request = match parsed {
                InputType::PaymentRequest(payment_request) => payment_request,
                _ => return Err(anyhow::anyhow!("Not a payment request")),
            };

            let picked = sdk.pick_payment_method(payment_request).await?;
            match picked {
                PickedPaymentMethod::Bitcoin(bitcoin_payment_method) => {
                    let amount =
                        amount.ok_or(anyhow!("Amount is required for Bitcoin payments"))?;
                    match &bitcoin_payment_method {
                        BitcoinPaymentMethod::BitcoinAddress(bitcoin_address) => {
                            println!("Bitcoin address: {}", bitcoin_address.address)
                        }
                        BitcoinPaymentMethod::SilentPaymentAddress(silent_payment_address) => {
                            println!("Silent payment address: {}", silent_payment_address.address)
                        }
                    }
                    let recommended = sdk.fetch_recommended_fees().await?;
                    print_value(&recommended)?;
                    let rate = rl.readline_with_initial(
                        "fee rate (sat/kw)",
                        (&recommended.fastest_fee.to_string(), ""),
                    )?;
                    let prepared = sdk
                        .prepare_send_bitcoin(PrepareSendBitcoinRequest {
                            amount: MilliSatoshi(amount * 1000),
                            method: bitcoin_payment_method,
                            fee_rate_sat_per_kw: Some(rate.parse()?),
                        })
                        .await?;
                    print_value(&prepared)?;
                    let result = sdk.send_bitcoin(SendBitcoinRequest { prepared }).await?;
                    print_value(&result)?;
                    Ok(true)
                }
                PickedPaymentMethod::Lightning(lightning_payment_request) => {
                    match &lightning_payment_request.method {
                        LightningPaymentMethod::Bolt11Invoice(bolt11_invoice) => {
                            println!("Bolt11 invoice: {}", bolt11_invoice.bolt11)
                        }
                        LightningPaymentMethod::Bolt12Invoice(bolt12_invoice) => {
                            println!("Bolt12 invoice: {}", bolt12_invoice.invoice)
                        }
                        LightningPaymentMethod::Bolt12Offer(bolt12_offer) => {
                            println!("Bolt12 offer: {}", bolt12_offer.offer)
                        }
                    }

                    let amount = match (
                        lightning_payment_request.min_amount.0,
                        lightning_payment_request.max_amount.0,
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
                        .prepare_send_lightning(PrepareSendLightningRequest {
                            payment_request: lightning_payment_request,
                            amount: MilliSatoshi(amount),
                        })
                        .await?;
                    let result = sdk
                        .send_lightning(SendLightningRequest { prepared })
                        .await?;
                    print_value(&result)?;
                    Ok(true)
                }
                PickedPaymentMethod::LnurlPay(lnurl_payment_request) => {
                    match &lnurl_payment_request.payment_method {
                        LnurlPaymentMethod::LnurlPay(url) => println!("LnurlPay request: {}", url),
                        LnurlPaymentMethod::LightningAddress(address) => {
                            println!("Lightning address: {}", address)
                        }
                    }

                    let amount = match (
                        lnurl_payment_request.request.min_sendable,
                        lnurl_payment_request.request.max_sendable,
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

                    let line = rl.readline("comment (optional)")?;
                    let comment = if line.is_empty() { None } else { Some(line) };
                    let prepared = sdk
                        .prepare_send_lnurl_pay(PrepareSendLnurlPayRequest {
                            lnurl_pay: lnurl_payment_request,
                            amount: MilliSatoshi(amount),
                            comment,
                        })
                        .await?;
                    let result = sdk.send_lnurl_pay(SendLnurlPayRequest { prepared }).await?;
                    print_value(&result)?;
                    Ok(true)
                }
                PickedPaymentMethod::LiquidAddress(liquid_address) => todo!(),
            }
        }
        Command::ReceiveOnchain => {
            let line = rl.readline("amount (satoshis)")?;
            let amount: u64 = line.parse().map_err(|_| anyhow!("Invalid amount"))?;
            let prepared = sdk
                .prepare_receive_payment(PrepareReceivePaymentRequest {
                    receive_method: ReceiveMethod::BitcoinAddress,
                    amount: MilliSatoshi(amount * 1000),
                })
                .await?;
            let line = rl.readline("description (optional)")?;
            let description = if line.is_empty() { None } else { Some(line) };
            let result = sdk
                .receive_payment(ReceivePaymentRequest {
                    prepared,
                    description,
                    use_description_hash: None,
                })
                .await?;
            print_value(&result)?;
            Ok(true)
        }
        Command::ReceiveLightning { amount, memo } => {
            let prepared = sdk
                .prepare_receive_payment(PrepareReceivePaymentRequest {
                    receive_method: ReceiveMethod::Bolt11Invoice,
                    amount: MilliSatoshi(amount * 1000),
                })
                .await?;
            let result = sdk
                .receive_payment(ReceivePaymentRequest {
                    prepared,
                    description: memo,
                    use_description_hash: None,
                })
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
