#include <android/log.h>
#include <cstdint>
#include <exception>
#include <jni.h>
#include <optional>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <vector>

#include "generated/ark_cxx.h"

namespace {

constexpr const char* LOG_TAG = "NitroArkJni";

// Convert a jstring to a std::string, handling null safely.
std::string JStringToString(JNIEnv* env, jstring jStr) {
  if (jStr == nullptr) {
    return std::string();
  }
  const char* chars = env->GetStringUTFChars(jStr, nullptr);
  if (chars == nullptr) {
    return std::string();
  }
  std::string result(chars);
  env->ReleaseStringUTFChars(jStr, chars);
  return result;
}

void ThrowJavaException(JNIEnv* env, const char* message) {
  jclass exClass = env->FindClass("java/lang/RuntimeException");
  if (exClass != nullptr) {
    env->ThrowNew(exClass, message);
    env->DeleteLocalRef(exClass);
  }
  __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, "Throwing Java exception: %s", message);
}

template <typename T, typename J>
std::optional<T> GetOptionalNumber(JNIEnv* env, jobject obj, const char* className, const char* methodName,
                                   const char* methodSig) {
  if (obj == nullptr) {
    return std::nullopt;
  }
  jclass cls = env->FindClass(className);
  if (cls == nullptr) {
    return std::nullopt;
  }

  std::optional<T> result = std::nullopt;
  jmethodID mid = env->GetMethodID(cls, methodName, methodSig);
  if (mid != nullptr) {
    if constexpr (std::is_same_v<J, jint>) {
      jint value = env->CallIntMethod(obj, mid);
      result = static_cast<T>(value);
    } else if constexpr (std::is_same_v<J, jlong>) {
      jlong value = env->CallLongMethod(obj, mid);
      result = static_cast<T>(value);
    }
  }

  env->DeleteLocalRef(cls);
  return result;
}

std::optional<int32_t> GetOptionalInt(JNIEnv* env, jobject obj) {
  return GetOptionalNumber<int32_t, jint>(env, obj, "java/lang/Integer", "intValue", "()I");
}

std::optional<int64_t> GetOptionalLong(JNIEnv* env, jobject obj) {
  return GetOptionalNumber<int64_t, jlong>(env, obj, "java/lang/Long", "longValue", "()J");
}

void HandleException(JNIEnv* env, const std::exception& e) {
  __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, "Native exception: %s", e.what());
  ThrowJavaException(env, e.what());
}

void HandleUnknownException(JNIEnv* env) {
  __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, "Unknown exception in NitroArk native call.");
  ThrowJavaException(env, "Unknown exception in NitroArk native call.");
}

// Helpers to construct Java/Kotlin objects for return values.
jobject MakeArrayList(JNIEnv* env, const std::vector<std::string>& elements) {
  jclass arrayListClass = env->FindClass("java/util/ArrayList");
  if (arrayListClass == nullptr)
    return nullptr;
  jmethodID arrayListCtor = env->GetMethodID(arrayListClass, "<init>", "()V");
  jmethodID arrayListAdd = env->GetMethodID(arrayListClass, "add", "(Ljava/lang/Object;)Z");
  if (arrayListCtor == nullptr || arrayListAdd == nullptr) {
    env->DeleteLocalRef(arrayListClass);
    return nullptr;
  }

  jobject arrayListObj = env->NewObject(arrayListClass, arrayListCtor);
  if (arrayListObj == nullptr) {
    env->DeleteLocalRef(arrayListClass);
    return nullptr;
  }

  for (const auto& element : elements) {
    jstring jStr = env->NewStringUTF(element.c_str());
    if (jStr != nullptr) {
      env->CallBooleanMethod(arrayListObj, arrayListAdd, jStr);
      env->DeleteLocalRef(jStr);
    }
  }
  env->DeleteLocalRef(arrayListClass);
  return arrayListObj;
}

jobject MakeRoundStatusResult(JNIEnv* env, const bark_cxx::RoundStatus& status) {
  jclass cls = env->FindClass("com/margelo/nitro/nitroark/RoundStatusResult");
  if (cls == nullptr)
    return nullptr;
  jmethodID ctor =
      env->GetMethodID(cls, "<init>", "(Ljava/lang/String;Ljava/lang/String;Ljava/util/List;Ljava/lang/String;ZZ)V");
  if (ctor == nullptr) {
    env->DeleteLocalRef(cls);
    return nullptr;
  }

  std::string statusStr(status.status.data(), status.status.length());
  std::string fundingTxid(status.funding_txid.data(), status.funding_txid.length());
  std::string error(status.error.data(), status.error.length());

  // Convert unsigned txids
  std::vector<std::string> txids;
  txids.reserve(status.unsigned_funding_txids.size());
  for (const auto& tx : status.unsigned_funding_txids) {
    txids.emplace_back(std::string(tx.data(), tx.length()));
  }
  jobject txidList = MakeArrayList(env, txids);

  jstring jStatus = env->NewStringUTF(statusStr.c_str());
  jstring jFundingTxid = fundingTxid.empty() ? nullptr : env->NewStringUTF(fundingTxid.c_str());
  jstring jError = error.empty() ? nullptr : env->NewStringUTF(error.c_str());

  jobject result =
      env->NewObject(cls, ctor, jStatus, jFundingTxid, txidList, jError, status.is_final, status.is_success);

  if (jStatus)
    env->DeleteLocalRef(jStatus);
  if (jFundingTxid)
    env->DeleteLocalRef(jFundingTxid);
  if (jError)
    env->DeleteLocalRef(jError);
  if (txidList)
    env->DeleteLocalRef(txidList);
  env->DeleteLocalRef(cls);
  return result;
}

