#pragma once

#include "HybridNitroArkSpec.hpp"
#include "bark-cpp.h"
#include <memory>
#include <stdexcept>
#include <string>
#include <vector>

namespace margelo::nitro::nitroark
{

  // Helper function to handle potential errors from bark-cpp calls
  inline void check_bark_error(bark::bark_BarkError *error)
  {
    if (error != nullptr)
    {
      std::string error_message = "Bark-cpp error: Unknown";
      if (error->message != nullptr)
      {
        // Assuming error->message is valid C string allocated correctly
        error_message = std::string("Bark-cpp error: ") + error->message;
      }
      // Use the FFI function to free the error struct and its contents
      bark::bark_free_error(error);
      throw std::runtime_error(error_message);
    }
  }

  class NitroArk : public HybridNitroArkSpec
  {
  public:
    NitroArk() : HybridObject(TAG)
    {
      // Initialize the Rust logger once when a NitroArk object is created.
      bark::bark_init_logger();
    }

    // --- Management ---

    std::shared_ptr<Promise<std::string>> createMnemonic() override
    {
      return Promise<std::string>::async([]()
                                         {
      char *mnemonic_c = bark::bark_create_mnemonic();
      if (mnemonic_c == nullptr) {
        throw std::runtime_error(
            "Bark-cpp error: Failed to create mnemonic (returned NULL)");
      }
      std::string mnemonic_str(mnemonic_c);
      bark::bark_free_string(mnemonic_c);
      return mnemonic_str; });
    }

    std::shared_ptr<Promise<void>>
    createWallet(const std::string &datadir,
                 const BarkCreateOpts &opts) override
    {
      return Promise<void>::async([datadir, opts]()
                                  {
      // Keep fee rate value alive for the C call
      std::optional<uint64_t> fallback_fee_rate_val;
      if (opts.config.has_value() && opts.config->fallback_fee_rate.has_value()) {
          fallback_fee_rate_val = static_cast<uint64_t>(opts.config->fallback_fee_rate.value());
      }

      bark::bark_BarkConfigOpts config = {
          opts.config.has_value() && opts.config->asp.has_value()
              ? opts.config->asp->c_str()
              : nullptr,
          opts.config.has_value() && opts.config->esplora.has_value()
              ? opts.config->esplora->c_str()
              : nullptr,
          opts.config.has_value() && opts.config->bitcoind.has_value()
              ? opts.config->bitcoind->c_str()
              : nullptr,
          opts.config.has_value() && opts.config->bitcoind_cookie.has_value()
              ? opts.config->bitcoind_cookie->c_str()
              : nullptr,
          opts.config.has_value() && opts.config->bitcoind_user.has_value()
              ? opts.config->bitcoind_user->c_str()
              : nullptr,
          opts.config.has_value() && opts.config->bitcoind_pass.has_value()
              ? opts.config->bitcoind_pass->c_str()
              : nullptr,
          opts.config.has_value() &&
                  opts.config->vtxo_refresh_expiry_threshold.has_value()
              ? static_cast<uint32_t>(
                    opts.config->vtxo_refresh_expiry_threshold.value())
              : 0,
          fallback_fee_rate_val.has_value() ? &fallback_fee_rate_val.value()
                                            : nullptr
      };

      bark::bark_BarkCreateOpts barkOpts = {
          opts.force.value_or(false),
          opts.regtest.value_or(false),
          opts.signet.value_or(false),
          opts.bitcoin.value_or(true),
          opts.mnemonic.empty() ? nullptr : opts.mnemonic.c_str(),
          opts.birthday_height.has_value()
              ? static_cast<uint32_t>(opts.birthday_height.value())
              : 0,
          config};

      bark::bark_BarkError *error =
          bark::bark_create_wallet(datadir.c_str(), barkOpts);
      check_bark_error(error); });
    }

    // --- Wallet Info ---

