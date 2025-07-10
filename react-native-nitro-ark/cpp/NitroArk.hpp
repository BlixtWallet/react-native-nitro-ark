#pragma once

#include "HybridNitroArkSpec.hpp"
#include "generated/ark_cxx.h"
#include "generated/cxx.h"
#include "bark-cpp.h"
#include <memory>
#include <stdexcept>
#include <string>
#include <vector>

namespace margelo::nitro::nitroark
{

    class NitroArk : public HybridNitroArkSpec
    {
    public:
        NitroArk() : HybridObject(TAG)
        {
            // Initialize the Rust logger once when a NitroArk object is created.
            bark_cxx::init_logger();
        }

        // --- Management ---

        std::shared_ptr<Promise<std::string>> createMnemonic() override
        {
            return Promise<std::string>::async([]()
                                               {
            try {
                rust::String mnemonic_rs = bark_cxx::create_mnemonic();
                return std::string(mnemonic_rs.data(), mnemonic_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<void>>
        loadWallet(const std::string &datadir,
                   const BarkCreateOpts &opts) override
        {
            return Promise<void>::async([datadir, opts]()
                                        {
            try {
                  bark_cxx::ConfigOpts config_opts;
                if (opts.config.has_value()) {
                    config_opts.asp = opts.config->asp.value_or("");
                    config_opts.esplora = opts.config->esplora.value_or("");
                    config_opts.bitcoind = opts.config->bitcoind.value_or("");
                    config_opts.bitcoind_cookie = opts.config->bitcoind_cookie.value_or("");
                    config_opts.bitcoind_user = opts.config->bitcoind_user.value_or("");
                    config_opts.bitcoind_pass = opts.config->bitcoind_pass.value_or("");
                    config_opts.vtxo_refresh_expiry_threshold = static_cast<uint32_t>(opts.config->vtxo_refresh_expiry_threshold.value_or(0));
                    config_opts.fallback_fee_rate = static_cast<uint64_t>(opts.config->fallback_fee_rate.value_or(0));
                }

                bark_cxx::CreateOpts create_opts;
                create_opts.regtest = opts.regtest.value_or(false);
                create_opts.signet = opts.signet.value_or(false);
                create_opts.bitcoin = opts.bitcoin.value_or(true);
                create_opts.mnemonic = opts.mnemonic;
                create_opts.birthday_height = static_cast<uint32_t>(opts.birthday_height.value_or(0));
                create_opts.config = config_opts;

                bark_cxx::load_wallet(datadir, create_opts);
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<void>> closeWallet() override
        {
            return Promise<void>::async([]()
                                        {
            try {
                bark_cxx::close_wallet();
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<bool>> isWalletLoaded() override
        {
            return Promise<bool>::async([]()
                                        { return bark_cxx::is_wallet_loaded(); });
        }

        // --- Wallet Info ---

        std::shared_ptr<Promise<BarkBalance>>
        getBalance(bool no_sync) override
        {
            return Promise<BarkBalance>::async([no_sync]()
                                               {
            try {
                bark_cxx::CxxBalance c_balance = bark_cxx::get_balance(no_sync);
                return BarkBalance{static_cast<double>(c_balance.onchain),
                                   static_cast<double>(c_balance.offchain),
                                   static_cast<double>(c_balance.pending_exit)};
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>>
        getOnchainAddress() override
        {
            return Promise<std::string>::async([]()
                                               {
            try {
                rust::String address_rs = bark_cxx::get_onchain_address();
                return std::string(address_rs.data(), address_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>>
        getOnchainUtxos(bool no_sync) override
        {
            return Promise<std::string>::async([no_sync]()
                                               {
            try {
                rust::String json_rs = bark_cxx::get_onchain_utxos(no_sync);
                return std::string(json_rs.data(), json_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>>
        getVtxoPubkey(std::optional<double> index) override
        {
            return Promise<std::string>::async([index]()
                                               {
            try {
                uint32_t index_val = index.has_value() ? static_cast<uint32_t>(index.value()) : UINT32_MAX;
                rust::String pubkey_rs = bark_cxx::get_vtxo_pubkey(index_val);
                return std::string(pubkey_rs.data(), pubkey_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>> getVtxos(bool no_sync) override
        {
            return Promise<std::string>::async([no_sync]()
                                               {
            try {
                rust::String json_rs = bark_cxx::get_vtxos(no_sync);
                return std::string(json_rs.data(), json_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        // --- Onchain Operations ---

        std::shared_ptr<Promise<std::string>>
        sendOnchain(const std::string &destination, double amountSat,
                    bool no_sync) override
        {
            return Promise<std::string>::async([destination, amountSat, no_sync]()
                                               {
            try {
                rust::String txid_rs = bark_cxx::send_onchain(destination, static_cast<uint64_t>(amountSat), no_sync);
                return std::string(txid_rs.data(), txid_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>>
        drainOnchain(const std::string &destination, bool no_sync) override
        {
            return Promise<std::string>::async(
                [destination, no_sync]()
                {
                    try
                    {
                        rust::String txid_rs = bark_cxx::drain_onchain(destination, no_sync);
                        return std::string(txid_rs.data(), txid_rs.length());
                    }
                    catch (const rust::Error &e)
                    {
                        throw std::runtime_error(e.what());
                    }
                });
        }

        std::shared_ptr<Promise<std::string>>
        sendManyOnchain(const std::vector<BarkSendManyOutput> &outputs,
                        bool no_sync) override
        {
            return Promise<std::string>::async([outputs, no_sync]()
                                               {
            try {
                rust::Vec<bark_cxx::SendManyOutput> cxx_outputs;
                for (const auto &output : outputs) {
                    cxx_outputs.push_back({rust::String(output.destination), static_cast<uint64_t>(output.amountSat)});
                }
                rust::String txid_rs = bark_cxx::send_many_onchain(std::move(cxx_outputs), no_sync);
                return std::string(txid_rs.data(), txid_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        // --- Ark Operations ---

        std::shared_ptr<Promise<std::string>>
        refreshVtxos(const BarkRefreshOpts &refreshOpts, bool no_sync) override
        {
            return Promise<std::string>::async([refreshOpts,
                                                no_sync]()
                                               {
            try {
                bark_cxx::RefreshOpts opts;
                switch (refreshOpts.mode_type) {
                    case BarkRefreshModeType::DEFAULTTHRESHOLD:
                        opts.mode_type = bark_cxx::RefreshModeType::DefaultThreshold;
                        break;
                    case BarkRefreshModeType::THRESHOLDBLOCKS:
                        opts.mode_type = bark_cxx::RefreshModeType::ThresholdBlocks;
                        break;
                    case BarkRefreshModeType::THRESHOLDHOURS:
                        opts.mode_type = bark_cxx::RefreshModeType::ThresholdHours;
                        break;
                    case BarkRefreshModeType::COUNTERPARTY:
                        opts.mode_type = bark_cxx::RefreshModeType::Counterparty;
                        break;
                    case BarkRefreshModeType::ALL:
                        opts.mode_type = bark_cxx::RefreshModeType::All;
                        break;
                    case BarkRefreshModeType::SPECIFIC:
                        opts.mode_type = bark_cxx::RefreshModeType::Specific;
                        break;
                }
                opts.threshold_value = static_cast<uint32_t>(refreshOpts.threshold_value.value_or(0));
                if (refreshOpts.specific_vtxo_ids.has_value()) {
                    for (const auto &id : refreshOpts.specific_vtxo_ids.value()) {
                        opts.specific_vtxo_ids.push_back(id);
                    }
                }
                rust::String status_rs = bark_cxx::refresh_vtxos(opts, no_sync);
                return std::string(status_rs.data(), status_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>> boardAmount(double amountSat,
                                                          bool no_sync) override
        {
            return Promise<std::string>::async([amountSat,
                                                no_sync]()
                                               {
            try {
                rust::String status_rs = bark_cxx::board_amount(static_cast<uint64_t>(amountSat), no_sync);
                return std::string(status_rs.data(), status_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>> boardAll(bool no_sync) override
        {
            return Promise<std::string>::async([no_sync]()
                                               {
            try {
                rust::String status_rs = bark_cxx::board_all(no_sync);
                return std::string(status_rs.data(), status_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>>
        send(const std::string &destination, std::optional<double> amountSat,
             const std::optional<std::string> &comment, bool no_sync) override
        {
            return Promise<std::string>::async([destination, amountSat, comment, no_sync]()
                                               {
            try {
                uint64_t amount_val = 0;
                if (amountSat.has_value()) {
                    amount_val = static_cast<uint64_t>(amountSat.value());
                }
                std::string comment_val = "";
                if (comment.has_value()) {
                    comment_val = comment.value();
                }
                rust::String status_rs = bark_cxx::send_payment(destination, amount_val, comment_val, no_sync);
                return std::string(status_rs.data(), status_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>>
        sendRoundOnchain(const std::string &destination, double amountSat,
                         bool no_sync) override
        {
            return Promise<std::string>::async(
                [destination, amountSat, no_sync]()
                {
                    try
                    {
                        rust::String status_rs = bark_cxx::send_round_onchain(destination, static_cast<uint64_t>(amountSat), no_sync);
                        return std::string(status_rs.data(), status_rs.length());
                    }
                    catch (const rust::Error &e)
                    {
                        throw std::runtime_error(e.what());
                    }
                });
        }

        // --- Lightning Operations ---

        std::shared_ptr<Promise<std::string>>
        bolt11Invoice(double amountMsat) override
        {
            return Promise<std::string>::async([amountMsat]()
                                               {
            try {
                rust::String invoice_rs = bark_cxx::bolt11_invoice(static_cast<uint64_t>(amountMsat));
                return std::string(invoice_rs.data(), invoice_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<void>>
        claimBolt11Payment(const std::string &bolt11) override
        {
            return Promise<void>::async([bolt11]()
                                        {
            try {
                bark_cxx::claim_bolt11_payment(bolt11);
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        // --- Offboarding / Exiting ---

        std::shared_ptr<Promise<std::string>>
        offboardSpecific(const std::vector<std::string> &vtxoIds,
                         const std::optional<std::string> &optionalAddress,
                         bool no_sync) override
        {
            return Promise<std::string>::async(
                [vtxoIds, optionalAddress, no_sync]()
                {
                    try
                    {
                        rust::Vec<rust::String> rust_vtxo_ids;
                        for (const auto &id : vtxoIds)
                        {
                            rust_vtxo_ids.push_back(rust::String(id));
                        }
                        std::string address = optionalAddress.has_value() ? optionalAddress.value() : "";
                        rust::String status_rs = bark_cxx::offboard_specific(std::move(rust_vtxo_ids), address, no_sync);
                        return std::string(status_rs.data(), status_rs.length());
                    }
                    catch (const rust::Error &e)
                    {
                        throw std::runtime_error(e.what());
                    }
                });
        }

        std::shared_ptr<Promise<std::string>>
        offboardAll(const std::optional<std::string> &optionalAddress,
                    bool no_sync) override
        {
            return Promise<std::string>::async([optionalAddress,
                                                no_sync]()
                                               {
            try {
                std::string address = optionalAddress.has_value() ? optionalAddress.value() : "";
                rust::String status_rs = bark_cxx::offboard_all(address, no_sync);
                return std::string(status_rs.data(), status_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>> exitStartSpecific(
            const std::vector<std::string> &vtxoIds) override
        {
            return Promise<std::string>::async([vtxoIds]()
                                               {
            try {
                rust::Vec<rust::String> rust_vtxo_ids;
                for (const auto &id : vtxoIds)
                {
                  rust_vtxo_ids.push_back(rust::String(id));
                }
                rust::String status_rs = bark_cxx::start_exit_for_vtxos(std::move(rust_vtxo_ids));
                return std::string(status_rs.data(), status_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>>
        exitStartAll() override
        {
            return Promise<std::string>::async([]()
                                               {
            try {
                rust::String status_rs = bark_cxx::start_exit_for_entire_wallet();
                return std::string(status_rs.data(), status_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>>
        exitProgressOnce() override
        {
            return Promise<std::string>::async([]()
                                               {
            try {
                rust::String status_rs = bark_cxx::exit_progress_once();
                return std::string(status_rs.data(), status_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

    private:
        // Tag for logging/debugging within Nitro
        static constexpr auto TAG = "NitroArk";
    };

} // namespace margelo::nitro::nitroark
