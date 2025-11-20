#include <jni.h>
#include <string>
#include <stdexcept>
#include <exception>
#include <optional>
#include <cstdint>
#include <android/log.h>
#include <sstream>

#include "generated/ark_cxx.h"

namespace {

constexpr const char* LOG_TAG = "NitroArkJni";

// Convert a jstring to a std::string, handling null safely.
std::string JStringToString(JNIEnv* env, jstring jStr) {
  if (jStr == nullptr) {
    return std::string();
  }
  const char* chars = env->GetStringUTFChars(jStr, nullptr);
  std::string result(chars);
  env->ReleaseStringUTFChars(jStr, chars);
  return result;
}

void ThrowJavaException(JNIEnv* env, const char* message) {
  jclass exClass = env->FindClass("java/lang/RuntimeException");
  if (exClass != nullptr) {
    env->ThrowNew(exClass, message);
  }
  __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, "Throwing Java exception: %s", message);
}

std::optional<int32_t> GetOptionalInt(JNIEnv* env, jobject obj) {
  if (obj == nullptr) return std::nullopt;
  jclass cls = env->FindClass("java/lang/Integer");
  if (cls == nullptr) return std::nullopt;
  jmethodID mid = env->GetMethodID(cls, "intValue", "()I");
  if (mid == nullptr) return std::nullopt;
  jint value = env->CallIntMethod(obj, mid);
  return static_cast<int32_t>(value);
}

std::optional<int64_t> GetOptionalLong(JNIEnv* env, jobject obj) {
  if (obj == nullptr) return std::nullopt;
  jclass cls = env->FindClass("java/lang/Long");
  if (cls == nullptr) return std::nullopt;
  jmethodID mid = env->GetMethodID(cls, "longValue", "()J");
  if (mid == nullptr) return std::nullopt;
  jlong value = env->CallLongMethod(obj, mid);
  return static_cast<int64_t>(value);
}

void HandleException(JNIEnv* env, const std::exception& e) {
  __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, "Native exception: %s", e.what());
  ThrowJavaException(env, e.what());
}

void HandleUnknownException(JNIEnv* env) {
  __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, "Unknown exception in NitroArk native call.");
  ThrowJavaException(env, "Unknown exception in NitroArk native call.");
}

std::string RoundStatusToJson(const bark_cxx::RoundStatus& status) {
  std::ostringstream oss;
  oss << "{";
  oss << "\"status\":\"" << std::string(status.status.data(), status.status.length()) << "\",";
  oss << "\"funding_txid\":\"" << std::string(status.funding_txid.data(), status.funding_txid.length()) << "\",";

  oss << "\"unsigned_funding_txids\":[";
  for (size_t i = 0; i < status.unsigned_funding_txids.size(); ++i) {
    const auto& txid = status.unsigned_funding_txids[i];
    oss << "\"" << std::string(txid.data(), txid.length()) << "\"";
    if (i + 1 < status.unsigned_funding_txids.size()) {
      oss << ",";
    }
  }
  oss << "],";

  oss << "\"error\":\"" << std::string(status.error.data(), status.error.length()) << "\",";
  oss << "\"is_final\":" << (status.is_final ? "true" : "false") << ",";
  oss << "\"is_success\":" << (status.is_success ? "true" : "false");
  oss << "}";
  return oss.str();
}

std::string KeyPairToJson(const bark_cxx::KeyPairResult& keypair) {
  std::ostringstream oss;
  oss << "{";
  oss << "\"public_key\":\"" << std::string(keypair.public_key.data(), keypair.public_key.length()) << "\",";
  oss << "\"secret_key\":\"" << std::string(keypair.secret_key.data(), keypair.secret_key.length()) << "\"";
  oss << "}";
  return oss.str();
}

std::string Bolt11InvoiceToJson(const bark_cxx::Bolt11Invoice& invoice) {
  std::ostringstream oss;
  oss << "{";
  oss << "\"bolt11_invoice\":\"" << std::string(invoice.bolt11_invoice.data(), invoice.bolt11_invoice.length()) << "\",";
  oss << "\"payment_secret\":\"" << std::string(invoice.payment_secret.data(), invoice.payment_secret.length()) << "\",";
  oss << "\"payment_hash\":\"" << std::string(invoice.payment_hash.data(), invoice.payment_hash.length()) << "\"";
  oss << "}";
  return oss.str();
}

} // namespace

