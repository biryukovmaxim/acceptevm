use web3::{transports::Http, types::U256, Web3};

use crate::{
    audit::logger::log_sync,
    erc20::ERC20Token,
    gateway::{
        db::{delete, get_all},
        PaymentGateway,
    },
    types::Invoice,
};

async fn check_if_token_received(
    token: ERC20Token,
    invoice: Invoice,
) -> Result<bool, web3::contract::Error> {
    let balance_of_recipient = token.get_balance(invoice.to).await?;
    if balance_of_recipient.ge(&invoice.amount) {
        return Ok(true);
    }
    Ok(false)
}

/// Retrieves the gas token balance of the specified address on the specified web3 instance
async fn get_native_balance(web3: Web3<Http>, address: String) -> Result<U256, web3::Error> {
    web3.eth().balance(address.parse().unwrap(), None).await
}

/// Used to check if the invoice recipient has received enough money to cover the invoice
async fn check_if_native_received(web3: Web3<Http>, invoice: Invoice) -> Result<bool, web3::Error> {
    let balance_of_recipient = get_native_balance(web3, invoice.to).await?;
    if balance_of_recipient.ge(&invoice.amount) {
        return Ok(true);
    }
    Ok(false)
}

/// A function that branches control flow depending on the invoice shall
/// be paid by an ERC20-compatible token or the native gas token on the network
async fn check_and_process(web3: Web3<Http>, invoice: Invoice) -> bool {
    match invoice.clone().method.token_address {
        Some(address) => {
            let token = ERC20Token::new(web3, address);
            match check_if_token_received(token, invoice).await {
                Ok(result) => result,
                Err(error) => {
                    log_sync(&format!("Failed to check balance: {}", error));
                    false
                }
            }
        }
        None => match check_if_native_received(web3, invoice).await {
            Ok(result) => result,
            Err(error) => {
                log_sync(&format!("Failed to check balance: {}", error));
                false
            }
        },
    }
}

/// Periodically checks if invoices are paid in accordance
/// to the specified polling interval.
pub async fn poll_payments(gateway: PaymentGateway) {
    loop {
        match get_all::<Invoice>(&gateway.tree).await {
            Ok(all) => {
                for entry in all {
                    let check_result =
                        check_and_process(gateway.web3.clone(), entry.clone().1).await;
                    if check_result {
                        match delete(&gateway.tree, &entry.0).await {
                            Ok(()) => {
                                let mut lock = gateway.callback.lock().await;
                                (&mut *lock)(entry.1).await;
                            }
                            Err(error) => {
                                log_sync(&format!(
                                    "Could not remove paid invoice, did not callback: {}",
                                    error
                                ));
                            }
                        }
                    }
                }
            }
            Err(error) => {
                log_sync(&format!(
                    "Could not get all invoices, did not callback: {}",
                    error
                ));
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(
            gateway.poll_interval_seconds,
        ))
        .await;
    }
}

#[cfg(test)]
mod tests {
    use web3::{transports::Http, types::U256, Web3};

    use crate::poller::get_native_balance;

    #[tokio::test]
    async fn valid_balance() {
        let http = Http::new("https://bsc-dataseed1.binance.org/").unwrap();
        let web3 = Web3::new(http);
        let balance = get_native_balance(
            web3,
            "0x2170ed0880ac9a755fd29b2688956bd959f933f8".to_string(),
        )
        .await
        .unwrap();
        println!("Balance check: {}", balance);
        assert!(balance.ge(&U256::zero()));
    }
}
