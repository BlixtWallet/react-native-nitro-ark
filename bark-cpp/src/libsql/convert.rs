use bark::ark::bitcoin::Amount;
use bark::movement::{Movement, MovementRecipient, VtxoSubset};
use bark::persist::OffchainBoard;
use libsql::Row;

pub(crate) fn row_to_movement(row: &Row) -> anyhow::Result<Movement> {
    let fees: Amount = Amount::from_sat(row.get(2)?);

    let spends = serde_json::from_str::<Vec<VtxoSubset>>(&row.get::<String>(3)?)?;
    let receives = serde_json::from_str::<Vec<VtxoSubset>>(&row.get::<String>(4)?)?;
    let recipients = serde_json::from_str::<Vec<MovementRecipient>>(&row.get::<String>(5)?)?;

    Ok(Movement {
        id: row.get(0)?,
        fees,
        spends,
        receives,
        recipients,
        created_at: row.get(1)?,
    })
}

pub(crate) fn row_to_offchain_board(row: &Row) -> anyhow::Result<OffchainBoard> {
    let raw_payment = row.get::<Vec<u8>>(2)?;
    Ok(OffchainBoard {
        payment_hash: row.get(0)?,
        payment_preimage: row.get(1)?,
        payment: serde_json::from_slice(&raw_payment)?,
    })
}
