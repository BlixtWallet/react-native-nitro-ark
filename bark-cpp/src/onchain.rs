use bark::onchain::Utxo;
use bdk_wallet::bitcoin::{Address, Amount, FeeRate, Txid};

use crate::GLOBAL_WALLET_MANAGER;

/// Get onchain balance
pub async fn onchain_balance() -> anyhow::Result<bdk_wallet::Balance> {
    let manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context_ref(|ctx| Ok(ctx.onchain_wallet.balance()))
}

/// Get a new address
pub async fn address() -> anyhow::Result<Address> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context(|ctx| ctx.onchain_wallet.address())
}

/// Get unspent outputs
pub async fn list_unspent() -> anyhow::Result<Vec<bdk_wallet::LocalOutput>> {
    let manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context_ref(|ctx| Ok(ctx.onchain_wallet.list_unspent()))
}

/// Get utxos
pub async fn utxos() -> anyhow::Result<Vec<Utxo>> {
    let manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager.with_context_ref(|ctx| Ok(ctx.onchain_wallet.utxos()))
}

/// Send onchain transaction
pub async fn send(dest: Address, amount: Amount, fee_rate: FeeRate) -> anyhow::Result<Txid> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.onchain_wallet
                .send(&ctx.chain_client, dest, amount, fee_rate)
                .await
        })
        .await
}

/// Send many onchain transactions
pub async fn send_many<T: IntoIterator<Item = (Address, Amount)> + Send>(
    destinations: T,
    fee_rate: FeeRate,
) -> anyhow::Result<Txid> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.onchain_wallet
                .send_many(&ctx.chain_client, destinations, fee_rate)
                .await
        })
        .await
}

/// Drain the wallet to a destination address with a specified fee rate
pub async fn drain(destination: Address, fee_rate: FeeRate) -> anyhow::Result<Txid> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async {
            ctx.onchain_wallet
                .drain(&ctx.chain_client, destination, fee_rate)
                .await
        })
        .await
}

/// Synchronize the onchain wallet with the blockchain
pub async fn sync() -> anyhow::Result<Amount> {
    let mut manager = GLOBAL_WALLET_MANAGER.lock().await;
    manager
        .with_context_async(|ctx| async { ctx.onchain_wallet.sync(&ctx.chain_client).await })
        .await
}
