#pragma once
#include <algorithm>
#include <array>
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <initializer_list>
#include <iterator>
#include <new>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <utility>
#if __cplusplus >= 201703L
#include <string_view>
#endif
#if __cplusplus >= 202002L
#include <ranges>
#endif

namespace rust {
inline namespace cxxbridge1 {
// #include "rust/cxx.h"

#ifndef CXXBRIDGE1_PANIC
#define CXXBRIDGE1_PANIC
template <typename Exception>
void panic [[noreturn]] (const char *msg);
#endif // CXXBRIDGE1_PANIC

struct unsafe_bitcopy_t;

namespace {
template <typename T>
class impl;
} // namespace

class Opaque;

template <typename T>
::std::size_t size_of();
template <typename T>
::std::size_t align_of();

#ifndef CXXBRIDGE1_RUST_STRING
#define CXXBRIDGE1_RUST_STRING
class String final {
public:
  String() noexcept;
  String(const String &) noexcept;
  String(String &&) noexcept;
  ~String() noexcept;

  String(const std::string &);
  String(const char *);
  String(const char *, std::size_t);
  String(const char16_t *);
  String(const char16_t *, std::size_t);
#ifdef __cpp_char8_t
  String(const char8_t *s);
  String(const char8_t *s, std::size_t len);
#endif

  static String lossy(const std::string &) noexcept;
  static String lossy(const char *) noexcept;
  static String lossy(const char *, std::size_t) noexcept;
  static String lossy(const char16_t *) noexcept;
  static String lossy(const char16_t *, std::size_t) noexcept;

  String &operator=(const String &) & noexcept;
  String &operator=(String &&) & noexcept;

  explicit operator std::string() const;

  const char *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  const char *c_str() noexcept;

  std::size_t capacity() const noexcept;
  void reserve(size_t new_cap) noexcept;

  using iterator = char *;
  iterator begin() noexcept;
  iterator end() noexcept;

  using const_iterator = const char *;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  bool operator==(const String &) const noexcept;
  bool operator!=(const String &) const noexcept;
  bool operator<(const String &) const noexcept;
  bool operator<=(const String &) const noexcept;
  bool operator>(const String &) const noexcept;
  bool operator>=(const String &) const noexcept;

  void swap(String &) noexcept;

  String(unsafe_bitcopy_t, const String &) noexcept;

private:
  struct lossy_t;
  String(lossy_t, const char *, std::size_t) noexcept;
  String(lossy_t, const char16_t *, std::size_t) noexcept;
  friend void swap(String &lhs, String &rhs) noexcept { lhs.swap(rhs); }

  std::array<std::uintptr_t, 3> repr;
};
#endif // CXXBRIDGE1_RUST_STRING

#ifndef CXXBRIDGE1_RUST_STR
#define CXXBRIDGE1_RUST_STR
class Str final {
public:
  Str() noexcept;
  Str(const String &) noexcept;
  Str(const std::string &);
  Str(const char *);
  Str(const char *, std::size_t);

  Str &operator=(const Str &) & noexcept = default;

  explicit operator std::string() const;
#if __cplusplus >= 201703L
  explicit operator std::string_view() const;
#endif

  const char *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  Str(const Str &) noexcept = default;
  ~Str() noexcept = default;

  using iterator = const char *;
  using const_iterator = const char *;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  bool operator==(const Str &) const noexcept;
  bool operator!=(const Str &) const noexcept;
  bool operator<(const Str &) const noexcept;
  bool operator<=(const Str &) const noexcept;
  bool operator>(const Str &) const noexcept;
  bool operator>=(const Str &) const noexcept;

  void swap(Str &) noexcept;

private:
  class uninit;
  Str(uninit) noexcept;
  friend impl<Str>;

  std::array<std::uintptr_t, 2> repr;
};
#endif // CXXBRIDGE1_RUST_STR

#ifndef CXXBRIDGE1_RUST_SLICE
#define CXXBRIDGE1_RUST_SLICE
namespace detail {
template <bool>
struct copy_assignable_if {};

template <>
struct copy_assignable_if<false> {
  copy_assignable_if() noexcept = default;
  copy_assignable_if(const copy_assignable_if &) noexcept = default;
  copy_assignable_if &operator=(const copy_assignable_if &) & noexcept = delete;
  copy_assignable_if &operator=(copy_assignable_if &&) & noexcept = default;
};
} // namespace detail

template <typename T>
class Slice final
    : private detail::copy_assignable_if<std::is_const<T>::value> {
public:
  using value_type = T;

  Slice() noexcept;
  Slice(T *, std::size_t count) noexcept;

  template <typename C>
  explicit Slice(C &c) : Slice(c.data(), c.size()) {}

  Slice &operator=(const Slice<T> &) & noexcept = default;
  Slice &operator=(Slice<T> &&) & noexcept = default;

  T *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  T &operator[](std::size_t n) const noexcept;
  T &at(std::size_t n) const;
  T &front() const noexcept;
  T &back() const noexcept;

  Slice(const Slice<T> &) noexcept = default;
  ~Slice() noexcept = default;

  class iterator;
  iterator begin() const noexcept;
  iterator end() const noexcept;

  void swap(Slice &) noexcept;

private:
  class uninit;
  Slice(uninit) noexcept;
  friend impl<Slice>;
  friend void sliceInit(void *, const void *, std::size_t) noexcept;
  friend void *slicePtr(const void *) noexcept;
  friend std::size_t sliceLen(const void *) noexcept;

  std::array<std::uintptr_t, 2> repr;
};

#ifdef __cpp_deduction_guides
template <typename C>
explicit Slice(C &c)
    -> Slice<std::remove_reference_t<decltype(*std::declval<C>().data())>>;
#endif // __cpp_deduction_guides

template <typename T>
class Slice<T>::iterator final {
public:
#if __cplusplus >= 202002L
  using iterator_category = std::contiguous_iterator_tag;
#else
  using iterator_category = std::random_access_iterator_tag;
#endif
  using value_type = T;
  using difference_type = std::ptrdiff_t;
  using pointer = typename std::add_pointer<T>::type;
  using reference = typename std::add_lvalue_reference<T>::type;

  reference operator*() const noexcept;
  pointer operator->() const noexcept;
  reference operator[](difference_type) const noexcept;

  iterator &operator++() noexcept;
  iterator operator++(int) noexcept;
  iterator &operator--() noexcept;
  iterator operator--(int) noexcept;

  iterator &operator+=(difference_type) noexcept;
  iterator &operator-=(difference_type) noexcept;
  iterator operator+(difference_type) const noexcept;
  friend inline iterator operator+(difference_type lhs, iterator rhs) noexcept {
    return rhs + lhs;
  }
  iterator operator-(difference_type) const noexcept;
  difference_type operator-(const iterator &) const noexcept;

  bool operator==(const iterator &) const noexcept;
  bool operator!=(const iterator &) const noexcept;
  bool operator<(const iterator &) const noexcept;
  bool operator<=(const iterator &) const noexcept;
  bool operator>(const iterator &) const noexcept;
  bool operator>=(const iterator &) const noexcept;

private:
  friend class Slice;
  void *pos;
  std::size_t stride;
};

#if __cplusplus >= 202002L
static_assert(std::ranges::contiguous_range<rust::Slice<const uint8_t>>);
static_assert(std::contiguous_iterator<rust::Slice<const uint8_t>::iterator>);
#endif

template <typename T>
Slice<T>::Slice() noexcept {
  sliceInit(this, reinterpret_cast<void *>(align_of<T>()), 0);
}

template <typename T>
Slice<T>::Slice(T *s, std::size_t count) noexcept {
  assert(s != nullptr || count == 0);
  sliceInit(this,
            s == nullptr && count == 0
                ? reinterpret_cast<void *>(align_of<T>())
                : const_cast<typename std::remove_const<T>::type *>(s),
            count);
}

template <typename T>
T *Slice<T>::data() const noexcept {
  return reinterpret_cast<T *>(slicePtr(this));
}

template <typename T>
std::size_t Slice<T>::size() const noexcept {
  return sliceLen(this);
}

template <typename T>
std::size_t Slice<T>::length() const noexcept {
  return this->size();
}

template <typename T>
bool Slice<T>::empty() const noexcept {
  return this->size() == 0;
}

template <typename T>
T &Slice<T>::operator[](std::size_t n) const noexcept {
  assert(n < this->size());
  auto ptr = static_cast<char *>(slicePtr(this)) + size_of<T>() * n;
  return *reinterpret_cast<T *>(ptr);
}

template <typename T>
T &Slice<T>::at(std::size_t n) const {
  if (n >= this->size()) {
    panic<std::out_of_range>("rust::Slice index out of range");
  }
  return (*this)[n];
}

template <typename T>
T &Slice<T>::front() const noexcept {
  assert(!this->empty());
  return (*this)[0];
}

template <typename T>
T &Slice<T>::back() const noexcept {
  assert(!this->empty());
  return (*this)[this->size() - 1];
}

template <typename T>
typename Slice<T>::iterator::reference
Slice<T>::iterator::operator*() const noexcept {
  return *static_cast<T *>(this->pos);
}

template <typename T>
typename Slice<T>::iterator::pointer
Slice<T>::iterator::operator->() const noexcept {
  return static_cast<T *>(this->pos);
}

template <typename T>
typename Slice<T>::iterator::reference Slice<T>::iterator::operator[](
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ptr = static_cast<char *>(this->pos) + this->stride * n;
  return *reinterpret_cast<T *>(ptr);
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator++() noexcept {
  this->pos = static_cast<char *>(this->pos) + this->stride;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator++(int) noexcept {
  auto ret = iterator(*this);
  this->pos = static_cast<char *>(this->pos) + this->stride;
  return ret;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator--() noexcept {
  this->pos = static_cast<char *>(this->pos) - this->stride;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator--(int) noexcept {
  auto ret = iterator(*this);
  this->pos = static_cast<char *>(this->pos) - this->stride;
  return ret;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator+=(
    typename Slice<T>::iterator::difference_type n) noexcept {
  this->pos = static_cast<char *>(this->pos) + this->stride * n;
  return *this;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator-=(
    typename Slice<T>::iterator::difference_type n) noexcept {
  this->pos = static_cast<char *>(this->pos) - this->stride * n;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator+(
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ret = iterator(*this);
  ret.pos = static_cast<char *>(this->pos) + this->stride * n;
  return ret;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator-(
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ret = iterator(*this);
  ret.pos = static_cast<char *>(this->pos) - this->stride * n;
  return ret;
}

template <typename T>
typename Slice<T>::iterator::difference_type
Slice<T>::iterator::operator-(const iterator &other) const noexcept {
  auto diff = std::distance(static_cast<char *>(other.pos),
                            static_cast<char *>(this->pos));
  return diff / static_cast<typename Slice<T>::iterator::difference_type>(
                    this->stride);
}

template <typename T>
bool Slice<T>::iterator::operator==(const iterator &other) const noexcept {
  return this->pos == other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator!=(const iterator &other) const noexcept {
  return this->pos != other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator<(const iterator &other) const noexcept {
  return this->pos < other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator<=(const iterator &other) const noexcept {
  return this->pos <= other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator>(const iterator &other) const noexcept {
  return this->pos > other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator>=(const iterator &other) const noexcept {
  return this->pos >= other.pos;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::begin() const noexcept {
  iterator it;
  it.pos = slicePtr(this);
  it.stride = size_of<T>();
  return it;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::end() const noexcept {
  iterator it = this->begin();
  it.pos = static_cast<char *>(it.pos) + it.stride * this->size();
  return it;
}

template <typename T>
void Slice<T>::swap(Slice &rhs) noexcept {
  std::swap(*this, rhs);
}
#endif // CXXBRIDGE1_RUST_SLICE

#ifndef CXXBRIDGE1_RUST_BITCOPY_T
#define CXXBRIDGE1_RUST_BITCOPY_T
struct unsafe_bitcopy_t final {
  explicit unsafe_bitcopy_t() = default;
};
#endif // CXXBRIDGE1_RUST_BITCOPY_T

#ifndef CXXBRIDGE1_RUST_VEC
#define CXXBRIDGE1_RUST_VEC
template <typename T>
class Vec final {
public:
  using value_type = T;

  Vec() noexcept;
  Vec(std::initializer_list<T>);
  Vec(const Vec &);
  Vec(Vec &&) noexcept;
  ~Vec() noexcept;

  Vec &operator=(Vec &&) & noexcept;
  Vec &operator=(const Vec &) &;

  std::size_t size() const noexcept;
  bool empty() const noexcept;
  const T *data() const noexcept;
  T *data() noexcept;
  std::size_t capacity() const noexcept;

  const T &operator[](std::size_t n) const noexcept;
  const T &at(std::size_t n) const;
  const T &front() const noexcept;
  const T &back() const noexcept;

  T &operator[](std::size_t n) noexcept;
  T &at(std::size_t n);
  T &front() noexcept;
  T &back() noexcept;

  void reserve(std::size_t new_cap);
  void push_back(const T &value);
  void push_back(T &&value);
  template <typename... Args>
  void emplace_back(Args &&...args);
  void truncate(std::size_t len);
  void clear();

  using iterator = typename Slice<T>::iterator;
  iterator begin() noexcept;
  iterator end() noexcept;

  using const_iterator = typename Slice<const T>::iterator;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  void swap(Vec &) noexcept;

  Vec(unsafe_bitcopy_t, const Vec &) noexcept;

private:
  void reserve_total(std::size_t new_cap) noexcept;
  void set_len(std::size_t len) noexcept;
  void drop() noexcept;

  friend void swap(Vec &lhs, Vec &rhs) noexcept { lhs.swap(rhs); }

  std::array<std::uintptr_t, 3> repr;
};

template <typename T>
Vec<T>::Vec(std::initializer_list<T> init) : Vec{} {
  this->reserve_total(init.size());
  std::move(init.begin(), init.end(), std::back_inserter(*this));
}

template <typename T>
Vec<T>::Vec(const Vec &other) : Vec() {
  this->reserve_total(other.size());
  std::copy(other.begin(), other.end(), std::back_inserter(*this));
}

template <typename T>
Vec<T>::Vec(Vec &&other) noexcept : repr(other.repr) {
  new (&other) Vec();
}

template <typename T>
Vec<T>::~Vec() noexcept {
  this->drop();
}

template <typename T>
Vec<T> &Vec<T>::operator=(Vec &&other) & noexcept {
  this->drop();
  this->repr = other.repr;
  new (&other) Vec();
  return *this;
}

template <typename T>
Vec<T> &Vec<T>::operator=(const Vec &other) & {
  if (this != &other) {
    this->drop();
    new (this) Vec(other);
  }
  return *this;
}

template <typename T>
bool Vec<T>::empty() const noexcept {
  return this->size() == 0;
}

template <typename T>
T *Vec<T>::data() noexcept {
  return const_cast<T *>(const_cast<const Vec<T> *>(this)->data());
}

template <typename T>
const T &Vec<T>::operator[](std::size_t n) const noexcept {
  assert(n < this->size());
  auto data = reinterpret_cast<const char *>(this->data());
  return *reinterpret_cast<const T *>(data + n * size_of<T>());
}

template <typename T>
const T &Vec<T>::at(std::size_t n) const {
  if (n >= this->size()) {
    panic<std::out_of_range>("rust::Vec index out of range");
  }
  return (*this)[n];
}

template <typename T>
const T &Vec<T>::front() const noexcept {
  assert(!this->empty());
  return (*this)[0];
}

template <typename T>
const T &Vec<T>::back() const noexcept {
  assert(!this->empty());
  return (*this)[this->size() - 1];
}

template <typename T>
T &Vec<T>::operator[](std::size_t n) noexcept {
  assert(n < this->size());
  auto data = reinterpret_cast<char *>(this->data());
  return *reinterpret_cast<T *>(data + n * size_of<T>());
}

template <typename T>
T &Vec<T>::at(std::size_t n) {
  if (n >= this->size()) {
    panic<std::out_of_range>("rust::Vec index out of range");
  }
  return (*this)[n];
}

template <typename T>
T &Vec<T>::front() noexcept {
  assert(!this->empty());
  return (*this)[0];
}

template <typename T>
T &Vec<T>::back() noexcept {
  assert(!this->empty());
  return (*this)[this->size() - 1];
}

template <typename T>
void Vec<T>::reserve(std::size_t new_cap) {
  this->reserve_total(new_cap);
}

template <typename T>
void Vec<T>::push_back(const T &value) {
  this->emplace_back(value);
}

template <typename T>
void Vec<T>::push_back(T &&value) {
  this->emplace_back(std::move(value));
}

template <typename T>
template <typename... Args>
void Vec<T>::emplace_back(Args &&...args) {
  auto size = this->size();
  this->reserve_total(size + 1);
  ::new (reinterpret_cast<T *>(reinterpret_cast<char *>(this->data()) +
                               size * size_of<T>()))
      T(std::forward<Args>(args)...);
  this->set_len(size + 1);
}

template <typename T>
void Vec<T>::clear() {
  this->truncate(0);
}

template <typename T>
typename Vec<T>::iterator Vec<T>::begin() noexcept {
  return Slice<T>(this->data(), this->size()).begin();
}

template <typename T>
typename Vec<T>::iterator Vec<T>::end() noexcept {
  return Slice<T>(this->data(), this->size()).end();
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::begin() const noexcept {
  return this->cbegin();
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::end() const noexcept {
  return this->cend();
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::cbegin() const noexcept {
  return Slice<const T>(this->data(), this->size()).begin();
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::cend() const noexcept {
  return Slice<const T>(this->data(), this->size()).end();
}

template <typename T>
void Vec<T>::swap(Vec &rhs) noexcept {
  using std::swap;
  swap(this->repr, rhs.repr);
}

template <typename T>
Vec<T>::Vec(unsafe_bitcopy_t, const Vec &bits) noexcept : repr(bits.repr) {}
#endif // CXXBRIDGE1_RUST_VEC

#ifndef CXXBRIDGE1_IS_COMPLETE
#define CXXBRIDGE1_IS_COMPLETE
namespace detail {
namespace {
template <typename T, typename = std::size_t>
struct is_complete : std::false_type {};
template <typename T>
struct is_complete<T, decltype(sizeof(T))> : std::true_type {};
} // namespace
} // namespace detail
#endif // CXXBRIDGE1_IS_COMPLETE

#ifndef CXXBRIDGE1_LAYOUT
#define CXXBRIDGE1_LAYOUT
class layout {
  template <typename T>
  friend std::size_t size_of();
  template <typename T>
  friend std::size_t align_of();
  template <typename T>
  static typename std::enable_if<std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_size_of() {
    return T::layout::size();
  }
  template <typename T>
  static typename std::enable_if<!std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_size_of() {
    return sizeof(T);
  }
  template <typename T>
  static
      typename std::enable_if<detail::is_complete<T>::value, std::size_t>::type
      size_of() {
    return do_size_of<T>();
  }
  template <typename T>
  static typename std::enable_if<std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_align_of() {
    return T::layout::align();
  }
  template <typename T>
  static typename std::enable_if<!std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_align_of() {
    return alignof(T);
  }
  template <typename T>
  static
      typename std::enable_if<detail::is_complete<T>::value, std::size_t>::type
      align_of() {
    return do_align_of<T>();
  }
};

template <typename T>
std::size_t size_of() {
  return layout::size_of<T>();
}

template <typename T>
std::size_t align_of() {
  return layout::align_of<T>();
}
#endif // CXXBRIDGE1_LAYOUT
} // namespace cxxbridge1
} // namespace rust

#if __cplusplus >= 201402L
#define CXX_DEFAULT_VALUE(value) = value
#else
#define CXX_DEFAULT_VALUE(value)
#endif

namespace bark_cxx {
  struct BarkVtxo;
  enum class PaymentTypes : ::std::uint8_t;
  struct NewAddressResult;
  struct Bolt11PaymentResult;
  struct LnurlPaymentResult;
  struct ArkoorPaymentResult;
  struct OnchainPaymentResult;
  struct CxxArkInfo;
  struct ConfigOpts;
  struct CreateOpts;
  struct SendManyOutput;
  enum class RefreshModeType : ::std::uint8_t;
  struct OffchainBalance;
  struct OnChainBalance;
  struct KeyPairResult;
}

namespace bark_cxx {
#ifndef CXXBRIDGE1_STRUCT_bark_cxx$BarkVtxo
#define CXXBRIDGE1_STRUCT_bark_cxx$BarkVtxo
struct BarkVtxo final {
  ::std::uint64_t amount CXX_DEFAULT_VALUE(0);
  ::std::uint32_t expiry_height CXX_DEFAULT_VALUE(0);
  ::rust::String server_pubkey;
  ::std::uint16_t exit_delta CXX_DEFAULT_VALUE(0);
  ::rust::String anchor_point;
  ::rust::String point;

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_bark_cxx$BarkVtxo

#ifndef CXXBRIDGE1_ENUM_bark_cxx$PaymentTypes
#define CXXBRIDGE1_ENUM_bark_cxx$PaymentTypes
enum class PaymentTypes : ::std::uint8_t {
  Bolt11 = 0,
  Lnurl = 1,
  Arkoor = 2,
  Onchain = 3,
};
#endif // CXXBRIDGE1_ENUM_bark_cxx$PaymentTypes

#ifndef CXXBRIDGE1_STRUCT_bark_cxx$NewAddressResult
#define CXXBRIDGE1_STRUCT_bark_cxx$NewAddressResult
struct NewAddressResult final {
  ::rust::String user_pubkey;
  ::rust::String ark_id;
  ::rust::String address;

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_bark_cxx$NewAddressResult

#ifndef CXXBRIDGE1_STRUCT_bark_cxx$Bolt11PaymentResult
#define CXXBRIDGE1_STRUCT_bark_cxx$Bolt11PaymentResult
struct Bolt11PaymentResult final {
  ::rust::String bolt11_invoice;
  ::rust::String preimage;
  ::bark_cxx::PaymentTypes payment_type;

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_bark_cxx$Bolt11PaymentResult

#ifndef CXXBRIDGE1_STRUCT_bark_cxx$LnurlPaymentResult
#define CXXBRIDGE1_STRUCT_bark_cxx$LnurlPaymentResult
struct LnurlPaymentResult final {
  ::rust::String lnurl;
  ::rust::String bolt11_invoice;
  ::rust::String preimage;
  ::bark_cxx::PaymentTypes payment_type;

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_bark_cxx$LnurlPaymentResult

#ifndef CXXBRIDGE1_STRUCT_bark_cxx$ArkoorPaymentResult
#define CXXBRIDGE1_STRUCT_bark_cxx$ArkoorPaymentResult
struct ArkoorPaymentResult final {
  ::std::uint64_t amount_sat CXX_DEFAULT_VALUE(0);
  ::rust::String destination_pubkey;
  ::bark_cxx::PaymentTypes payment_type;
  ::rust::Vec<::bark_cxx::BarkVtxo> vtxos;

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_bark_cxx$ArkoorPaymentResult

#ifndef CXXBRIDGE1_STRUCT_bark_cxx$OnchainPaymentResult
#define CXXBRIDGE1_STRUCT_bark_cxx$OnchainPaymentResult
struct OnchainPaymentResult final {
  ::rust::String txid;
  ::std::uint64_t amount_sat CXX_DEFAULT_VALUE(0);
  ::rust::String destination_address;
  ::bark_cxx::PaymentTypes payment_type;

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_bark_cxx$OnchainPaymentResult

#ifndef CXXBRIDGE1_STRUCT_bark_cxx$CxxArkInfo
#define CXXBRIDGE1_STRUCT_bark_cxx$CxxArkInfo
struct CxxArkInfo final {
  ::rust::String network;
  ::rust::String server_pubkey;
  ::std::uint64_t round_interval_secs CXX_DEFAULT_VALUE(0);
  ::std::uint16_t vtxo_exit_delta CXX_DEFAULT_VALUE(0);
  ::std::uint16_t vtxo_expiry_delta CXX_DEFAULT_VALUE(0);
  ::std::uint16_t htlc_expiry_delta CXX_DEFAULT_VALUE(0);
  ::std::uint64_t max_vtxo_amount_sat CXX_DEFAULT_VALUE(0);

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_bark_cxx$CxxArkInfo

#ifndef CXXBRIDGE1_STRUCT_bark_cxx$ConfigOpts
#define CXXBRIDGE1_STRUCT_bark_cxx$ConfigOpts
struct ConfigOpts final {
  ::rust::String ark;
  ::rust::String esplora;
  ::rust::String bitcoind;
  ::rust::String bitcoind_cookie;
  ::rust::String bitcoind_user;
  ::rust::String bitcoind_pass;
  ::std::uint32_t vtxo_refresh_expiry_threshold CXX_DEFAULT_VALUE(0);
  ::std::uint64_t fallback_fee_rate CXX_DEFAULT_VALUE(0);

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_bark_cxx$ConfigOpts

#ifndef CXXBRIDGE1_STRUCT_bark_cxx$CreateOpts
#define CXXBRIDGE1_STRUCT_bark_cxx$CreateOpts
struct CreateOpts final {
  bool regtest CXX_DEFAULT_VALUE(false);
  bool signet CXX_DEFAULT_VALUE(false);
  bool bitcoin CXX_DEFAULT_VALUE(false);
  ::rust::String mnemonic;
  ::std::uint32_t const *birthday_height CXX_DEFAULT_VALUE(nullptr);
  ::bark_cxx::ConfigOpts config;

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_bark_cxx$CreateOpts

#ifndef CXXBRIDGE1_STRUCT_bark_cxx$SendManyOutput
#define CXXBRIDGE1_STRUCT_bark_cxx$SendManyOutput
struct SendManyOutput final {
  ::rust::String destination;
  ::std::uint64_t amount_sat CXX_DEFAULT_VALUE(0);

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_bark_cxx$SendManyOutput

#ifndef CXXBRIDGE1_ENUM_bark_cxx$RefreshModeType
#define CXXBRIDGE1_ENUM_bark_cxx$RefreshModeType
enum class RefreshModeType : ::std::uint8_t {
  DefaultThreshold = 0,
  ThresholdBlocks = 1,
  ThresholdHours = 2,
  Counterparty = 3,
  All = 4,
  Specific = 5,
};
#endif // CXXBRIDGE1_ENUM_bark_cxx$RefreshModeType

#ifndef CXXBRIDGE1_STRUCT_bark_cxx$OffchainBalance
#define CXXBRIDGE1_STRUCT_bark_cxx$OffchainBalance
struct OffchainBalance final {
  // Coins that are spendable in the Ark, either in-round or out-of-round.
  ::std::uint64_t spendable CXX_DEFAULT_VALUE(0);
  // Coins that are in the process of being sent over Lightning.
  ::std::uint64_t pending_lightning_send CXX_DEFAULT_VALUE(0);
  // Coins locked in a round.
  ::std::uint64_t pending_in_round CXX_DEFAULT_VALUE(0);
  // Coins that are in the process of unilaterally exiting the Ark.
  ::std::uint64_t pending_exit CXX_DEFAULT_VALUE(0);

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_bark_cxx$OffchainBalance

#ifndef CXXBRIDGE1_STRUCT_bark_cxx$OnChainBalance
#define CXXBRIDGE1_STRUCT_bark_cxx$OnChainBalance
struct OnChainBalance final {
  // All coinbase outputs not yet matured
  ::std::uint64_t immature CXX_DEFAULT_VALUE(0);
  // Unconfirmed UTXOs generated by a wallet tx
  ::std::uint64_t trusted_pending CXX_DEFAULT_VALUE(0);
  // Unconfirmed UTXOs received from an external wallet
  ::std::uint64_t untrusted_pending CXX_DEFAULT_VALUE(0);
  // Confirmed and immediately spendable balance
  ::std::uint64_t confirmed CXX_DEFAULT_VALUE(0);

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_bark_cxx$OnChainBalance

#ifndef CXXBRIDGE1_STRUCT_bark_cxx$KeyPairResult
#define CXXBRIDGE1_STRUCT_bark_cxx$KeyPairResult
struct KeyPairResult final {
  ::rust::String public_key;
  ::rust::String secret_key;

  using IsRelocatable = ::std::true_type;
};
#endif // CXXBRIDGE1_STRUCT_bark_cxx$KeyPairResult

void init_logger() noexcept;

::rust::String create_mnemonic();

bool is_wallet_loaded() noexcept;

void close_wallet();

void persist_config(::bark_cxx::ConfigOpts opts);

::bark_cxx::CxxArkInfo get_ark_info();

::bark_cxx::OffchainBalance offchain_balance();

::bark_cxx::KeyPairResult derive_store_next_keypair();

::bark_cxx::KeyPairResult peak_keypair(::std::uint32_t index);

::bark_cxx::NewAddressResult new_address();

::rust::String sign_message(::rust::Str message, ::std::uint32_t index);

::rust::String sign_messsage_with_mnemonic(::rust::Str message, ::rust::Str mnemonic, ::rust::Str network, ::std::uint32_t index);

::bark_cxx::KeyPairResult derive_keypair_from_mnemonic(::rust::Str mnemonic, ::rust::Str network, ::std::uint32_t index);

bool verify_message(::rust::Str message, ::rust::Str signature, ::rust::Str public_key);

::rust::Vec<::bark_cxx::BarkVtxo> get_vtxos();

::rust::Vec<::bark_cxx::BarkVtxo> get_expiring_vtxos(::std::uint32_t threshold);

::rust::String bolt11_invoice(::std::uint64_t amount_msat);

void maintenance();

void maintenance_refresh();

void sync();

void sync_rounds();

void create_wallet(::rust::Str datadir, ::bark_cxx::CreateOpts opts);

void load_wallet(::rust::Str datadir, ::rust::Str mnemonic);

::rust::String board_amount(::std::uint64_t amount_sat);

::rust::String board_all();

void validate_arkoor_address(::rust::Str address);

::bark_cxx::ArkoorPaymentResult send_arkoor_payment(::rust::Str destination, ::std::uint64_t amount_sat);

::bark_cxx::Bolt11PaymentResult send_lightning_payment(::rust::Str destination, ::std::uint64_t const *amount_sat);

::bark_cxx::LnurlPaymentResult send_lnaddr(::rust::Str addr, ::std::uint64_t amount_sat, ::rust::Str comment);

::rust::String send_round_onchain_payment(::rust::Str destination, ::std::uint64_t amount_sat);

::rust::String offboard_specific(::rust::Vec<::rust::String> vtxo_ids, ::rust::Str destination_address);

::rust::String offboard_all(::rust::Str destination_address);

void finish_lightning_receive(::rust::String bolt11);

void sync_exits();

::bark_cxx::OnChainBalance onchain_balance();

void onchain_sync();

::rust::String onchain_list_unspent();

::rust::String onchain_utxos();

::rust::String onchain_address();

::bark_cxx::OnchainPaymentResult onchain_send(::rust::Str destination, ::std::uint64_t amount_sat, ::std::uint64_t const *fee_rate);

::rust::String onchain_drain(::rust::Str destination, ::std::uint64_t const *fee_rate);

::rust::String onchain_send_many(::rust::Vec<::bark_cxx::SendManyOutput> outputs, ::std::uint64_t const *fee_rate);
} // namespace bark_cxx
