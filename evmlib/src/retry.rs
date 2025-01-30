use crate::common::{Address, Calldata, TxHash};
use crate::TX_TIMEOUT;
use alloy::network::{Network, TransactionBuilder};
use alloy::providers::{PendingTransactionBuilder, Provider};
use alloy::transports::Transport;
use std::time::Duration;

pub(crate) const MAX_RETRIES: u8 = 3;

pub(crate) async fn with_retries<F, Fut, T, E>(mut action: F) -> Result<T, E>
where
    F: FnMut() -> Fut + Send,
    Fut: std::future::Future<Output = Result<T, E>> + Send,
    E: std::fmt::Debug,
{
    let mut retries = 0;
    loop {
        match action().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                if retries == MAX_RETRIES {
                    error!("Operation failed after {retries} retries: {err:?}");
                    return Err(err);
                }
                retries += 1;
                let delay = Duration::from_secs(retries.pow(2) as u64);
                warn!("Retry #{retries} in {delay:?} due to error: {err:?}");
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Generic function to send a transaction with retries.
pub(crate) async fn send_transaction_with_retries<P, T, N, E>(
    provider: P,
    calldata: Calldata,
    to: Address,
    tx_identifier: &str,
) -> Result<TxHash, E>
where
    T: Transport + Clone,
    P: Provider<T, N>,
    N: Network,
    E: From<alloy::transports::RpcError<alloy::transports::TransportErrorKind>>
        + From<alloy::providers::PendingTransactionError>,
{
    let mut nonce: Option<u64> = None;
    let mut retries = 0;

    loop {
        let result = {
            let mut transaction_request = provider
                .transaction_request()
                .with_to(to)
                .with_input(calldata.clone());

            if let Some(nonce) = nonce {
                transaction_request.set_nonce(nonce);
            } else {
                nonce = transaction_request.nonce();
            }

            let pending_tx_builder = provider
                .send_transaction(transaction_request.clone())
                .await?;

            debug!(
                "{tx_identifier} transaction is pending with tx_hash: {:?}",
                pending_tx_builder.tx_hash()
            );

            with_retries(|| async {
                PendingTransactionBuilder::from_config(
                    provider.root().clone(),
                    pending_tx_builder.inner().clone(),
                )
                .with_timeout(Some(TX_TIMEOUT))
                .watch()
                .await
            })
            .await
        };

        match result {
            Ok(tx_hash) => {
                debug!("{tx_identifier} transaction with hash {tx_hash:?} is successful");
                break Ok(tx_hash);
            }
            Err(err) => {
                if retries == MAX_RETRIES {
                    error!("Failed to send and confirm {tx_identifier} transaction after {retries} retries. Giving up. Error: {err:?}");
                    break Err(E::from(err));
                }

                retries += 1;
                let retry_delay_secs = retries.pow(2) as u64;

                warn!(
                        "Error sending and confirming {tx_identifier} transaction: {err:?}. Try #{}. Trying again in {} second(s).",
                        retries,
                        retry_delay_secs
                    );

                tokio::time::sleep(Duration::from_secs(retry_delay_secs)).await;
            }
        }
    }
}