    std::shared_ptr<Promise<BarkBalance>>
    getBalance(const std::string &datadir, bool no_sync,
               const std::string &mnemonic) override
    {
      return Promise<BarkBalance>::async([datadir, no_sync, mnemonic]()
                                         {
      bark::bark_BarkBalance c_balance;
      bark::bark_BarkError *error = bark::bark_get_balance(
          datadir.c_str(), no_sync, mnemonic.c_str(), &c_balance);
      check_bark_error(error);

      return BarkBalance(static_cast<double>(c_balance.onchain),
                         static_cast<double>(c_balance.offchain),
                         static_cast<double>(c_balance.pending_exit)); });
    }

    std::shared_ptr<Promise<std::string>>
    getOnchainAddress(const std::string &datadir,
                      const std::string &mnemonic) override
    {
      return Promise<std::string>::async([datadir, mnemonic]()
                                         {
      char *address_c = nullptr;
      bark::bark_BarkError *error = bark::bark_get_onchain_address(
          datadir.c_str(), mnemonic.c_str(), &address_c);
      check_bark_error(error);
      if (address_c == nullptr) {
        throw std::runtime_error("Bark-cpp error: getOnchainAddress returned "
                                 "success but address is null");
      }
      std::string address_str(address_c);
      bark::bark_free_string(address_c); // Use helper
      return address_str; });
    }

    std::shared_ptr<Promise<std::string>>
    getOnchainUtxos(const std::string &datadir, const std::string &mnemonic,
                    bool no_sync) override
    {
      return Promise<std::string>::async([datadir, mnemonic, no_sync]()
                                         {
      char *json_c = nullptr;
      bark::bark_BarkError *error = bark::bark_get_onchain_utxos(
          datadir.c_str(), mnemonic.c_str(), no_sync, &json_c);
      check_bark_error(error);
      if (json_c == nullptr) {
        throw std::runtime_error("Bark-cpp error: getOnchainUtxos returned "
                                 "success but JSON is null");
      }
      std::string json_str(json_c);
      bark::bark_free_string(json_c); // Use helper
      return json_str; });
    }

    std::shared_ptr<Promise<std::string>>
    getVtxoPubkey(const std::string &datadir,
                  const std::string &mnemonic) override
    {
      return Promise<std::string>::async([datadir, mnemonic]()
                                         {
      char *pubkey_c = nullptr;
      bark::bark_BarkError *error = bark::bark_get_vtxo_pubkey(
          datadir.c_str(), mnemonic.c_str(), &pubkey_c);
      check_bark_error(error);
      if (pubkey_c == nullptr) {
        throw std::runtime_error("Bark-cpp error: getVtxoPubkey returned "
                                 "success but pubkey is null");
      }
      std::string pubkey_str(pubkey_c);
      bark::bark_free_string(pubkey_c); // Use helper
      return pubkey_str; });
    }

    std::shared_ptr<Promise<std::string>> getVtxos(const std::string &datadir,
                                                   const std::string &mnemonic,
                                                   bool no_sync) override
    {
      return Promise<std::string>::async([datadir, mnemonic, no_sync]()
                                         {
      char *json_c = nullptr;
      bark::bark_BarkError *error = bark::bark_get_vtxos(
          datadir.c_str(), mnemonic.c_str(), no_sync, &json_c);
      check_bark_error(error);
      if (json_c == nullptr) {
        throw std::runtime_error(
            "Bark-cpp error: getVtxos returned success but JSON is null");
      }
      std::string json_str(json_c);
      bark::bark_free_string(json_c); // Use helper
      return json_str; });
    }

    // --- Onchain Operations ---