jobject MakeKeyPairResult(JNIEnv* env, const bark_cxx::KeyPairResult& keypair) {
  jclass cls = env->FindClass("com/margelo/nitro/nitroark/KeyPairResultAndroid");
  if (cls == nullptr)
    return nullptr;
  jmethodID ctor = env->GetMethodID(cls, "<init>", "(Ljava/lang/String;Ljava/lang/String;)V");
  if (ctor == nullptr) {
    env->DeleteLocalRef(cls);
    return nullptr;
  }

  std::string pub(keypair.public_key.data(), keypair.public_key.length());
  std::string sec(keypair.secret_key.data(), keypair.secret_key.length());

  jstring jPub = env->NewStringUTF(pub.c_str());
  jstring jSec = env->NewStringUTF(sec.c_str());

  jobject result = env->NewObject(cls, ctor, jPub, jSec);

  if (jPub)
    env->DeleteLocalRef(jPub);
  if (jSec)
    env->DeleteLocalRef(jSec);
  env->DeleteLocalRef(cls);
  return result;
}

jobject MakeBolt11Invoice(JNIEnv* env, const bark_cxx::Bolt11Invoice& invoice) {
  jclass cls = env->FindClass("com/margelo/nitro/nitroark/Bolt11InvoiceResult");
  if (cls == nullptr)
    return nullptr;
  jmethodID ctor = env->GetMethodID(cls, "<init>", "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V");
  if (ctor == nullptr) {
    env->DeleteLocalRef(cls);
    return nullptr;
  }

  std::string bolt11(invoice.bolt11_invoice.data(), invoice.bolt11_invoice.length());
  std::string paymentSecret(invoice.payment_secret.data(), invoice.payment_secret.length());
  std::string paymentHash(invoice.payment_hash.data(), invoice.payment_hash.length());

  jstring jBolt11 = env->NewStringUTF(bolt11.c_str());
  jstring jSecret = env->NewStringUTF(paymentSecret.c_str());
  jstring jHash = env->NewStringUTF(paymentHash.c_str());

  jobject result = env->NewObject(cls, ctor, jBolt11, jSecret, jHash);

  if (jBolt11)
    env->DeleteLocalRef(jBolt11);
  if (jSecret)
    env->DeleteLocalRef(jSecret);
  if (jHash)
    env->DeleteLocalRef(jHash);
  env->DeleteLocalRef(cls);
  return result;
}

} // namespace

