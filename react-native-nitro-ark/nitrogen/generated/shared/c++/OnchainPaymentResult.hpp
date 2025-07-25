///
/// OnchainPaymentResult.hpp
/// This file was generated by nitrogen. DO NOT MODIFY THIS FILE.
/// https://github.com/mrousavy/nitro
/// Copyright © 2025 Marc Rousavy @ Margelo
///

#pragma once

#if __has_include(<NitroModules/JSIConverter.hpp>)
#include <NitroModules/JSIConverter.hpp>
#else
#error NitroModules cannot be found! Are you sure you installed NitroModules properly?
#endif
#if __has_include(<NitroModules/NitroDefines.hpp>)
#include <NitroModules/NitroDefines.hpp>
#else
#error NitroModules cannot be found! Are you sure you installed NitroModules properly?
#endif

// Forward declaration of `PaymentTypes` to properly resolve imports.
namespace margelo::nitro::nitroark { enum class PaymentTypes; }

#include <string>
#include "PaymentTypes.hpp"

namespace margelo::nitro::nitroark {

  /**
   * A struct which can be represented as a JavaScript object (OnchainPaymentResult).
   */
  struct OnchainPaymentResult {
  public:
    std::string txid     SWIFT_PRIVATE;
    double amount_sat     SWIFT_PRIVATE;
    std::string destination_address     SWIFT_PRIVATE;
    PaymentTypes payment_type     SWIFT_PRIVATE;

  public:
    OnchainPaymentResult() = default;
    explicit OnchainPaymentResult(std::string txid, double amount_sat, std::string destination_address, PaymentTypes payment_type): txid(txid), amount_sat(amount_sat), destination_address(destination_address), payment_type(payment_type) {}
  };

} // namespace margelo::nitro::nitroark

namespace margelo::nitro {

  using namespace margelo::nitro::nitroark;

  // C++ OnchainPaymentResult <> JS OnchainPaymentResult (object)
  template <>
  struct JSIConverter<OnchainPaymentResult> final {
    static inline OnchainPaymentResult fromJSI(jsi::Runtime& runtime, const jsi::Value& arg) {
      jsi::Object obj = arg.asObject(runtime);
      return OnchainPaymentResult(
        JSIConverter<std::string>::fromJSI(runtime, obj.getProperty(runtime, "txid")),
        JSIConverter<double>::fromJSI(runtime, obj.getProperty(runtime, "amount_sat")),
        JSIConverter<std::string>::fromJSI(runtime, obj.getProperty(runtime, "destination_address")),
        JSIConverter<PaymentTypes>::fromJSI(runtime, obj.getProperty(runtime, "payment_type"))
      );
    }
    static inline jsi::Value toJSI(jsi::Runtime& runtime, const OnchainPaymentResult& arg) {
      jsi::Object obj(runtime);
      obj.setProperty(runtime, "txid", JSIConverter<std::string>::toJSI(runtime, arg.txid));
      obj.setProperty(runtime, "amount_sat", JSIConverter<double>::toJSI(runtime, arg.amount_sat));
      obj.setProperty(runtime, "destination_address", JSIConverter<std::string>::toJSI(runtime, arg.destination_address));
      obj.setProperty(runtime, "payment_type", JSIConverter<PaymentTypes>::toJSI(runtime, arg.payment_type));
      return obj;
    }
    static inline bool canConvert(jsi::Runtime& runtime, const jsi::Value& value) {
      if (!value.isObject()) {
        return false;
      }
      jsi::Object obj = value.getObject(runtime);
      if (!JSIConverter<std::string>::canConvert(runtime, obj.getProperty(runtime, "txid"))) return false;
      if (!JSIConverter<double>::canConvert(runtime, obj.getProperty(runtime, "amount_sat"))) return false;
      if (!JSIConverter<std::string>::canConvert(runtime, obj.getProperty(runtime, "destination_address"))) return false;
      if (!JSIConverter<PaymentTypes>::canConvert(runtime, obj.getProperty(runtime, "payment_type"))) return false;
      return true;
    }
  };

} // namespace margelo::nitro