    std::shared_ptr<Promise<std::string>>
    sendOnchain(const std::string &datadir, const std::string &mnemonic,
                const std::string &destination, double amountSat,
                bool no_sync) override
    {
      return Promise<std::string>::async([datadir, mnemonic, destination,
                                          amountSat, no_sync]()
                                         {
      char *txid_c = nullptr;
      bark::bark_BarkError *error = bark::bark_send_onchain(
          datadir.c_str(), mnemonic.c_str(), destination.c_str(),
          static_cast<uint64_t>(amountSat), no_sync, &txid_c);
      check_bark_error(error);
      if (txid_c == nullptr) {
        throw std::runtime_error(
            "Bark-cpp error: sendOnchain returned success but txid is null");
      }
      std::string txid_str(txid_c);
      bark::bark_free_string(txid_c); // Use helper
      return txid_str; });
    }

    std::shared_ptr<Promise<std::string>>
    drainOnchain(const std::string &datadir, const std::string &mnemonic,
                 const std::string &destination, bool no_sync) override
    {
      return Promise<std::string>::async(
          [datadir, mnemonic, destination, no_sync]()
          {
            char *txid_c = nullptr;
            bark::bark_BarkError *error =
                bark::bark_drain_onchain(datadir.c_str(), mnemonic.c_str(),
                                         destination.c_str(), no_sync, &txid_c);
            check_bark_error(error);
            if (txid_c == nullptr)
            {
              throw std::runtime_error("Bark-cpp error: drainOnchain returned "
                                       "success but txid is null");
            }
            std::string txid_str(txid_c);
            bark::bark_free_string(txid_c); // Use helper
            return txid_str;
          });
    }

    std::shared_ptr<Promise<std::string>>
    sendManyOnchain(const std::string &datadir, const std::string &mnemonic,
                    const std::vector<BarkSendManyOutput> &outputs,
                    bool no_sync) override
    {
      return Promise<std::string>::async([datadir, mnemonic, outputs, no_sync]()
                                         {
      size_t num_outputs = outputs.size();
      if (num_outputs == 0) {
        throw std::runtime_error(
            "sendManyOnchain requires at least one output");
      }

      std::vector<const char *> destinations_c;
      std::vector<uint64_t> amounts_c;
      destinations_c.reserve(num_outputs);
      amounts_c.reserve(num_outputs);

      for (const auto &output : outputs) {
        // --- FIX: Access directly, no .value() ---
        destinations_c.push_back(output.destination.c_str());
        amounts_c.push_back(static_cast<uint64_t>(output.amountSat));
        // --- End FIX ---
      }

      char *txid_c = nullptr;
      bark::bark_BarkError *error = bark::bark_send_many_onchain(
          datadir.c_str(), mnemonic.c_str(), destinations_c.data(),
          amounts_c.data(), num_outputs, no_sync, &txid_c);
      check_bark_error(error);
      if (txid_c == nullptr) {
        throw std::runtime_error("Bark-cpp error: sendManyOnchain returned "
                                 "success but txid is null");
      }
      std::string txid_str(txid_c);
      bark::bark_free_string(txid_c); // Use helper
      return txid_str; });
    }

    // --- Ark Operations ---

