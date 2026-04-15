#ifndef ABSL_STRINGS_STR_CAT_H_
#define ABSL_STRINGS_STR_CAT_H_
#include <string>
namespace absl {
inline std::string StrCat() { return ""; }
template<class T>
std::string StrCat(const T& a) { return std::to_string(a); }
inline std::string StrCat(const char* a) { return a ? a : ""; }
inline std::string StrCat(const std::string& a) { return a; }
template<class A, class B>
std::string StrCat(const A& a, const B& b) { return StrCat(a) + StrCat(b); }
template<class A, class B, class C>
std::string StrCat(const A& a, const B& b, const C& c) { return StrCat(a) + StrCat(b) + StrCat(c); }
template<class A, class B, class C, class D>
std::string StrCat(const A& a, const B& b, const C& c, const D& d) { return StrCat(a,b) + StrCat(c,d); }
inline void StrAppend(std::string* dest, const std::string& s) { dest->append(s.c_str()); }
}
#endif
