#pragma once

#include "HybridNitroArkSpec.hpp"
#include "generated/ark_cxx.h"
#include "generated/cxx.h"
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
                uint32_t birthday_height_val = opts.birthday_height.value_or(0);
                create_opts.birthday_height = opts.birthday_height.has_value() ? &birthday_height_val : nullptr;
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

        std::shared_ptr<Promise<void>>
        persistConfig(const BarkConfigOpts &opts) override
        {
            return Promise<void>::async([opts]()
                                        {
            try {
                bark_cxx::ConfigOpts config_opts;
                config_opts.asp = opts.asp.value_or("");
                config_opts.esplora = opts.esplora.value_or("");
                config_opts.bitcoind = opts.bitcoind.value_or("");
                config_opts.bitcoind_cookie = opts.bitcoind_cookie.value_or("");
                config_opts.bitcoind_user = opts.bitcoind_user.value_or("");
                config_opts.bitcoind_pass = opts.bitcoind_pass.value_or("");
                config_opts.vtxo_refresh_expiry_threshold = static_cast<uint32_t>(opts.vtxo_refresh_expiry_threshold.value_or(0));
                config_opts.fallback_fee_rate = static_cast<uint64_t>(opts.fallback_fee_rate.value_or(0));
                bark_cxx::persist_config(config_opts);
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<void>> maintenance() override
        {
            return Promise<void>::async([]()
                                        {
            try {
                bark_cxx::maintenance();
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<void>> sync() override
        {
            return Promise<void>::async([]()
                                        {
            try {
                bark_cxx::sync();
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<void>> syncArk() override
        {
            return Promise<void>::async([]()
                                        {
            try {
                bark_cxx::sync_ark();
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<void>> syncRounds() override
        {
            return Promise<void>::async([]()
                                        {
            try {
                bark_cxx::sync_rounds();
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        // --- Wallet Info ---

        std::shared_ptr<Promise<BarkArkInfo>> getArkInfo() override
        {
            return Promise<BarkArkInfo>::async([]()
                                               {
            try {
                bark_cxx::CxxArkInfo rust_info = bark_cxx::get_ark_info();
                BarkArkInfo info;
                info.network = std::string(rust_info.network.data(), rust_info.network.length());
                info.asp_pubkey = std::string(rust_info.asp_pubkey.data(), rust_info.asp_pubkey.length());
                info.round_interval_secs = static_cast<double>(rust_info.round_interval_secs);
                info.vtxo_exit_delta = static_cast<double>(rust_info.vtxo_exit_delta);
                info.vtxo_expiry_delta = static_cast<double>(rust_info.vtxo_expiry_delta);
                info.htlc_expiry_delta = static_cast<double>(rust_info.htlc_expiry_delta);
                info.max_vtxo_amount_sat = static_cast<double>(rust_info.max_vtxo_amount_sat);
                return info;
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<double>> onchainBalance() override
        {
            return Promise<double>::async([]()
                                          {
            try {
                return static_cast<double>(bark_cxx::onchain_balance());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<double>> offchainBalance() override
        {
            return Promise<double>::async([]()
                                          {
            try {
                return static_cast<double>(bark_cxx::offchain_balance());
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
                uint32_t index_val = index.has_value() ? static_cast<uint32_t>(index.value()) : 0;
                rust::String pubkey_rs = bark_cxx::get_vtxo_pubkey(index.has_value() ? &index_val : nullptr);
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

        // --- Ark & Lightning Payments ---

        std::shared_ptr<Promise<std::string>> boardAmount(double amountSat) override
        {
            return Promise<std::string>::async([amountSat]()
                                               {
            try {
                rust::String status_rs = bark_cxx::board_amount(static_cast<uint64_t>(amountSat));
                return std::string(status_rs.data(), status_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>> boardAll() override
        {
            return Promise<std::string>::async([]()
                                               {
            try {
                rust::String status_rs = bark_cxx::board_all();
                return std::string(status_rs.data(), status_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>>
        sendArkoorPayment(const std::string &destination, double amountSat) override
        {
            return Promise<std::string>::async([destination, amountSat]()
                                               {
            try {
                rust::String status_rs = bark_cxx::send_arkoor_payment(destination, static_cast<uint64_t>(amountSat));
                return std::string(status_rs.data(), status_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>>
        sendBolt11Payment(const std::string &destination, double amountSat) override
        {
            return Promise<std::string>::async([destination, amountSat]()
                                               {
            try {
                rust::String status_rs = bark_cxx::send_bolt11_payment(destination, static_cast<uint64_t>(amountSat));
                return std::string(status_rs.data(), status_rs.length());
            } catch (const rust::Error &e) {
                throw std::runtime_error(e.what());
            } });
        }

        std::shared_ptr<Promise<std::string>>
        sendLnaddr(const std::string &addr, double amountSat, const std::string &comment) override
        {
            return Promise<std::string>::async([addr, amountSat, comment]()
                                               {
            try {
                rust::String status_rs = bark_cxx::send_lnaddr(addr, static_cast<uint64_t>(amountSat), comment);
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

        // --- Lightning Invoicing ---

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
                         const std::string &destinationAddress,
                         bool no_sync) override
        {
            return Promise<std::string>::async(
                [vtxoIds, destinationAddress, no_sync]()
                {
                    try
                    {
                        rust::Vec<rust::String> rust_vtxo_ids;
                        for (const auto &id : vtxoIds)
                        {
                            rust_vtxo_ids.push_back(rust::String(id));
                        }
                        rust::String status_rs = bark_cxx::offboard_specific(std::move(rust_vtxo_ids), destinationAddress, no_sync);
                        return std::string(status_rs.data(), status_rs.length());
                    }
                    catch (const rust::Error &e)
                    {
                        throw std::runtime_error(e.what());
                    }
                });
        }

        std::shared_ptr<Promise<std::string>>
        offboardAll(const std::string &destinationAddress,
                    bool no_sync) override
        {
            return Promise<std::string>::async([destinationAddress,
                                                no_sync]()
                                               {
            try {
                rust::String status_rs = bark_cxx::offboard_all(destinationAddress, no_sync);
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