    std::shared_ptr<Promise<std::string>>
    refreshVtxos(const std::string &datadir, const std::string &mnemonic,
                 const BarkRefreshOpts &refreshOpts, bool no_sync) override
    {
      return Promise<std::string>::async([datadir, mnemonic, refreshOpts,
                                          no_sync]()
                                         {
      bark::bark_BarkRefreshOpts c_opts;
      std::vector<const char *> specific_ids_c; // Keep alive for the C call

      // Map the C++ enum to the C FFI enum using a switch
      switch (refreshOpts.mode_type) {
      case margelo::nitro::nitroark::BarkRefreshModeType::DEFAULTTHRESHOLD:
        c_opts.mode_type = bark::bark_BarkRefreshModeType::DefaultThreshold;
        break;
      case margelo::nitro::nitroark::BarkRefreshModeType::THRESHOLDBLOCKS:
        c_opts.mode_type = bark::bark_BarkRefreshModeType::ThresholdBlocks;
        break;
      case margelo::nitro::nitroark::BarkRefreshModeType::THRESHOLDHOURS:
        c_opts.mode_type = bark::bark_BarkRefreshModeType::ThresholdHours;
        break;
      case margelo::nitro::nitroark::BarkRefreshModeType::COUNTERPARTY:
        c_opts.mode_type = bark::bark_BarkRefreshModeType::Counterparty;
        break;
      case margelo::nitro::nitroark::BarkRefreshModeType::ALL:
        c_opts.mode_type = bark::bark_BarkRefreshModeType::All;
        break;
      case margelo::nitro::nitroark::BarkRefreshModeType::SPECIFIC:
        c_opts.mode_type = bark::bark_BarkRefreshModeType::Specific;
        break;
      default:
        // This should ideally not happen with a closed enum, but handle
        // defensively
        throw std::runtime_error(
            "Unknown BarkRefreshModeType encountered: " +
            std::to_string(static_cast<int>(refreshOpts.mode_type)));
      }

      // Assign threshold_value (handle optional)
      // Note: C struct expects uint32_t, C++ has optional<double>. Cast needed.
      c_opts.threshold_value =
          static_cast<uint32_t>(refreshOpts.threshold_value.value_or(0));

      // Handle specific_vtxo_ids only if mode is Specific
      if (c_opts.mode_type == bark::bark_BarkRefreshModeType::Specific) {
        if (!refreshOpts.specific_vtxo_ids.has_value() ||
            refreshOpts.specific_vtxo_ids->empty()) {
          throw std::runtime_error(
              "Specific refresh mode requires non-empty specific_vtxo_ids");
        }
        specific_ids_c.reserve(refreshOpts.specific_vtxo_ids->size());
        for (const auto &id : refreshOpts.specific_vtxo_ids.value()) {
          specific_ids_c.push_back(id.c_str()); // Get C string pointer
        }
        c_opts.specific_vtxo_ids =
            specific_ids_c.data(); // Point to the data in the vector
        c_opts.num_specific_vtxo_ids = specific_ids_c.size(); // Get the size
      } else {
        // Ensure these are null/zero if not in Specific mode
        c_opts.specific_vtxo_ids = nullptr;
        c_opts.num_specific_vtxo_ids = 0;
      }

      // Make the C FFI call
      char *status_c = nullptr;
      bark::bark_BarkError *error = bark::bark_refresh_vtxos(
          datadir.c_str(), mnemonic.c_str(), c_opts, no_sync, &status_c);

      check_bark_error(error);
      if (status_c == nullptr) {
        // Decide if null status is an error or just empty status
        // For consistency let's assume null should not happen on success.
        throw std::runtime_error("Bark-cpp error: refreshVtxos returned "
                                 "success but status pointer is null");
      }
      std::string status_str(status_c);
      bark::bark_free_string(status_c); // Use helper
      return status_str; });
    }

    std::shared_ptr<Promise<std::string>> boardAmount(const std::string &datadir,
                                                      const std::string &mnemonic,
                                                      double amountSat,
                                                      bool no_sync) override
    {
      return Promise<std::string>::async([datadir, mnemonic, amountSat,
                                          no_sync]()
                                         {
      char *status_c = nullptr;
      bark::bark_BarkError *error = bark::bark_board_amount(
          datadir.c_str(), mnemonic.c_str(), static_cast<uint64_t>(amountSat),
          no_sync, &status_c);
      check_bark_error(error);
      if (status_c == nullptr) {
        throw std::runtime_error(
            "Bark-cpp error: boardAmount returned success but status is null");
      }
      std::string status_str(status_c);
      bark::bark_free_string(status_c); // Use helper
      return status_str; });
    }

