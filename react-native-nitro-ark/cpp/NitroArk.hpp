#pragma once

#include "HybridNitroArkSpec.hpp"
#include "generated/ark_cxx.h"
#include "generated/cxx.h"
#include <memory>
#include <stdexcept>
#include <string>
#include <sys/wait.h>
#include <vector>

namespace margelo::nitro::nitroark {

using namespace margelo::nitro;
// Helper function to convert rust vtxos vector to C++ vector
inline std::vector<BarkVtxo> convertRustVtxosToVector(const rust::Vec<bark_cxx::BarkVtxo>& rust_vtxos) {
  std::vector<BarkVtxo> vtxos;
  vtxos.reserve(rust_vtxos.size());

  for (const auto& vtxo_rs : rust_vtxos) {
    BarkVtxo vtxo;
    vtxo.amount = static_cast<double>(vtxo_rs.amount);
    vtxo.expiry_height = static_cast<double>(vtxo_rs.expiry_height);
    vtxo.server_pubkey = std::string(vtxo_rs.server_pubkey.data(), vtxo_rs.server_pubkey.length());
    vtxo.exit_delta = static_cast<double>(vtxo_rs.exit_delta);
    vtxo.anchor_point = std::string(vtxo_rs.anchor_point.data(), vtxo_rs.anchor_point.length());
    vtxo.point = std::string(vtxo_rs.point.data(), vtxo_rs.point.length());
    vtxo.state = std::string(vtxo_rs.state.data(), vtxo_rs.state.length());
    vtxos.push_back(std::move(vtxo));
  }

  return vtxos;
}

class NitroArk : public HybridNitroArkSpec {

private:
  // Helper function to create ConfigOpts from BarkConfigOpts
  static bark_cxx::ConfigOpts createConfigOpts(const std::optional<BarkConfigOpts>& config) {
    bark_cxx::ConfigOpts config_opts;
    if (config.has_value()) {
      config_opts.ark = config->ark.value_or("");
      config_opts.esplora = config->esplora.value_or("");
      config_opts.bitcoind = config->bitcoind.value_or("");
      config_opts.bitcoind_cookie = config->bitcoind_cookie.value_or("");
      config_opts.bitcoind_user = config->bitcoind_user.value_or("");
      config_opts.bitcoind_pass = config->bitcoind_pass.value_or("");
      config_opts.vtxo_refresh_expiry_threshold =
          static_cast<uint32_t>(config->vtxo_refresh_expiry_threshold.value_or(0));
      config_opts.fallback_fee_rate = static_cast<uint64_t>(config->fallback_fee_rate.value_or(0));
      config_opts.htlc_recv_claim_delta = static_cast<uint32_t>(config->htlc_recv_claim_delta);
      config_opts.vtxo_exit_margin = static_cast<uint32_t>(config->vtxo_exit_margin);
      config_opts.round_tx_required_confirmations = static_cast<uint32_t>(config->round_tx_required_confirmations);
    }
    return config_opts;
  }

public:
  NitroArk() : HybridObject(TAG) {
    // Initialize the Rust logger once when a NitroArk object is created.
    bark_cxx::init_logger();
  }

  // --- Management ---

