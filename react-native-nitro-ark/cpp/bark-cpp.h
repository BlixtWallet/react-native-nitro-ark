// Auto-generated C/C++ bindings for bark-cpp

#ifndef BARK_CPP_H
#define BARK_CPP_H

/* Generated with cbindgen:0.28.0 */

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

namespace bark {

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
  uint64_t birthday_height;
  bark_BarkConfigOpts config;
};

struct bark_BarkBalance {
  uint64_t onchain;
  uint64_t offchain;
  uint64_t pending_exit;
};

extern "C" {

void bark_free_error(bark_BarkError *error);

const char *bark_error_message(const bark_BarkError *error);

/// Create a new wallet at the specified directory
///
/// @param datadir Path to the data directory
/// @param opts Creation options
/// @return Error pointer or NULL on success
bark_BarkError *bark_create_wallet(const char *datadir, bark_BarkCreateOpts opts);

/// Get offchain and onchain balances
///
/// @param datadir Path to the data directory
/// @param no_sync Whether to skip syncing the wallet
/// @param balance_out Pointer to a BarkBalance struct where the result will be stored
/// @return Error pointer or NULL on success
bark_BarkError *bark_get_balance(const char *datadir, bool no_sync, bark_BarkBalance *balance_out);

}  // extern "C"

}  // namespace bark

#endif  // BARK_CPP_H