    std::shared_ptr<Promise<std::string>> boardAll(const std::string &datadir,
                                                   const std::string &mnemonic,
                                                   bool no_sync) override
    {
      return Promise<std::string>::async([datadir, mnemonic, no_sync]()
                                         {
      char *status_c = nullptr;
      bark::bark_BarkError *error = bark::bark_board_all(
          datadir.c_str(), mnemonic.c_str(), no_sync, &status_c);
      check_bark_error(error);
      if (status_c == nullptr) {
        throw std::runtime_error(
            "Bark-cpp error: boardAll returned success but status is null");
      }
      std::string status_str(status_c);
      bark::bark_free_string(status_c); // Use helper
      return status_str; });
    }

    std::shared_ptr<Promise<std::string>>
    send(const std::string &datadir, const std::string &mnemonic,
         const std::string &destination, double amountSat,
         const std::optional<std::string> &comment, bool no_sync) override
    {
      return Promise<std::string>::async([datadir, mnemonic, destination,
                                          amountSat, comment, no_sync]()
                                         {
      char *status_c = nullptr;
      const char *comment_c = comment.has_value() ? comment->c_str() : nullptr;
      // NOTE: bark_send in ffi.rs expects u64::MAX if amount is not provided.
      // Here, amountSat (double) is always passed from TS. If you want to
      // support sending MAX, the TS/Nitro interface needs adjustment (e.g.,
      // optional amount). Assuming amountSat passed from TS is always the
      // intended amount.
      bark::bark_BarkError *error = bark::bark_send(
          datadir.c_str(), mnemonic.c_str(), destination.c_str(),
          static_cast<uint64_t>(amountSat), comment_c, no_sync, &status_c);
      check_bark_error(error);
      if (status_c == nullptr) {
        throw std::runtime_error(
            "Bark-cpp error: send returned success but status is null");
      }
      std::string status_str(status_c);
      bark::bark_free_string(status_c); // Use helper
      return status_str; });
    }

    std::shared_ptr<Promise<std::string>>
    sendRoundOnchain(const std::string &datadir, const std::string &mnemonic,
                     const std::string &destination, double amountSat,
                     bool no_sync) override
    {
      return Promise<std::string>::async(
          [datadir, mnemonic, destination, amountSat, no_sync]()
          {
            char *status_c = nullptr;
            bark::bark_BarkError *error = bark::bark_send_round_onchain(
                datadir.c_str(), mnemonic.c_str(), destination.c_str(),
                static_cast<uint64_t>(amountSat), no_sync, &status_c);
            check_bark_error(error);
            if (status_c == nullptr)
            {
              throw std::runtime_error("Bark-cpp error: sendRoundOnchain "
                                       "returned success but status is null");
            }
            std::string status_str(status_c);
            bark::bark_free_string(status_c); // Use helper
            return status_str;
          });
    }

    // --- Offboarding / Exiting ---

    std::shared_ptr<Promise<std::string>>
    offboardSpecific(const std::string &datadir, const std::string &mnemonic,
                     const std::vector<std::string> &vtxoIds,
                     const std::optional<std::string> &optionalAddress,
                     bool no_sync) override
    {
      return Promise<std::string>::async(
          [datadir, mnemonic, vtxoIds, optionalAddress, no_sync]()
          {
            if (vtxoIds.empty())
            {
              throw std::runtime_error(
                  "offboardSpecific requires at least one vtxoId");
            }
            std::vector<const char *> ids_c;
            ids_c.reserve(vtxoIds.size());
            for (const auto &id : vtxoIds)
            {
              ids_c.push_back(id.c_str());
            }
            const char *addr_c =
                optionalAddress.has_value() ? optionalAddress->c_str() : nullptr;
            char *status_c = nullptr;

            bark::bark_BarkError *error = bark::bark_offboard_specific(
                datadir.c_str(), mnemonic.c_str(), ids_c.data(), ids_c.size(),
                addr_c, no_sync, &status_c);
            check_bark_error(error);
            if (status_c == nullptr)
            {
              throw std::runtime_error("Bark-cpp error: offboardSpecific "
                                       "returned success but status is null");
            }
            std::string status_str(status_c);
            bark::bark_free_string(status_c); // Use helper
            return status_str;
          });
    }

