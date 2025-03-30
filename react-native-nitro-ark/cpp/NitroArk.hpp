#pragma once

#include "HybridNitroArkSpec.hpp"
#include "bark-cpp.h"
#include <memory>
#include <string>

namespace margelo::nitro::nitroark {

  class NitroArk : public HybridNitroArkSpec {
  public:
    NitroArk() : HybridObject(TAG) {}

    // Original multiply function
    double multiply(double a, double b) override {
      return a * b * 2;
    }

    std::shared_ptr<Promise<bool>> createWallet(const std::string &datadir, 
                                              const BarkCreateOpts &opts) override {
      return Promise<bool>::async([datadir, opts]() {
        // Convert from our JS interface to the C API structure
        bark::bark_BarkConfigOpts config = {
            opts.config.has_value() && opts.config->asp.has_value() ? opts.config->asp->c_str() : nullptr,
            opts.config.has_value() && opts.config->esplora.has_value() ? opts.config->esplora->c_str() : nullptr,
            opts.config.has_value() && opts.config->bitcoind.has_value() ? opts.config->bitcoind->c_str() : nullptr,
            opts.config.has_value() && opts.config->bitcoind_cookie.has_value() ? opts.config->bitcoind_cookie->c_str() : nullptr,
            opts.config.has_value() && opts.config->bitcoind_user.has_value() ? opts.config->bitcoind_user->c_str() : nullptr,
            opts.config.has_value() && opts.config->bitcoind_pass.has_value() ? opts.config->bitcoind_pass->c_str() : nullptr
        };
        
        bark::bark_BarkCreateOpts barkOpts = {
            opts.force.value_or(false),
            opts.regtest.value_or(false),
            opts.signet.value_or(false),
            opts.bitcoin.value_or(false),
            opts.mnemonic.has_value() ? opts.mnemonic->c_str() : nullptr,
            opts.birthday_height.has_value() ? static_cast<uint64_t>(opts.birthday_height.value()) : 0,
            config
        };
        
        // Call the C API function
        bark::bark_BarkError *error = bark::bark_create_wallet(datadir.c_str(), barkOpts);
        
        // Check if there was an error
        bool success = (error == nullptr);
        
        // If there was an error, free it
        if (error) {
            bark::bark_free_error(error);
        }
        
        return success;
      });
    }

    std::shared_ptr<Promise<BarkBalance>> getBalance(const std::string &datadir, 
                                                  bool no_sync) override {
      return Promise<BarkBalance>::async([datadir, no_sync]() {
        // Create a struct to store the balance
        bark::bark_BarkBalance balance;
        
        // Call the C API function
        bark::bark_BarkError *error = bark::bark_get_balance(datadir.c_str(), no_sync, &balance);
        
        // Check if there was an error
        if (error) {
            // Free the error
            bark::bark_free_error(error);
            
            // Return a default balance with zeros
            return BarkBalance(0, 0, 0);
        }
        
        // Convert the C API balance to our JS interface balance
        return BarkBalance(
            static_cast<double>(balance.onchain),
            static_cast<double>(balance.offchain),
            static_cast<double>(balance.pending_exit)
        );
      });
    }
  };

} // namespace margelo::nitro::nitroark