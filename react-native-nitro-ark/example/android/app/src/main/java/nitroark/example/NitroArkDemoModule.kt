package nitroark.example

import com.facebook.react.bridge.Promise
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.bridge.ReactContextBaseJavaModule
import com.facebook.react.bridge.ReactMethod
import com.facebook.react.bridge.ReadableMap
import com.facebook.react.module.annotations.ReactModule
import com.margelo.nitro.nitroark.NitroArkNative
import android.util.Log

@ReactModule(name = NitroArkDemoModule.NAME)
class NitroArkDemoModule(reactContext: ReactApplicationContext) :
    ReactContextBaseJavaModule(reactContext) {

  override fun getName(): String = NAME

  @ReactMethod
  fun loadWallet(datadir: String, mnemonic: String, config: ReadableMap?, promise: Promise) {
    try {
      val nestedConfig = config?.getMapOrNull("config")
      if (config != null) {
        Log.i(NAME, "loadWallet config top-level: $config")
      }
      if (nestedConfig != null) {
        Log.i(NAME, "loadWallet nested config: $nestedConfig")
      }

      val parsedConfig = nestedConfig?.let { map ->
        NitroArkNative.AndroidBarkConfig(
            ark = map.getStringOrNull("ark"),
            esplora = map.getStringOrNull("esplora"),
            bitcoind = map.getStringOrNull("bitcoind"),
            bitcoindCookie = map.getStringOrNull("bitcoind_cookie"),
            bitcoindUser = map.getStringOrNull("bitcoind_user"),
            bitcoindPass = map.getStringOrNull("bitcoind_pass"),
            vtxoRefreshExpiryThreshold = map.getIntOrNull("vtxo_refresh_expiry_threshold"),
            fallbackFeeRate = map.getLongOrNull("fallback_fee_rate"),
            htlcRecvClaimDelta = map.getIntOrNull("htlc_recv_claim_delta"),
            vtxoExitMargin = map.getIntOrNull("vtxo_exit_margin"),
            roundTxRequiredConfirmations = map.getIntOrNull("round_tx_required_confirmations"),
        )
      }

      NitroArkNative.loadWallet(
          datadir = datadir,
          mnemonic = mnemonic,
          regtest = config?.getBooleanOrDefault("regtest", false) ?: false,
          signet = config?.getBooleanOrDefault("signet", false) ?: false,
          bitcoin = config?.getBooleanOrDefault("bitcoin", true) ?: true,
          birthdayHeight = config?.getIntOrNull("birthday_height"),
          config = parsedConfig)
      promise.resolve(null)
    } catch (e: Exception) {
      promise.reject("ERR_LOAD_WALLET_JNI", e)
    }
  }

  @ReactMethod
  fun isWalletLoaded(promise: Promise) {
    try {
      promise.resolve(NitroArkNative.isWalletLoaded())
    } catch (e: Exception) {
      promise.reject("ERR_IS_WALLET_LOADED_JNI", e)
    }
  }

  @ReactMethod
  fun closeWallet(promise: Promise) {
    try {
      NitroArkNative.closeWallet()
      promise.resolve(null)
    } catch (e: Exception) {
      promise.reject("ERR_CLOSE_WALLET_JNI", e)
    }
  }

  @ReactMethod
  fun maintenanceRefresh(promise: Promise) {
    try {
      NitroArkNative.maintenanceRefresh()
      promise.resolve(null)
    } catch (e: Exception) {
      promise.reject("ERR_MAINTENANCE_REFRESH_JNI", e)
    }
  }

  @ReactMethod
  fun tryClaimLightningReceive(paymentHash: String, wait: Boolean, token: String?, promise: Promise) {
    try {
      NitroArkNative.tryClaimLightningReceive(paymentHash, wait, token)
      promise.resolve(null)
    } catch (e: Exception) {
      promise.reject("ERR_TRY_CLAIM_LN_RECEIVE_JNI", e)
    }
  }

  @ReactMethod
  fun offboardAll(destinationAddress: String, promise: Promise) {
    try {
      val result = NitroArkNative.offboardAll(destinationAddress)
      promise.resolve(result)
    } catch (e: Exception) {
      promise.reject("ERR_OFFBOARD_ALL_JNI", e)
    }
  }

  @ReactMethod
  fun peakKeyPair(index: Int, promise: Promise) {
    try {
      val result = NitroArkNative.peakKeyPair(index)
      promise.resolve(result)
    } catch (e: Exception) {
      promise.reject("ERR_PEAK_KEYPAIR_JNI", e)
    }
  }

  @ReactMethod
  fun verifyMessage(message: String, signature: String, publicKey: String, promise: Promise) {
    try {
      val result = NitroArkNative.verifyMessage(message, signature, publicKey)
      promise.resolve(result)
    } catch (e: Exception) {
      promise.reject("ERR_VERIFY_MESSAGE_JNI", e)
    }
  }

  @ReactMethod
  fun bolt11Invoice(amountMsat: Double, promise: Promise) {
    try {
      val result = NitroArkNative.bolt11Invoice(amountMsat.toLong())
      promise.resolve(result)
    } catch (e: Exception) {
      promise.reject("ERR_BOLT11_INVOICE_JNI", e)
    }
  }

  companion object {
    const val NAME = "NitroArkDemo"
  }
}

private fun ReadableMap.getStringOrNull(key: String): String? =
    if (hasKey(key) && !isNull(key)) getString(key) else null

private fun ReadableMap.getIntOrNull(key: String): Int? =
    if (hasKey(key) && !isNull(key)) getInt(key) else null

private fun ReadableMap.getLongOrNull(key: String): Long? =
    if (hasKey(key) && !isNull(key)) getDouble(key).toLong() else null

private fun ReadableMap.getBooleanOrDefault(key: String, defaultValue: Boolean): Boolean =
    if (hasKey(key) && !isNull(key)) getBoolean(key) else defaultValue

private fun ReadableMap.getMapOrNull(key: String): ReadableMap? =
    if (hasKey(key) && !isNull(key)) getMap(key) else null