extern "C" {

JNIEXPORT jboolean JNICALL
Java_com_margelo_nitro_nitroark_NitroArkNative_isWalletLoaded(JNIEnv* env, jobject /*thiz*/) {
  try {
    return bark_cxx::is_wallet_loaded();
  } catch (const std::exception& e) {
    HandleException(env, e);
    return JNI_FALSE;
  } catch (...) {
    HandleUnknownException(env);
    return JNI_FALSE;
  }
}

JNIEXPORT void JNICALL
Java_com_margelo_nitro_nitroark_NitroArkNative_closeWallet(JNIEnv* env, jobject /*thiz*/) {
  try {
    bark_cxx::close_wallet();
  } catch (const std::exception& e) {
    HandleException(env, e);
  } catch (...) {
    HandleUnknownException(env);
  }
}

JNIEXPORT void JNICALL
Java_com_margelo_nitro_nitroark_NitroArkNative_loadWalletNative(
    JNIEnv* env, jobject /*thiz*/, jstring jDatadir, jstring jMnemonic, jboolean jRegtest,
    jboolean jSignet, jboolean jBitcoin, jobject jBirthdayHeight, jstring jArk, jstring jEsplora,
    jstring jBitcoind, jstring jBitcoindCookie, jstring jBitcoindUser, jstring jBitcoindPass,
    jobject jVtxoRefreshExpiryThreshold, jobject jFallbackFeeRate, jobject jHtlcRecvClaimDelta,
    jobject jVtxoExitMargin, jobject jRoundTxRequiredConfirmations) {
  try {
    const std::string datadir = JStringToString(env, jDatadir);
    const std::string mnemonic = JStringToString(env, jMnemonic);

    bark_cxx::CreateOpts opts{};
    opts.regtest = jRegtest == JNI_TRUE;
    opts.signet = jSignet == JNI_TRUE;
    opts.bitcoin = jBitcoin == JNI_TRUE;
    opts.mnemonic = mnemonic;

    auto birthday_height = GetOptionalInt(env, jBirthdayHeight);
    uint32_t birthday_height_val = 0;
    if (birthday_height.has_value()) {
      birthday_height_val = static_cast<uint32_t>(birthday_height.value());
      opts.birthday_height = &birthday_height_val;
    } else {
      opts.birthday_height = nullptr;
    }

    bark_cxx::ConfigOpts config{};
    config.ark = JStringToString(env, jArk);
    config.esplora = JStringToString(env, jEsplora);
    config.bitcoind = JStringToString(env, jBitcoind);
    config.bitcoind_cookie = JStringToString(env, jBitcoindCookie);
    config.bitcoind_user = JStringToString(env, jBitcoindUser);
    config.bitcoind_pass = JStringToString(env, jBitcoindPass);

    config.vtxo_refresh_expiry_threshold =
        static_cast<uint32_t>(GetOptionalInt(env, jVtxoRefreshExpiryThreshold).value_or(0));
    config.fallback_fee_rate =
        static_cast<uint64_t>(GetOptionalLong(env, jFallbackFeeRate).value_or(0));
    config.htlc_recv_claim_delta =
        static_cast<uint16_t>(GetOptionalInt(env, jHtlcRecvClaimDelta).value_or(0));
    config.vtxo_exit_margin = static_cast<uint16_t>(GetOptionalInt(env, jVtxoExitMargin).value_or(0));
    config.round_tx_required_confirmations =
        static_cast<uint32_t>(GetOptionalInt(env, jRoundTxRequiredConfirmations).value_or(0));

    opts.config = config;

    __android_log_print(
        ANDROID_LOG_INFO,
        LOG_TAG,
        "load_wallet(native) datadir=%s regtest=%s signet=%s bitcoin=%s birthday_height=%s ark=%s "
        "esplora=%s bitcoind=%s",
        datadir.c_str(),
        opts.regtest ? "true" : "false",
        opts.signet ? "true" : "false",
        opts.bitcoin ? "true" : "false",
        opts.birthday_height != nullptr ? std::to_string(*opts.birthday_height).c_str() : "null",
        config.ark.c_str(),
        config.esplora.c_str(),
        config.bitcoind.c_str());

    bark_cxx::load_wallet(datadir, opts);
    __android_log_print(ANDROID_LOG_INFO, LOG_TAG, "load_wallet(native) success");
  } catch (const std::exception& e) {
    HandleException(env, e);
  } catch (...) {
    HandleUnknownException(env);
  }
}

JNIEXPORT void JNICALL
Java_com_margelo_nitro_nitroark_NitroArkNative_maintenanceRefresh(JNIEnv* env, jobject /*thiz*/) {
  try {
    bark_cxx::maintenance_refresh();
  } catch (const std::exception& e) {
    HandleException(env, e);
  } catch (...) {
    HandleUnknownException(env);
  }
}

JNIEXPORT void JNICALL
Java_com_margelo_nitro_nitroark_NitroArkNative_tryClaimLightningReceive(JNIEnv* env, jobject /*thiz*/,
                                                                       jstring jPaymentHash, jboolean jWait,
                                                                       jstring jToken) {
  try {
    const std::string payment_hash = JStringToString(env, jPaymentHash);
    const std::string token_str = JStringToString(env, jToken);

    rust::String payment_hash_rs(payment_hash);
    rust::String token_rs(token_str);
    const rust::String* token_ptr = token_str.empty() ? nullptr : &token_rs;

    bark_cxx::try_claim_lightning_receive(payment_hash_rs, jWait == JNI_TRUE, token_ptr);
  } catch (const std::exception& e) {
    HandleException(env, e);
  } catch (...) {
    HandleUnknownException(env);
  }
}

JNIEXPORT jstring JNICALL
Java_com_margelo_nitro_nitroark_NitroArkNative_offboardAll(JNIEnv* env, jobject /*thiz*/,
                                                          jstring jDestination) {
  try {
    const std::string destination = JStringToString(env, jDestination);
    bark_cxx::RoundStatus status = bark_cxx::offboard_all(destination);
    std::string json = RoundStatusToJson(status);
    return env->NewStringUTF(json.c_str());
  } catch (const std::exception& e) {
    HandleException(env, e);
    return nullptr;
  } catch (...) {
    HandleUnknownException(env);
    return nullptr;
  }
}

JNIEXPORT jstring JNICALL
Java_com_margelo_nitro_nitroark_NitroArkNative_peakKeyPair(JNIEnv* env, jobject /*thiz*/, jint jIndex) {
  try {
    bark_cxx::KeyPairResult keypair = bark_cxx::peak_keypair(static_cast<uint32_t>(jIndex));
    std::string json = KeyPairToJson(keypair);
    return env->NewStringUTF(json.c_str());
  } catch (const std::exception& e) {
    HandleException(env, e);
    return nullptr;
  } catch (...) {
    HandleUnknownException(env);
    return nullptr;
  }
}

JNIEXPORT jboolean JNICALL
Java_com_margelo_nitro_nitroark_NitroArkNative_verifyMessage(JNIEnv* env, jobject /*thiz*/,
                                                            jstring jMessage, jstring jSignature,
                                                            jstring jPublicKey) {
  try {
    const std::string message = JStringToString(env, jMessage);
    const std::string signature = JStringToString(env, jSignature);
    const std::string publicKey = JStringToString(env, jPublicKey);
    return bark_cxx::verify_message(message, signature, publicKey);
  } catch (const std::exception& e) {
    HandleException(env, e);
    return JNI_FALSE;
  } catch (...) {
    HandleUnknownException(env);
    return JNI_FALSE;
  }
}

JNIEXPORT jstring JNICALL
Java_com_margelo_nitro_nitroark_NitroArkNative_bolt11Invoice(JNIEnv* env, jobject /*thiz*/, jlong jAmountMsat) {
  try {
    bark_cxx::Bolt11Invoice invoice = bark_cxx::bolt11_invoice(static_cast<uint64_t>(jAmountMsat));
    std::string json = Bolt11InvoiceToJson(invoice);
    return env->NewStringUTF(json.c_str());
  } catch (const std::exception& e) {
    HandleException(env, e);
    return nullptr;
  } catch (...) {
    HandleUnknownException(env);
    return nullptr;
  }
}

} // extern "C"
