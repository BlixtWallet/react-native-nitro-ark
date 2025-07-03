// Auto-generated C/C++ bindings for bark-cpp

#ifndef BARK_CPP_H
#define BARK_CPP_H

/* Generated with cbindgen:0.28.0 */

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

namespace bark
{

  enum class bark_BarkRefreshModeType
  {
    DefaultThreshold,
    ThresholdBlocks,
    ThresholdHours,
    Counterparty,
    All,
    Specific,
  };

  struct bark_BarkError
  {
    char *message;
  };

  struct bark_BarkConfigOpts
  {
    const char *asp;
    const char *esplora;
    const char *bitcoind;
    const char *bitcoind_cookie;
    const char *bitcoind_user;
    const char *bitcoind_pass;
    uint32_t vtxo_refresh_expiry_threshold;
    const uint64_t *fallback_fee_rate;
  };

  struct bark_BarkCreateOpts
  {
    bool force;
    bool regtest;
    bool signet;
    bool bitcoin;
    const char *mnemonic;
    uint32_t birthday_height;
    bark_BarkConfigOpts config;
  };

  struct bark_BarkBalance
  {
    uint64_t onchain;
    uint64_t offchain;
    uint64_t pending_exit;
  };

  struct bark_BarkRefreshOpts
  {
    bark_BarkRefreshModeType mode_type;
    uint32_t threshold_value;
    const char *const *specific_vtxo_ids;
    uintptr_t num_specific_vtxo_ids;
  };

  extern "C"
  {

    /// Initializes the logger for the library.
    /// This should be called once when the library is loaded by the C/C++ application,
    /// before any other library functions are used.
    void bark_init_logger();

    void bark_free_error(bark_BarkError *error);

    const char *bark_error_message(const bark_BarkError *error);

    /// Frees a C string allocated by a bark-cpp function.
    void bark_free_string(char *s);

    /// Create a new mnemonic
    char *bark_create_mnemonic();

    /// Load an existing wallet or create a new one at the specified directory
    bark_BarkError *bark_load_wallet(const char *datadir, bark_BarkCreateOpts opts);

    /// Close the currently loaded wallet
    bark_BarkError *bark_close_wallet();

    /// Get offchain and onchain balances
    bark_BarkError *bark_get_balance(bool no_sync, bark_BarkBalance *balance_out);

    /// Get an onchain address.
    bark_BarkError *bark_get_onchain_address(char **address_out);

    /// Send funds using the onchain wallet.
    bark_BarkError *bark_send_onchain(const char *destination,
                                      uint64_t amount_sat,
                                      bool no_sync,
                                      char **txid_out);

    /// Send all funds from the onchain wallet to a destination address.
    bark_BarkError *bark_drain_onchain(const char *destination, bool no_sync, char **txid_out);

    /// Send funds to multiple recipients using the onchain wallet.
    bark_BarkError *bark_send_many_onchain(const char *const *destinations,
                                           const uint64_t *amounts_sat,
                                           uintptr_t num_outputs,
                                           bool no_sync,
                                           char **txid_out);

    /// Get the list of onchain UTXOs as a JSON string.
    bark_BarkError *bark_get_onchain_utxos(bool no_sync, char **utxos_json_out);

    /// Get the wallet's VTXO public key (hex string).
    bark_BarkError *bark_get_vtxo_pubkey(const uint32_t *index, char **pubkey_hex_out);

    /// Get the list of VTXOs as a JSON string.
    bark_BarkError *bark_get_vtxos(bool no_sync, char **vtxos_json_out);

    /// Refresh VTXOs based on specified criteria.
    bark_BarkError *bark_refresh_vtxos(bark_BarkRefreshOpts refresh_opts,
                                       bool no_sync,
                                       char **status_json_out);

    /// Board a specific amount from the onchain wallet into Ark.
    bark_BarkError *bark_board_amount(uint64_t amount_sat, bool no_sync, char **status_json_out);

    /// Board all available funds from the onchain wallet into Ark.
    bark_BarkError *bark_board_all(bool no_sync, char **status_json_out);

    bark_BarkError *bark_send(const char *destination,
                              uint64_t amount_sat,
                              const char *comment,
                              bool no_sync,
                              char **status_json_out);

    /// Send an onchain payment via an Ark round.
    bark_BarkError *bark_send_round_onchain(const char *destination,
                                            uint64_t amount_sat,
                                            bool no_sync,
                                            char **status_json_out);

    /// Offboard specific VTXOs to an optional onchain address.
    bark_BarkError *bark_offboard_specific(const char *const *specific_vtxo_ids,
                                           uintptr_t num_specific_vtxo_ids,
                                           const char *optional_address,
                                           bool no_sync,
                                           char **status_json_out);

    /// Offboard all VTXOs to an optional onchain address.
    bark_BarkError *bark_offboard_all(const char *optional_address,
                                      bool no_sync,
                                      char **status_json_out);

    /// Start the exit process for specific VTXOs.
    bark_BarkError *bark_exit_start_specific(const char *const *specific_vtxo_ids,
                                             uintptr_t num_specific_vtxo_ids,
                                             char **status_json_out);

    /// Start the exit process for all VTXOs in the wallet.
    bark_BarkError *bark_exit_start_all(char **status_json_out);

    /// Progress the exit process once and return the current status.
    bark_BarkError *bark_exit_progress_once(char **status_json_out);

    /// FFI: Creates a BOLT11 invoice for receiving payments.
    bark_BarkError *bark_bolt11_invoice(uint64_t amount_msat, char **invoice_out);

    /// FFI: Claims a BOLT11 payment using an invoice.
    bark_BarkError *bark_claim_bolt11_payment(const char *bolt11);

  } // extern "C"

} // namespace bark

#endif // BARK_CPP_H
