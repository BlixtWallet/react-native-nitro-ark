package com.margelo.nitro.nitroark

data class Bolt11InvoiceResult(
    val bolt11Invoice: String,
    val paymentSecret: String,
    val paymentHash: String,
)

data class KeyPairResultAndroid(
    val publicKey: String,
    val secretKey: String,
)

data class RoundStatusResult(
    val status: String,
    val fundingTxid: String?,
    val unsignedFundingTxids: List<String>,
    val error: String?,
    val isFinal: Boolean,
    val isSuccess: Boolean,
)
