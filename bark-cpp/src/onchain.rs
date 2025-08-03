use anyhow::Context;
use bark::onchain::{ChainSourceClient, Utxo};
use bdk_wallet::bitcoin::{Address, Amount, FeeRate, Txid};

use crate::GLOBAL_WALLET;

/// Get onchain balance
pub async fn onchain_balance() -> anyhow::Result<bdk_wallet::Balance> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (_, onchain_w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;

    Ok(onchain_w.balance())
}

/// Get a new address
pub async fn address() -> anyhow::Result<Address> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (_, onchain_w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;
    Ok(onchain_w.address()?)
}

/// Get unspent outputs
pub async fn list_unspent() -> anyhow::Result<Vec<bdk_wallet::LocalOutput>> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (_, onchain_w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;
    Ok(onchain_w.list_unspent())
}

/// Get utxos
pub async fn utxos() -> anyhow::Result<Vec<Utxo>> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (_, onchain_w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;
    Ok(onchain_w.utxos())
}

/// Send onchain transaction
pub async fn send(
    chain: &ChainSourceClient,
    dest: Address,
    amount: Amount,
    fee_rate: FeeRate,
) -> anyhow::Result<Txid> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (_, onchain_w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;
    onchain_w.send(chain, dest, amount, fee_rate).await
}

/// Send many onchain transactions
pub async fn send_many<T: IntoIterator<Item = (Address, Amount)>>(
    chain: &ChainSourceClient,
    destinations: T,
    fee_rate: FeeRate,
) -> anyhow::Result<Txid> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (_, onchain_w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;
    onchain_w.send_many(chain, destinations, fee_rate).await
}

/// Drain the wallet to a destination address with a specified fee rate
pub async fn drain(
    chain: &ChainSourceClient,
    destination: Address,
    fee_rate: FeeRate,
) -> anyhow::Result<Txid> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (_, onchain_w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;
    onchain_w.drain(chain, destination, fee_rate).await
}

/// Synchronize the onchain wallet with the blockchain
pub async fn sync(chain: &ChainSourceClient) -> anyhow::Result<Amount> {
    let mut wallet_guard = GLOBAL_WALLET.lock().await;
    let (_, onchain_w, _) = wallet_guard.as_mut().context("Wallet not loaded")?;
    onchain_w.sync(chain).await
}
