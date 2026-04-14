#ifndef _ARPA_INET_H
#define _ARPA_INET_H
#include <stdint.h>
uint32_t htonl(uint32_t hostlong);
uint16_t htons(uint16_t hostshort);
uint32_t ntohl(uint32_t netlong);
uint16_t ntohs(uint16_t netshort);
int inet_pton(int af, const char *src, void *dst);
const char *inet_ntop(int af, const void *src, char *dst, unsigned int size);
#define INET_ADDRSTRLEN 16
#define INET6_ADDRSTRLEN 46
#endif
