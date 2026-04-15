#ifndef SIMDUTF_H
#define SIMDUTF_H
#include <cstddef>
#include <cstdint>
namespace simdutf {
enum encoding_type { UTF8, UTF16LE, UTF16BE, UTF32 };
inline bool validate_utf16le(const char16_t* buf, size_t len) { (void)buf; (void)len; return true; }
inline bool validate_utf16be(const char16_t* buf, size_t len) { (void)buf; (void)len; return true; }
inline size_t utf8_length_from_utf16(const char16_t* buf, size_t len) { (void)buf; return len * 3; }
inline bool validate_ascii(const char* buf, size_t len) { (void)buf; (void)len; return true; }
inline bool validate_utf8(const char* buf, size_t len) { (void)buf; (void)len; return true; }
inline size_t convert_utf8_to_utf16le(const char* in, size_t len, char16_t* out) { (void)in; (void)len; (void)out; return 0; }
inline size_t convert_utf16le_to_utf8(const char16_t* in, size_t len, char* out) { (void)in; (void)len; (void)out; return 0; }
inline size_t utf16_length_from_utf8(const char* in, size_t len) { (void)in; return len; }
inline size_t utf8_length_from_utf16le(const char16_t* in, size_t len) { (void)in; return len * 3; }
}
#endif
