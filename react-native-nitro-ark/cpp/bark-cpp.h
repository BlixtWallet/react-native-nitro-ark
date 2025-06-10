// Auto-generated C/C++ bindings for bark-cpp

#ifndef BARK_CPP_H
#define BARK_CPP_H

/* Generated with cbindgen:0.28.0 */

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>
#include <ostream>

namespace bark {

enum class bark_BarkRefreshModeType {
  DefaultThreshold,
  ThresholdBlocks,
  ThresholdHours,
  Counterparty,
  All,
  Specific,
};

struct bark_BarkError {
  char *message;
};

struct bark_BarkConfigOpts {
  const char *asp;
  const char *esplora;
  const char *bitcoind;
  const char *bitcoind_cookie;
  const char *bitcoind_user;
  const char *bitcoind_pass;
};

struct bark_BarkCreateOpts {
  bool force;
  bool regtest;
  bool signet;
  bool bitcoin;
  const char *mnemonic;
  uint32_t birthday_height;
  bark_BarkConfigOpts config;
};

struct bark_BarkBalance {
  uint64_t onchain;
  uint64_t offchain;
  uint64_t pending_exit;
};

struct bark_BarkRefreshOpts {
  bark_BarkRefreshModeType mode_type;
  uint32_t threshold_value;
  const char *const *specific_vtxo_ids;
  uintptr_t num_specific_vtxo_ids;
};

extern "C" {

/// Initializes the logger for the library.
/// This should be called once when the library is loaded by the C/C++
/// application, before any other library functions are used.
void bark_init_logger();

void bark_free_error(bark_BarkError *error);

const char *bark_error_message(const bark_BarkError *error);

/// Frees a C string allocated by a bark-cpp function.
///
/// This function should be called by the C/C++ side on any `char*`
/// that was returned by functions like `bark_create_mnemonic`,
/// `bark_get_onchain_address`, `bark_send_onchain`, etc.
///
/// # Safety
///
/// The pointer `s` must have been previously allocated by Rust using
/// `CString::into_raw` or a similar mechanism within this library.
/// Calling this with a null pointer is safe (it does nothing).
/// Calling this with a pointer not allocated by this library, or calling
/// it more than once on the same pointer, results in undefined behavior.
void bark_free_string(char *s);

/// Create a new mnemonic
///
/// @return The mnemonic string as a C string, or NULL on error
char *bark_create_mnemonic();

/// Create a new wallet at the specified directory
///
/// @param datadir Path to the data directory
/// @param opts Creation options
/// @return Error pointer or NULL on success
bark_BarkError *bark_create_wallet(const char *datadir,
                                   bark_BarkCreateOpts opts);

/// Get offchain and onchain balances
///
/// @param datadir Path to the data directory
/// @param no_sync Whether to skip syncing the wallet
/// @param balance_out Pointer to a BarkBalance struct where the result will be
/// stored
/// @return Error pointer or NULL on success
bark_BarkError *bark_get_balance(const char *datadir, bool no_sync,
                                 const char *mnemonic,
                                 bark_BarkBalance *balance_out);

/// Get an onchain address.
///
/// The returned address string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param address_out Pointer to a `*mut c_char` where the address string
/// pointer will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_get_onchain_address(const char *datadir,
                                         const char *mnemonic,
                                         char **address_out);

/// Send funds using the onchain wallet.
///
/// The returned transaction ID string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param destination The destination Bitcoin address as a string
/// @param amount_sat The amount to send in satoshis
/// @param no_sync Whether to skip syncing the wallet before sending
/// @param txid_out Pointer to a `*mut c_char` where the transaction ID string
/// pointer will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_send_onchain(const char *datadir, const char *mnemonic,
                                  const char *destination, uint64_t amount_sat,
                                  bool no_sync, char **txid_out);

/// Send all funds from the onchain wallet to a destination address.
///
/// The returned transaction ID string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param destination The destination Bitcoin address as a string
/// @param no_sync Whether to skip syncing the wallet before sending
/// @param txid_out Pointer to a `*mut c_char` where the transaction ID string
/// pointer will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_drain_onchain(const char *datadir, const char *mnemonic,
                                   const char *destination, bool no_sync,
                                   char **txid_out);

/// Send funds to multiple recipients using the onchain wallet.
///
/// The returned transaction ID string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param destinations Array of C strings representing destination Bitcoin
/// addresses
/// @param amounts_sat Array of u64 representing amounts in satoshis (must match
/// destinations array length)
/// @param num_outputs The number of outputs (length of the destinations and
/// amounts_sat arrays)
/// @param no_sync Whether to skip syncing the wallet before sending
/// @param txid_out Pointer to a `*mut c_char` where the transaction ID string
/// pointer will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_send_many_onchain(const char *datadir,
                                       const char *mnemonic,
                                       const char *const *destinations,
                                       const uint64_t *amounts_sat,
                                       uintptr_t num_outputs, bool no_sync,
                                       char **txid_out);

/// Get the list of onchain UTXOs as a JSON string.
///
/// The returned JSON string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param no_sync Whether to skip syncing the wallet before fetching
/// @param utxos_json_out Pointer to a `*mut c_char` where the JSON string
/// pointer will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_get_onchain_utxos(const char *datadir,
                                       const char *mnemonic, bool no_sync,
                                       char **utxos_json_out);