  std::shared_ptr<Promise<std::string>> createMnemonic() override {
    return Promise<std::string>::async([]() {
      try {
        rust::String mnemonic_rs = bark_cxx::create_mnemonic();
        return std::string(mnemonic_rs.data(), mnemonic_rs.length());
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<void>> createWallet(const std::string& datadir, const BarkCreateOpts& opts) override {
    return Promise<void>::async([datadir, opts]() {
      try {
        bark_cxx::CreateOpts create_opts;
        create_opts.regtest = opts.regtest.value_or(false);
        create_opts.signet = opts.signet.value_or(false);
        create_opts.bitcoin = opts.bitcoin.value_or(true);
        create_opts.mnemonic = opts.mnemonic;

        uint32_t birthday_height_val;
        if (opts.birthday_height.has_value()) {
          birthday_height_val = static_cast<uint32_t>(opts.birthday_height.value());
          create_opts.birthday_height = &birthday_height_val;
        } else {
          create_opts.birthday_height = nullptr;
        }

        create_opts.config = createConfigOpts(opts.config);

        bark_cxx::create_wallet(datadir, create_opts);
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<void>> loadWallet(const std::string& datadir, const BarkCreateOpts& opts) override {
    return Promise<void>::async([datadir, opts]() {
      try {
        bark_cxx::CreateOpts create_opts;
        create_opts.regtest = opts.regtest.value_or(false);
        create_opts.signet = opts.signet.value_or(false);
        create_opts.bitcoin = opts.bitcoin.value_or(true);
        create_opts.mnemonic = opts.mnemonic;

        uint32_t birthday_height_val;
        if (opts.birthday_height.has_value()) {
          birthday_height_val = static_cast<uint32_t>(opts.birthday_height.value());
          create_opts.birthday_height = &birthday_height_val;
        } else {
          create_opts.birthday_height = nullptr;
        }

        create_opts.config = createConfigOpts(opts.config);

        bark_cxx::load_wallet(datadir, create_opts);
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<void>> closeWallet() override {
    return Promise<void>::async([]() {
      try {
        bark_cxx::close_wallet();
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<void>> refreshServer() override {
    return Promise<void>::async([]() {
      try {
        bark_cxx::refresh_server();
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<bool>> isWalletLoaded() override {
    return Promise<bool>::async([]() { return bark_cxx::is_wallet_loaded(); });
  }

  std::shared_ptr<Promise<void>> syncPendingBoards() override {
    return Promise<void>::async([]() {
      try {
        bark_cxx::sync_pending_boards();
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<void>> maintenance() override {
    return Promise<void>::async([]() {
      try {
        bark_cxx::maintenance();
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<void>> maintenanceWithOnchain() override {
    return Promise<void>::async([]() {
      try {
        bark_cxx::maintenance_with_onchain();
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<void>> maintenanceRefresh() override {
    return Promise<void>::async([]() {
      try {
        bark_cxx::maintenance_refresh();
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<void>> sync() override {
    return Promise<void>::async([]() {
      try {
        bark_cxx::sync();
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<void>> syncExits() override {
    return Promise<void>::async([]() {
      try {
        bark_cxx::sync_exits();
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<void>> syncPendingRounds() override {
    return Promise<void>::async([]() {
      try {
        bark_cxx::sync_pending_rounds();
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  // --- Wallet Info ---

  std::shared_ptr<Promise<BarkArkInfo>> getArkInfo() override {
    return Promise<BarkArkInfo>::async([]() {
      try {
        bark_cxx::CxxArkInfo rust_info = bark_cxx::get_ark_info();
        BarkArkInfo info;
        info.network = std::string(rust_info.network.data(), rust_info.network.length());
        info.server_pubkey = std::string(rust_info.server_pubkey.data(), rust_info.server_pubkey.length());
        info.round_interval = static_cast<double>(rust_info.round_interval);
        info.nb_round_nonces = static_cast<double>(rust_info.nb_round_nonces);
        info.vtxo_exit_delta = static_cast<double>(rust_info.vtxo_exit_delta);
        info.vtxo_expiry_delta = static_cast<double>(rust_info.vtxo_expiry_delta);
        info.htlc_send_expiry_delta = static_cast<double>(rust_info.htlc_send_expiry_delta);
        info.max_vtxo_amount = static_cast<double>(rust_info.max_vtxo_amount);
        info.required_board_confirmations = static_cast<double>(rust_info.required_board_confirmations);
        return info;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<OffchainBalanceResult>> offchainBalance() override {
    return Promise<OffchainBalanceResult>::async([]() {
      try {
        bark_cxx::OffchainBalance rust_balance = bark_cxx::offchain_balance();
        OffchainBalanceResult balance;
        balance.spendable = static_cast<double>(rust_balance.spendable);
        balance.pending_lightning_send = static_cast<double>(rust_balance.pending_lightning_send);
        balance.pending_in_round = static_cast<double>(rust_balance.pending_in_round);
        balance.pending_exit = static_cast<double>(rust_balance.pending_exit);
        balance.pending_board = static_cast<double>(rust_balance.pending_board);

        return balance;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<KeyPairResult>> deriveStoreNextKeypair() override {
    return Promise<KeyPairResult>::async([]() {
      try {
        bark_cxx::KeyPairResult keypair_rs = bark_cxx::derive_store_next_keypair();
        KeyPairResult keypair;
        keypair.public_key = std::string(keypair_rs.public_key.data(), keypair_rs.public_key.length());
        keypair.secret_key = std::string(keypair_rs.secret_key.data(), keypair_rs.secret_key.length());

        return keypair;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<KeyPairResult>> peakKeyPair(double index) override {
    return Promise<KeyPairResult>::async([index]() {
      try {
        uint32_t index_val = static_cast<uint32_t>(index);
        bark_cxx::KeyPairResult keypair_rs = bark_cxx::peak_keypair(index_val);
        KeyPairResult keypair;
        keypair.public_key = std::string(keypair_rs.public_key.data(), keypair_rs.public_key.length());
        keypair.secret_key = std::string(keypair_rs.secret_key.data(), keypair_rs.secret_key.length());
        return keypair;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<NewAddressResult>> newAddress() override {
    return Promise<NewAddressResult>::async([]() {
      try {
        bark_cxx::NewAddressResult address_rs = bark_cxx::new_address();
        NewAddressResult address;
        address.user_pubkey = std::string(address_rs.user_pubkey.data(), address_rs.user_pubkey.length());
        address.ark_id = std::string(address_rs.ark_id.data(), address_rs.ark_id.length());
        address.address = std::string(address_rs.address.data(), address_rs.address.length());
        return address;

      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<NewAddressResult>> peakAddress(double index) override {
    return Promise<NewAddressResult>::async([index]() {
      try {
        bark_cxx::NewAddressResult address_rs = bark_cxx::peak_address(static_cast<uint32_t>(index));
        NewAddressResult address;
        address.user_pubkey = std::string(address_rs.user_pubkey.data(), address_rs.user_pubkey.length());
        address.ark_id = std::string(address_rs.ark_id.data(), address_rs.ark_id.length());
        address.address = std::string(address_rs.address.data(), address_rs.address.length());
        return address;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::string>> signMessage(const std::string& message, double index) override {
    return Promise<std::string>::async([message, index]() {
      try {
        uint32_t index_val = static_cast<uint32_t>(index);
        rust::String signature_rs = bark_cxx::sign_message(message, index_val);
        return std::string(signature_rs.data(), signature_rs.length());
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::string>> signMesssageWithMnemonic(const std::string& message,
                                                                 const std::string& mnemonic,
                                                                 const std::string& network, double index) override {
    return Promise<std::string>::async([message, mnemonic, network, index]() {
      try {
        uint32_t index_val = static_cast<uint32_t>(index);
        rust::String signature_rs = bark_cxx::sign_messsage_with_mnemonic(message, mnemonic, network, index_val);
        return std::string(signature_rs.data(), signature_rs.length());
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<KeyPairResult>> deriveKeypairFromMnemonic(const std::string& mnemonic,
                                                                    const std::string& network, double index) override {
    return Promise<KeyPairResult>::async([mnemonic, network, index]() {
      try {
        uint32_t index_val = static_cast<uint32_t>(index);
        bark_cxx::KeyPairResult keypair_rs = bark_cxx::derive_keypair_from_mnemonic(mnemonic, network, index_val);
        KeyPairResult keypair;
        keypair.public_key = std::string(keypair_rs.public_key.data(), keypair_rs.public_key.length());
        keypair.secret_key = std::string(keypair_rs.secret_key.data(), keypair_rs.secret_key.length());
        return keypair;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<bool>> verifyMessage(const std::string& message, const std::string& signature,
                                               const std::string& publicKey) override {
    return Promise<bool>::async([message, signature, publicKey]() {
      try {
        return bark_cxx::verify_message(message, signature, publicKey);
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::vector<BarkMovement>>> history() override {
    return Promise<std::vector<BarkMovement>>::async([]() {
      try {
        rust::Vec<bark_cxx::BarkMovement> movements_rs = bark_cxx::history();

        std::vector<BarkMovement> movements;
        movements.reserve(movements_rs.size());

        for (const auto& movement_rs : movements_rs) {
          BarkMovement movement;
          movement.id = static_cast<double>(movement_rs.id);
          movement.status = std::string(movement_rs.status.data(), movement_rs.status.length());
          movement.metadata_json = std::string(movement_rs.metadata_json.data(), movement_rs.metadata_json.length());
          movement.intended_balance_sat = static_cast<double>(movement_rs.intended_balance_sat);
          movement.effective_balance_sat = static_cast<double>(movement_rs.effective_balance_sat);
          movement.offchain_fee_sat = static_cast<double>(movement_rs.offchain_fee_sat);
          movement.created_at = std::string(movement_rs.created_at.data(), movement_rs.created_at.length());
          movement.updated_at = std::string(movement_rs.updated_at.data(), movement_rs.updated_at.length());
          if (movement_rs.completed_at.length() == 0) {
            movement.completed_at = std::nullopt;
          } else {
            movement.completed_at = std::string(movement_rs.completed_at.data(), movement_rs.completed_at.length());
          }

          movement.subsystem.name = std::string(movement_rs.subsystem_name.data(), movement_rs.subsystem_name.length());
          movement.subsystem.kind = std::string(movement_rs.subsystem_kind.data(), movement_rs.subsystem_kind.length());

          movement.sent_to.reserve(movement_rs.sent_to.size());
          for (const auto& dest_rs : movement_rs.sent_to) {
            BarkMovementDestination destination;
            destination.destination = std::string(dest_rs.destination.data(), dest_rs.destination.length());
            destination.payment_method = std::string(dest_rs.payment_method.data(), dest_rs.payment_method.length());
            destination.amount_sat = static_cast<double>(dest_rs.amount_sat);
            movement.sent_to.push_back(std::move(destination));
          }

          movement.received_on.reserve(movement_rs.received_on.size());
          for (const auto& dest_rs : movement_rs.received_on) {
            BarkMovementDestination destination;
            destination.destination = std::string(dest_rs.destination.data(), dest_rs.destination.length());
            destination.payment_method = std::string(dest_rs.payment_method.data(), dest_rs.payment_method.length());
            destination.amount_sat = static_cast<double>(dest_rs.amount_sat);
            movement.received_on.push_back(std::move(destination));
          }

          movement.input_vtxos.reserve(movement_rs.input_vtxos.size());
          for (const auto& vtxo_id : movement_rs.input_vtxos) {
            movement.input_vtxos.emplace_back(std::string(vtxo_id.data(), vtxo_id.length()));
          }

          movement.output_vtxos.reserve(movement_rs.output_vtxos.size());
          for (const auto& vtxo_id : movement_rs.output_vtxos) {
            movement.output_vtxos.emplace_back(std::string(vtxo_id.data(), vtxo_id.length()));
          }

          movement.exited_vtxos.reserve(movement_rs.exited_vtxos.size());
          for (const auto& vtxo_id : movement_rs.exited_vtxos) {
            movement.exited_vtxos.emplace_back(std::string(vtxo_id.data(), vtxo_id.length()));
          }

          movements.push_back(std::move(movement));
        }

        return movements;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::vector<BarkVtxo>>> vtxos() override {
    return Promise<std::vector<BarkVtxo>>::async([]() {
      try {
        rust::Vec<bark_cxx::BarkVtxo> rust_vtxos = bark_cxx::vtxos();
        return convertRustVtxosToVector(rust_vtxos);
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::vector<BarkVtxo>>> getExpiringVtxos(double threshold) override {
    return Promise<std::vector<BarkVtxo>>::async([threshold]() {
      try {
        rust::Vec<bark_cxx::BarkVtxo> rust_vtxos = bark_cxx::get_expiring_vtxos(static_cast<uint32_t>(threshold));
        return convertRustVtxosToVector(rust_vtxos);
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::optional<double>>> getFirstExpiringVtxoBlockheight() override {
    return Promise<std::optional<double>>::async([]() {
      try {
        const uint32_t* result_ptr = bark_cxx::get_first_expiring_vtxo_blockheight();
        if (result_ptr == nullptr) {
          return std::optional<double>(std::nullopt);
        }
        double value = static_cast<double>(*result_ptr);
        delete result_ptr; // Free the heap-allocated memory from Rust
        return std::optional<double>(value);
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::optional<double>>> getNextRequiredRefreshBlockheight() override {
    return Promise<std::optional<double>>::async([]() {
      try {
        const uint32_t* result_ptr = bark_cxx::get_next_required_refresh_blockheight();
        if (result_ptr == nullptr) {
          return std::optional<double>(std::nullopt);
        }
        double value = static_cast<double>(*result_ptr);
        delete result_ptr; // Free the heap-allocated memory from Rust
        return std::optional<double>(value);
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  // --- Onchain Operations ---

  std::shared_ptr<Promise<OnchainBalanceResult>> onchainBalance() override {
    return Promise<OnchainBalanceResult>::async([]() {
      try {
        bark_cxx::OnChainBalance rust_balance = bark_cxx::onchain_balance();
        OnchainBalanceResult balance;
        balance.immature = static_cast<double>(rust_balance.immature);
        balance.trusted_pending = static_cast<double>(rust_balance.trusted_pending);
        balance.untrusted_pending = static_cast<double>(rust_balance.untrusted_pending);
        balance.confirmed = static_cast<double>(rust_balance.confirmed);
        return balance;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<void>> onchainSync() override {
    return Promise<void>::async([]() {
      try {
        bark_cxx::onchain_sync();
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::string>> onchainListUnspent() override {
    return Promise<std::string>::async([]() {
      try {
        rust::String json_rs = bark_cxx::onchain_list_unspent();
        return std::string(json_rs.data(), json_rs.length());
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::string>> onchainUtxos() override {
    return Promise<std::string>::async([]() {
      try {
        rust::String json_rs = bark_cxx::onchain_utxos();
        return std::string(json_rs.data(), json_rs.length());
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::string>> onchainAddress() override {
    return Promise<std::string>::async([]() {
      try {
        rust::String address_rs = bark_cxx::onchain_address();
        return std::string(address_rs.data(), address_rs.length());
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<OnchainPaymentResult>> onchainSend(const std::string& destination, double amountSat,
                                                             std::optional<double> feeRate) override {
    return Promise<OnchainPaymentResult>::async([destination, amountSat, feeRate]() {
      try {
        uint64_t feeRate_val;
        bark_cxx::OnchainPaymentResult rust_result;
        if (feeRate.has_value()) {
          feeRate_val = static_cast<uint64_t>(feeRate.value());
          rust_result = bark_cxx::onchain_send(destination, static_cast<uint64_t>(amountSat), &feeRate_val);
        } else {
          rust_result = bark_cxx::onchain_send(destination, static_cast<uint64_t>(amountSat), nullptr);
        }

        OnchainPaymentResult result;
        result.txid = std::string(rust_result.txid.data(), rust_result.txid.length());
        result.amount_sat = static_cast<double>(rust_result.amount_sat);
        result.destination_address =
            std::string(rust_result.destination_address.data(), rust_result.destination_address.length());

        return result;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::string>> onchainDrain(const std::string& destination,
                                                     std::optional<double> feeRate) override {
    return Promise<std::string>::async([destination, feeRate]() {
      try {
        uint64_t feeRate_val;
        rust::String txid_rs;
        if (feeRate.has_value()) {
          feeRate_val = static_cast<uint64_t>(feeRate.value());
          txid_rs = bark_cxx::onchain_drain(destination, &feeRate_val);
        } else {
          txid_rs = bark_cxx::onchain_drain(destination, nullptr);
        }
        return std::string(txid_rs.data(), txid_rs.length());
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::string>> onchainSendMany(const std::vector<BarkSendManyOutput>& outputs,
                                                        std::optional<double> feeRate) override {
    return Promise<std::string>::async([outputs, feeRate]() {
      try {
        rust::Vec<bark_cxx::SendManyOutput> cxx_outputs;
        for (const auto& output : outputs) {
          cxx_outputs.push_back({rust::String(output.destination), static_cast<uint64_t>(output.amountSat)});
        }
        uint64_t feeRate_val;
        rust::String txid_rs;
        if (feeRate.has_value()) {
          feeRate_val = static_cast<uint64_t>(feeRate.value());
          txid_rs = bark_cxx::onchain_send_many(std::move(cxx_outputs), &feeRate_val);
        } else {
          txid_rs = bark_cxx::onchain_send_many(std::move(cxx_outputs), nullptr);
        }
        return std::string(txid_rs.data(), txid_rs.length());
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  // --- Lightning Operations ---

  std::shared_ptr<Promise<LightningSendResult>> payLightningInvoice(const std::string& destination,
                                                                    std::optional<double> amountSat) override {
    return Promise<LightningSendResult>::async([destination, amountSat]() {
      try {
        bark_cxx::LightningSend rust_result;
        if (amountSat.has_value()) {
          uint64_t amountSat_val = static_cast<uint64_t>(amountSat.value());
          rust_result = bark_cxx::pay_lightning_invoice(destination, &amountSat_val);
        } else {
          rust_result = bark_cxx::pay_lightning_invoice(destination, nullptr);
        }

        LightningSendResult result;
        result.invoice = std::string(rust_result.invoice.data(), rust_result.invoice.length());
        result.payment_hash = std::string(rust_result.payment_hash.data(), rust_result.payment_hash.length());
        result.amount = static_cast<double>(rust_result.amount);
        result.htlc_vtxos = convertRustVtxosToVector(rust_result.htlc_vtxos);
        result.movement_id = static_cast<double>(rust_result.movement_id);
        std::string preimage_str(rust_result.preimage.data(), rust_result.preimage.length());
        result.preimage = preimage_str.empty()
                              ? std::nullopt
                              : std::make_optional(std::variant<nitro::NullType, std::string>(preimage_str));

        return result;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<LightningSendResult>> payLightningOffer(const std::string& offer,
                                                                  std::optional<double> amountSat) override {
    return Promise<LightningSendResult>::async([offer, amountSat]() {
      try {
        bark_cxx::LightningSend rust_result;
        if (amountSat.has_value()) {
          uint64_t amountSat_val = static_cast<uint64_t>(amountSat.value());
          rust_result = bark_cxx::pay_lightning_offer(offer, &amountSat_val);
        } else {
          rust_result = bark_cxx::pay_lightning_offer(offer, nullptr);
        }

        LightningSendResult result;
        result.invoice = std::string(rust_result.invoice.data(), rust_result.invoice.length());
        result.payment_hash = std::string(rust_result.payment_hash.data(), rust_result.payment_hash.length());
        result.amount = static_cast<double>(rust_result.amount);
        result.htlc_vtxos = convertRustVtxosToVector(rust_result.htlc_vtxos);
        result.movement_id = static_cast<double>(rust_result.movement_id);
        std::string preimage_str(rust_result.preimage.data(), rust_result.preimage.length());
        result.preimage = preimage_str.empty()
                              ? std::nullopt
                              : std::make_optional(std::variant<nitro::NullType, std::string>(preimage_str));

        return result;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<LightningSendResult>> payLightningAddress(const std::string& addr, double amountSat,
                                                                    const std::string& comment) override {
    return Promise<LightningSendResult>::async([addr, amountSat, comment]() {
      try {
        bark_cxx::LightningSend rust_result =
            bark_cxx::pay_lightning_address(addr, static_cast<uint64_t>(amountSat), comment);

        LightningSendResult result;
        result.invoice = std::string(rust_result.invoice.data(), rust_result.invoice.length());
        result.payment_hash = std::string(rust_result.payment_hash.data(), rust_result.payment_hash.length());
        result.amount = static_cast<double>(rust_result.amount);
        result.htlc_vtxos = convertRustVtxosToVector(rust_result.htlc_vtxos);
        result.movement_id = static_cast<double>(rust_result.movement_id);
        std::string preimage_str(rust_result.preimage.data(), rust_result.preimage.length());
        result.preimage = preimage_str.empty()
                              ? std::nullopt
                              : std::make_optional(std::variant<nitro::NullType, std::string>(preimage_str));

        return result;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<Bolt11Invoice>> bolt11Invoice(double amountMsat) override {
    return Promise<Bolt11Invoice>::async([amountMsat]() {
      try {
        bark_cxx::Bolt11Invoice invoice_rs = bark_cxx::bolt11_invoice(static_cast<uint64_t>(amountMsat));
        return Bolt11Invoice(std::string(invoice_rs.bolt11_invoice.data(), invoice_rs.bolt11_invoice.length()),
                             std::string(invoice_rs.payment_secret.data(), invoice_rs.payment_secret.length()),
                             std::string(invoice_rs.payment_hash.data(), invoice_rs.payment_hash.length()));
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<LightningReceive>>
  tryClaimLightningReceive(const std::string& paymentHash, bool wait,
                           const std::optional<std::string>& token) override {
    return Promise<LightningReceive>::async([paymentHash, wait, token]() -> LightningReceive {
      try {
        bark_cxx::LightningReceive result;
        if (token.has_value()) {
          rust::String token_rs(token.value());
          result = bark_cxx::try_claim_lightning_receive(paymentHash, wait, &token_rs);
        } else {
          result = bark_cxx::try_claim_lightning_receive(paymentHash, wait, nullptr);
        }

        LightningReceive lr;
        lr.payment_hash = std::string(result.payment_hash.data(), result.payment_hash.length());
        lr.payment_preimage = std::string(result.payment_preimage.data(), result.payment_preimage.length());
        lr.invoice = std::string(result.invoice.data(), result.invoice.length());

        if (result.preimage_revealed_at != nullptr) {
          lr.preimage_revealed_at = static_cast<double>(*result.preimage_revealed_at);
        } else {
          lr.preimage_revealed_at = std::nullopt;
        }

        if (result.finished_at != nullptr) {
          lr.finished_at = static_cast<double>(*result.finished_at);
        } else {
          lr.finished_at = std::nullopt;
        }

        return lr;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<void>> tryClaimAllLightningReceives(bool wait) override {
    return Promise<void>::async([wait]() {
      try {
        bark_cxx::try_claim_all_lightning_receives(wait);
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::optional<LightningReceive>>>
  lightningReceiveStatus(const std::string& paymentHash) override {
    return Promise<std::optional<LightningReceive>>::async([paymentHash]() {
      try {
        const bark_cxx::LightningReceive* status_ptr = bark_cxx::lightning_receive_status(paymentHash);

        if (status_ptr == nullptr) {
          return std::optional<LightningReceive>();
        }

        std::unique_ptr<const bark_cxx::LightningReceive> status(status_ptr);

        LightningReceive result;
        result.payment_hash = std::string(status->payment_hash.data(), status->payment_hash.length());
        result.payment_preimage = std::string(status->payment_preimage.data(), status->payment_preimage.length());
        result.invoice = std::string(status->invoice.data(), status->invoice.length());

        if (status->preimage_revealed_at != nullptr) {
          result.preimage_revealed_at = static_cast<double>(*status->preimage_revealed_at);
        } else {
          result.preimage_revealed_at = std::nullopt;
        }

        if (status->finished_at != nullptr) {
          result.finished_at = static_cast<double>(*status->finished_at);
        } else {
          result.finished_at = std::nullopt;
        }

        return std::optional<LightningReceive>(result);
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::variant<nitro::NullType, std::string>>>
  checkLightningPayment(const std::string& paymentHash, bool wait) override {
    return Promise<std::variant<nitro::NullType, std::string>>::async([paymentHash, wait]() {
      try {
        rust::String result = bark_cxx::check_lightning_payment(paymentHash, wait);
        std::string preimage_str(result.data(), result.length());
        if (preimage_str.empty()) {
          return std::variant<nitro::NullType, std::string>(nitro::NullType());
        }
        return std::variant<nitro::NullType, std::string>(preimage_str);
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  // --- Ark Operations ---
  std::shared_ptr<Promise<BoardResult>> boardAmount(double amountSat) override {
    return Promise<BoardResult>::async([amountSat]() {
      try {
        bark_cxx::BoardResult result_rs = bark_cxx::board_amount(static_cast<uint64_t>(amountSat));
        BoardResult result;
        result.funding_txid = std::string(result_rs.funding_txid.data(), result_rs.funding_txid.length());
        std::vector<std::string> vtxos_vec;
        for (const auto& vtxo : result_rs.vtxos) {
          vtxos_vec.push_back(std::string(vtxo.data(), vtxo.length()));
        }
        result.vtxos = vtxos_vec;
        return result;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<BoardResult>> boardAll() override {
    return Promise<BoardResult>::async([]() {
      try {
        bark_cxx::BoardResult result_rs = bark_cxx::board_all();
        BoardResult result;
        result.funding_txid = std::string(result_rs.funding_txid.data(), result_rs.funding_txid.length());
        std::vector<std::string> vtxos_vec;
        for (const auto& vtxo : result_rs.vtxos) {
          vtxos_vec.push_back(std::string(vtxo.data(), vtxo.length()));
        }
        result.vtxos = vtxos_vec;
        return result;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<void>> validateArkoorAddress(const std::string& address) override {
    return Promise<void>::async([address]() {
      try {
        bark_cxx::validate_arkoor_address(address);
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<ArkoorPaymentResult>> sendArkoorPayment(const std::string& destination,
                                                                  double amountSat) override {
    return Promise<ArkoorPaymentResult>::async([destination, amountSat]() {
      try {
        bark_cxx::ArkoorPaymentResult rust_result =
            bark_cxx::send_arkoor_payment(destination, static_cast<uint64_t>(amountSat));

        ArkoorPaymentResult result;
        result.amount_sat = static_cast<double>(rust_result.amount_sat);
        result.destination_pubkey =
            std::string(rust_result.destination_pubkey.data(), rust_result.destination_pubkey.length());

        result.vtxos = convertRustVtxosToVector(rust_result.vtxos);

        return result;
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  // --- Offboarding / Exiting ---

  std::shared_ptr<Promise<std::string>> offboardSpecific(const std::vector<std::string>& vtxoIds,
                                                         const std::string& destinationAddress) override {
    return Promise<std::string>::async([vtxoIds, destinationAddress]() {
      try {
        rust::Vec<rust::String> rust_vtxo_ids;
        for (const auto& id : vtxoIds) {
          rust_vtxo_ids.push_back(rust::String(id));
        }
        rust::String result = bark_cxx::offboard_specific(std::move(rust_vtxo_ids), destinationAddress);
        return std::string(result);
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

  std::shared_ptr<Promise<std::string>> offboardAll(const std::string& destinationAddress) override {
    return Promise<std::string>::async([destinationAddress]() {
      try {
        rust::String result = bark_cxx::offboard_all(destinationAddress);
        return std::string(result);
      } catch (const rust::Error& e) {
        throw std::runtime_error(e.what());
      }
    });
  }

private:
  // Tag for logging/debugging within Nitro
  static constexpr auto TAG = "NitroArk";
};

} // namespace margelo::nitro::nitroark
