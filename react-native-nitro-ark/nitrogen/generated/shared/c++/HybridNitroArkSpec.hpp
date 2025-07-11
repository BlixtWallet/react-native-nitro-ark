///
/// HybridNitroArkSpec.hpp
/// This file was generated by nitrogen. DO NOT MODIFY THIS FILE.
/// https://github.com/mrousavy/nitro
/// Copyright © 2025 Marc Rousavy @ Margelo
///

#pragma once

#if __has_include(<NitroModules/HybridObject.hpp>)
#include <NitroModules/HybridObject.hpp>
#else
#error NitroModules cannot be found! Are you sure you installed NitroModules properly?
#endif

// Forward declaration of `BarkCreateOpts` to properly resolve imports.
namespace margelo::nitro::nitroark { struct BarkCreateOpts; }
// Forward declaration of `BarkBalance` to properly resolve imports.
namespace margelo::nitro::nitroark { struct BarkBalance; }
// Forward declaration of `BarkSendManyOutput` to properly resolve imports.
namespace margelo::nitro::nitroark { struct BarkSendManyOutput; }
// Forward declaration of `BarkRefreshOpts` to properly resolve imports.
namespace margelo::nitro::nitroark { struct BarkRefreshOpts; }

#include <NitroModules/Promise.hpp>
#include <string>
#include "BarkCreateOpts.hpp"
#include "BarkBalance.hpp"
#include <optional>
#include <vector>
#include "BarkSendManyOutput.hpp"
#include "BarkRefreshOpts.hpp"

namespace margelo::nitro::nitroark {

  using namespace margelo::nitro;

  /**
   * An abstract base class for `NitroArk`
   * Inherit this class to create instances of `HybridNitroArkSpec` in C++.
   * You must explicitly call `HybridObject`'s constructor yourself, because it is virtual.
   * @example
   * ```cpp
   * class HybridNitroArk: public HybridNitroArkSpec {
   * public:
   *   HybridNitroArk(...): HybridObject(TAG) { ... }
   *   // ...
   * };
   * ```
   */
  class HybridNitroArkSpec: public virtual HybridObject {
    public:
      // Constructor
      explicit HybridNitroArkSpec(): HybridObject(TAG) { }

      // Destructor
      ~HybridNitroArkSpec() override = default;

    public:
      // Properties
      

    public:
      // Methods
      virtual std::shared_ptr<Promise<std::string>> createMnemonic() = 0;
      virtual std::shared_ptr<Promise<void>> loadWallet(const std::string& datadir, const BarkCreateOpts& opts) = 0;
      virtual std::shared_ptr<Promise<void>> closeWallet() = 0;
      virtual std::shared_ptr<Promise<bool>> isWalletLoaded() = 0;
      virtual std::shared_ptr<Promise<BarkBalance>> getBalance(bool no_sync) = 0;
      virtual std::shared_ptr<Promise<std::string>> getOnchainAddress() = 0;
      virtual std::shared_ptr<Promise<std::string>> getOnchainUtxos(bool no_sync) = 0;
      virtual std::shared_ptr<Promise<std::string>> getVtxoPubkey(std::optional<double> index) = 0;
      virtual std::shared_ptr<Promise<std::string>> getVtxos(bool no_sync) = 0;
      virtual std::shared_ptr<Promise<std::string>> sendOnchain(const std::string& destination, double amountSat, bool no_sync) = 0;
      virtual std::shared_ptr<Promise<std::string>> drainOnchain(const std::string& destination, bool no_sync) = 0;
      virtual std::shared_ptr<Promise<std::string>> sendManyOnchain(const std::vector<BarkSendManyOutput>& outputs, bool no_sync) = 0;
      virtual std::shared_ptr<Promise<std::string>> refreshVtxos(const BarkRefreshOpts& refreshOpts, bool no_sync) = 0;
      virtual std::shared_ptr<Promise<std::string>> boardAmount(double amountSat, bool no_sync) = 0;
      virtual std::shared_ptr<Promise<std::string>> boardAll(bool no_sync) = 0;
      virtual std::shared_ptr<Promise<std::string>> send(const std::string& destination, std::optional<double> amountSat, const std::optional<std::string>& comment, bool no_sync) = 0;
      virtual std::shared_ptr<Promise<std::string>> sendRoundOnchain(const std::string& destination, double amountSat, bool no_sync) = 0;
      virtual std::shared_ptr<Promise<std::string>> bolt11Invoice(double amountMsat) = 0;
      virtual std::shared_ptr<Promise<void>> claimBolt11Payment(const std::string& bolt11) = 0;
      virtual std::shared_ptr<Promise<std::string>> offboardSpecific(const std::vector<std::string>& vtxoIds, const std::optional<std::string>& optionalAddress, bool no_sync) = 0;
      virtual std::shared_ptr<Promise<std::string>> offboardAll(const std::optional<std::string>& optionalAddress, bool no_sync) = 0;
      virtual std::shared_ptr<Promise<std::string>> exitStartSpecific(const std::vector<std::string>& vtxoIds) = 0;
      virtual std::shared_ptr<Promise<std::string>> exitStartAll() = 0;
      virtual std::shared_ptr<Promise<std::string>> exitProgressOnce() = 0;

    protected:
      // Hybrid Setup
      void loadHybridMethods() override;

    protected:
      // Tag for logging
      static constexpr auto TAG = "NitroArk";
  };

} // namespace margelo::nitro::nitroark