    std::shared_ptr<Promise<std::string>>
    offboardAll(const std::string &datadir, const std::string &mnemonic,
                const std::optional<std::string> &optionalAddress,
                bool no_sync) override
    {
      return Promise<std::string>::async([datadir, mnemonic, optionalAddress,
                                          no_sync]()
                                         {
      const char *addr_c =
          optionalAddress.has_value() ? optionalAddress->c_str() : nullptr;
      char *status_c = nullptr;
      bark::bark_BarkError *error = bark::bark_offboard_all(
          datadir.c_str(), mnemonic.c_str(), addr_c, no_sync, &status_c);
      check_bark_error(error);
      if (status_c == nullptr) {
        throw std::runtime_error(
            "Bark-cpp error: offboardAll returned success but status is null");
      }
      std::string status_str(status_c);
      bark::bark_free_string(status_c); // Use helper
      return status_str; });
    }

    std::shared_ptr<Promise<std::string>> exitStartSpecific(
        const std::string &datadir, const std::string &mnemonic,
        const std::vector<std::string> &vtxoIds,
        bool no_sync /* Potential C header mismatch noted */) override
    {
      return Promise<std::string>::async([datadir, mnemonic, vtxoIds, no_sync]()
                                         {
      if (vtxoIds.empty()) {
        throw std::runtime_error(
            "exitStartSpecific requires at least one vtxoId");
      }
      std::vector<const char *> ids_c;
      ids_c.reserve(vtxoIds.size());
      for (const auto &id : vtxoIds) {
        ids_c.push_back(id.c_str());
      }
      char *status_c = nullptr;

      // Call reflects C header (which might be missing no_sync)
      bark::bark_BarkError *error =
          bark::bark_exit_start_specific(datadir.c_str(), mnemonic.c_str(),
                                         ids_c.data(), ids_c.size(), &status_c);
      check_bark_error(error);
      if (status_c == nullptr) {
        throw std::runtime_error("Bark-cpp error: exitStartSpecific returned "
                                 "success but status is null");
      }
      std::string status_str(status_c);
      bark::bark_free_string(status_c); // Use helper
      return status_str; });
    }

    std::shared_ptr<Promise<std::string>>
    exitStartAll(const std::string &datadir, const std::string &mnemonic,
                 bool no_sync /* Potential C header mismatch noted */) override
    {
      return Promise<std::string>::async([datadir, mnemonic, no_sync]()
                                         {
      char *status_c = nullptr;
      // Call reflects C header (which might be missing no_sync)
      bark::bark_BarkError *error = bark::bark_exit_start_all(
          datadir.c_str(), mnemonic.c_str(), &status_c);
      check_bark_error(error);
      if (status_c == nullptr) {
        throw std::runtime_error(
            "Bark-cpp error: exitStartAll returned success but status is null");
      }
      std::string status_str(status_c);
      bark::bark_free_string(status_c); // Use helper
      return status_str; });
    }

    std::shared_ptr<Promise<std::string>>
    exitProgressOnce(const std::string &datadir,
                     const std::string &mnemonic) override
    {
      return Promise<std::string>::async([datadir, mnemonic]()
                                         {
      char *status_c = nullptr;
      bark::bark_BarkError *error = bark::bark_exit_progress_once(
          datadir.c_str(), mnemonic.c_str(), &status_c);
      check_bark_error(error);
      if (status_c == nullptr) {
        throw std::runtime_error("Bark-cpp error: exitProgressOnce returned "
                                 "success but status is null");
      }
      std::string status_str(status_c);
      bark::bark_free_string(status_c); // Use helper
      return status_str; });
    }

  private:
    // Tag for logging/debugging within Nitro
    static constexpr auto TAG = "NitroArk";
  };

} // namespace margelo::nitro::nitroark
