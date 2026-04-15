#ifndef ABSL_STRINGS_STR_FORMAT_H_
#define ABSL_STRINGS_STR_FORMAT_H_
#include <cstdio>
#include <string>
namespace absl {
template<class... Args>
std::string StrFormat(const char* fmt, Args... args) {
    char buf[256];
    snprintf(buf, sizeof(buf), fmt, args...);
    return std::string(buf);
}
template<class... Args>
void PrintF(const char* fmt, Args... args) { printf(fmt, args...); }
template<class Sink, class... Args>
void Format(Sink* sink, const char* fmt, Args... args) {
    char buf[256];
    snprintf(buf, sizeof(buf), fmt, args...);
    *sink << buf;
}
}
#endif