extern "C" {

JNIEXPORT jboolean JNICALL Java_com_margelo_nitro_nitroark_NitroArkNative_isWalletLoaded(JNIEnv* env,
                                                                                         jobject /*thiz*/) {
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

JNIEXPORT void JNICALL Java_com_margelo_nitro_nitroark_NitroArkNative_closeWallet(JNIEnv* env, jobject /*thiz*/) {
  try {
    bark_cxx::close_wallet();
  } catch (const std::exception& e) {
    HandleException(env, e);
  } catch (...) {
    HandleUnknownException(env);
  }
}

JNIEXPORT void JNICALL Java_com_margelo_nitro_nitroark_NitroArkNative_loadWalletNative(
    JNIEnv* env, jobject /*thiz*/, jstring jDatadir, jstring jMnemonic, jboolean jRegtest, jboolean jSignet,
    jboolean jBitcoin, jobject jBirthdayHeight, jstring jArk, jstring jEsplora, jstring jBitcoind,
    jstring jBitcoindCookie, jstring jBitcoindUser, jstring jBitcoindPass, jobject jVtxoRefreshExpiryThreshold,
    jobject jFallbackFeeRate, jobject jHtlcRecvClaimDelta, jobject jVtxoExitMargin,
    jobject jRoundTxRequiredConfirmations) {
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
    config.fallback_fee_rate = static_cast<uint64_t>(GetOptionalLong(env, jFallbackFeeRate).value_or(0));
    config.htlc_recv_claim_delta = static_cast<uint16_t>(GetOptionalInt(env, jHtlcRecvClaimDelta).value_or(0));
    config.vtxo_exit_margin = static_cast<uint16_t>(GetOptionalInt(env, jVtxoExitMargin).value_or(0));
    config.round_tx_required_confirmations =
        static_cast<uint32_t>(GetOptionalInt(env, jRoundTxRequiredConfirmations).value_or(0));

    opts.config = config;

    std::string birthday_height_str = opts.birthday_height != nullptr ? std::to_string(*opts.birthday_height) : "null";
    __android_log_print(ANDROID_LOG_INFO, LOG_TAG,
                        "load_wallet(native) datadir=%s regtest=%s signet=%s bitcoin=%s birthday_height=%s ark=%s "
                        "esplora=%s bitcoind=%s",
                        datadir.c_str(), opts.regtest ? "true" : "false", opts.signet ? "true" : "false",
                        opts.bitcoin ? "true" : "false", birthday_height_str.c_str(), config.ark.c_str(),
                        config.esplora.c_str(), config.bitcoind.c_str());

    bark_cxx::load_wallet(datadir, opts);
    __android_log_print(ANDROID_LOG_INFO, LOG_TAG, "load_wallet(native) success");
  } catch (const std::exception& e) {
    HandleException(env, e);
  } catch (...) {
    HandleUnknownException(env);
  }
}

JNIEXPORT void JNICALL Java_com_margelo_nitro_nitroark_NitroArkNative_maintenance(JNIEnv* env, jobject /*thiz*/) {
  try {
    bark_cxx::maintenance();
  } catch (const std::exception& e) {
    HandleException(env, e);
  } catch (...) {
    HandleUnknownException(env);
  }
}

JNIEXPORT void JNICALL Java_com_margelo_nitro_nitroark_NitroArkNative_maintenanceRefresh(JNIEnv* env,
                                                                                         jobject /*thiz*/) {
  try {
    bark_cxx::maintenance_refresh();
  } catch (const std::exception& e) {
    HandleException(env, e);
  } catch (...) {
    HandleUnknownException(env);
  }
}

JNIEXPORT void JNICALL Java_com_margelo_nitro_nitroark_NitroArkNative_tryClaimLightningReceive(
    JNIEnv* env, jobject /*thiz*/, jstring jPaymentHash, jboolean jWait, jstring jToken) {
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

JNIEXPORT jobject JNICALL Java_com_margelo_nitro_nitroark_NitroArkNative_offboardAll(JNIEnv* env, jobject /*thiz*/,
                                                                                     jstring jDestination) {
  try {
    const std::string destination = JStringToString(env, jDestination);
    bark_cxx::RoundStatus status = bark_cxx::offboard_all(destination);
    return MakeRoundStatusResult(env, status);
  } catch (const std::exception& e) {
    HandleException(env, e);
    return nullptr;
  } catch (...) {
    HandleUnknownException(env);
    return nullptr;
  }
}

JNIEXPORT jobject JNICALL Java_com_margelo_nitro_nitroark_NitroArkNative_peakKeyPair(JNIEnv* env, jobject /*thiz*/,
                                                                                     jint jIndex) {
  try {
    bark_cxx::KeyPairResult keypair = bark_cxx::peak_keypair(static_cast<uint32_t>(jIndex));
    return MakeKeyPairResult(env, keypair);
  } catch (const std::exception& e) {
    HandleException(env, e);
    return nullptr;
  } catch (...) {
    HandleUnknownException(env);
    return nullptr;
  }
}

JNIEXPORT jboolean JNICALL Java_com_margelo_nitro_nitroark_NitroArkNative_verifyMessage(JNIEnv* env, jobject /*thiz*/,
                                                                                        jstring jMessage,
                                                                                        jstring jSignature,
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

JNIEXPORT jobject JNICALL Java_com_margelo_nitro_nitroark_NitroArkNative_bolt11Invoice(JNIEnv* env, jobject /*thiz*/,
                                                                                       jlong jAmountMsat) {
  try {
    bark_cxx::Bolt11Invoice invoice = bark_cxx::bolt11_invoice(static_cast<uint64_t>(jAmountMsat));
    return MakeBolt11Invoice(env, invoice);
  } catch (const std::exception& e) {
    HandleException(env, e);
    return nullptr;
  } catch (...) {
    HandleUnknownException(env);
    return nullptr;
  }
}

JNIEXPORT jstring JNICALL Java_com_margelo_nitro_nitroark_NitroArkNative_signMessage(JNIEnv* env, jobject /*thiz*/,
                                                                                     jstring jMessage, jint jIndex) {
  try {
    const std::string message = JStringToString(env, jMessage);
    rust::String signature = bark_cxx::sign_message(message, static_cast<uint32_t>(jIndex));
    std::string signatureStr(signature.data(), signature.length());
    return env->NewStringUTF(signatureStr.c_str());
  } catch (const std::exception& e) {
    HandleException(env, e);
    return nullptr;
  } catch (...) {
    HandleUnknownException(env);
    return nullptr;
  }
}

JNIEXPORT void JNICALL Java_com_margelo_nitro_nitroark_NitroArkNative_sync(JNIEnv* env, jobject /*thiz*/) {
  try {
    bark_cxx::sync();
  } catch (const std::exception& e) {
    HandleException(env, e);
  } catch (...) {
    HandleUnknownException(env);
  }
}

} // extern "C"