/// Get the wallet's VTXO public key (hex string).
///
/// The returned public key string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param pubkey_hex_out Pointer to a `*mut c_char` where the hex string
/// pointer will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_get_vtxo_pubkey(const char *datadir, const char *mnemonic,
                                     char **pubkey_hex_out);

/// Get the list of VTXOs as a JSON string.
///
/// The returned JSON string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param no_sync Whether to skip syncing the wallet before fetching
/// @param vtxos_json_out Pointer to a `*mut c_char` where the JSON string
/// pointer will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_get_vtxos(const char *datadir, const char *mnemonic,
                               bool no_sync, char **vtxos_json_out);

/// Refresh VTXOs based on specified criteria.
///
/// The returned JSON status string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param refresh_opts Options specifying which VTXOs to refresh
/// @param no_sync Whether to skip syncing the wallet before refreshing
/// @param status_json_out Pointer to a `*mut c_char` where the JSON status
/// string will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_refresh_vtxos(const char *datadir, const char *mnemonic,
                                   bark_BarkRefreshOpts refresh_opts,
                                   bool no_sync, char **status_json_out);

/// Board a specific amount from the onchain wallet into Ark.
///
/// The returned JSON status string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param amount_sat The amount in satoshis to board
/// @param no_sync Whether to skip syncing the onchain wallet before boarding
/// @param status_json_out Pointer to a `*mut c_char` where the JSON status
/// string will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_board_amount(const char *datadir, const char *mnemonic,
                                  uint64_t amount_sat, bool no_sync,
                                  char **status_json_out);

/// Board all available funds from the onchain wallet into Ark.
///
/// The returned JSON status string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param no_sync Whether to skip syncing the onchain wallet before boarding
/// @param status_json_out Pointer to a `*mut c_char` where the JSON status
/// string will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_board_all(const char *datadir, const char *mnemonic,
                               bool no_sync, char **status_json_out);

bark_BarkError *bark_send(const char *datadir, const char *mnemonic,
                          const char *destination, uint64_t amount_sat,
                          const char *comment, bool no_sync,
                          char **status_json_out);

/// Send an onchain payment via an Ark round.
///
/// The returned JSON status string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param destination The destination Bitcoin address as a string
/// @param amount_sat The amount in satoshis to send
/// @param no_sync Whether to skip syncing the wallet before sending
/// @param status_json_out Pointer to a `*mut c_char` where the JSON status
/// string will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_send_round_onchain(const char *datadir,
                                        const char *mnemonic,
                                        const char *destination,
                                        uint64_t amount_sat, bool no_sync,
                                        char **status_json_out);

/// Offboard specific VTXOs to an optional onchain address.
///
/// The returned JSON result string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param specific_vtxo_ids Array of VtxoId strings (cannot be empty)
/// @param num_specific_vtxo_ids Number of VtxoIds in the array
/// @param optional_address Optional destination Bitcoin address (pass NULL if
/// not provided)
/// @param no_sync Whether to skip syncing the wallet
/// @param status_json_out Pointer to a `*mut c_char` where the JSON result
/// string will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_offboard_specific(const char *datadir,
                                       const char *mnemonic,
                                       const char *const *specific_vtxo_ids,
                                       uintptr_t num_specific_vtxo_ids,
                                       const char *optional_address,
                                       bool no_sync, char **status_json_out);

/// Offboard all VTXOs to an optional onchain address.
///
/// The returned JSON result string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param optional_address Optional destination Bitcoin address (pass NULL if
/// not provided)
/// @param no_sync Whether to skip syncing the wallet
/// @param status_json_out Pointer to a `*mut c_char` where the JSON result
/// string will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_offboard_all(const char *datadir, const char *mnemonic,
                                  const char *optional_address, bool no_sync,
                                  char **status_json_out);

/// Start the exit process for specific VTXOs.
///
/// The returned JSON success string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param specific_vtxo_ids Array of VtxoId strings (cannot be empty)
/// @param num_specific_vtxo_ids Number of VtxoIds in the array
/// @param status_json_out Pointer to a `*mut c_char` where the JSON success
/// string will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_exit_start_specific(const char *datadir,
                                         const char *mnemonic,
                                         const char *const *specific_vtxo_ids,
                                         uintptr_t num_specific_vtxo_ids,
                                         char **status_json_out);

/// Start the exit process for all VTXOs in the wallet.
///
/// The returned JSON success string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param status_json_out Pointer to a `*mut c_char` where the JSON success
/// string will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_exit_start_all(const char *datadir, const char *mnemonic,
                                    char **status_json_out);

/// Progress the exit process once and return the current status.
///
/// The returned JSON status string must be freed by the caller using
/// `bark_free_string`.
///
/// @param datadir Path to the data directory
/// @param mnemonic The wallet mnemonic phrase
/// @param status_json_out Pointer to a `*mut c_char` where the JSON status
/// string will be written.
/// @return Error pointer or NULL on success.
bark_BarkError *bark_exit_progress_once(const char *datadir,
                                        const char *mnemonic,
                                        char **status_json_out);

} // extern "C"

} // namespace bark

#endif // BARK_CPP_H
