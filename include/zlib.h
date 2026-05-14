/*
 * Sphragis — zlib.h stub for NetSurf
 * Sphragis has its own DEFLATE implementation; this satisfies #include only.
 */
#ifndef _SPHRAGIS_ZLIB_H
#define _SPHRAGIS_ZLIB_H

#include <stddef.h>
#include <stdint.h>

#define ZLIB_VERSION "1.2.13-sphragis-stub"
#define ZLIB_VERNUM  0x12d0

/* zlib basic types */
typedef unsigned char  Byte;
typedef Byte           Bytef;
typedef unsigned int   uInt;
typedef unsigned long  uLong;
typedef void          *voidp;
typedef void          *voidpf;
typedef uLong          uLongf;
typedef Byte          *charf;

/* Flush values */
#define Z_NO_FLUSH      0
#define Z_PARTIAL_FLUSH 1
#define Z_SYNC_FLUSH    2
#define Z_FULL_FLUSH    3
#define Z_FINISH        4
#define Z_BLOCK         5
#define Z_TREES         6

/* Return codes */
#define Z_OK             0
#define Z_STREAM_END     1
#define Z_NEED_DICT      2
#define Z_ERRNO         (-1)
#define Z_STREAM_ERROR  (-2)
#define Z_DATA_ERROR    (-3)
#define Z_MEM_ERROR     (-4)
#define Z_BUF_ERROR     (-5)
#define Z_VERSION_ERROR (-6)

/* Compression levels */
#define Z_NO_COMPRESSION       0
#define Z_BEST_SPEED           1
#define Z_BEST_COMPRESSION     9
#define Z_DEFAULT_COMPRESSION (-1)

/* Strategy */
#define Z_FILTERED         1
#define Z_HUFFMAN_ONLY     2
#define Z_RLE              3
#define Z_FIXED            4
#define Z_DEFAULT_STRATEGY 0

/* Data type */
#define Z_BINARY  0
#define Z_TEXT    1
#define Z_ASCII  Z_TEXT
#define Z_UNKNOWN 2

/* Window bits */
#define MAX_WBITS 15

#define Z_NULL 0

typedef void *(*alloc_func)(void *opaque, unsigned items, unsigned size);
typedef void  (*free_func)(void *opaque, void *address);

typedef struct z_stream_s {
    const unsigned char *next_in;
    unsigned int         avail_in;
    unsigned long        total_in;

    unsigned char       *next_out;
    unsigned int         avail_out;
    unsigned long        total_out;

    const char          *msg;
    void                *state;

    alloc_func           zalloc;
    free_func            zfree;
    void                *opaque;

    int                  data_type;
    unsigned long        adler;
    unsigned long        reserved;
} z_stream;

typedef z_stream *z_streamp;

typedef struct gz_header_s {
    int     text;
    unsigned long time;
    int     xflags;
    int     os;
    unsigned char *extra;
    unsigned int   extra_len;
    unsigned int   extra_max;
    unsigned char *name;
    unsigned int   name_max;
    unsigned char *comment;
    unsigned int   comm_max;
    int     hcrc;
    int     done;
} gz_header;

typedef gz_header *gz_headerp;
typedef void *gzFile;

/* Basic functions */
const char *zlibVersion(void);

int deflateInit_(z_streamp strm, int level,
                 const char *version, int stream_size);
int deflate(z_streamp strm, int flush);
int deflateEnd(z_streamp strm);

int inflateInit_(z_streamp strm,
                 const char *version, int stream_size);
int inflate(z_streamp strm, int flush);
int inflateEnd(z_streamp strm);

/* Advanced functions */
int deflateInit2_(z_streamp strm, int level, int method,
                  int windowBits, int memLevel, int strategy,
                  const char *version, int stream_size);
int inflateInit2_(z_streamp strm, int windowBits,
                  const char *version, int stream_size);

int deflateReset(z_streamp strm);
int inflateReset(z_streamp strm);
int inflateReset2(z_streamp strm, int windowBits);
unsigned long crc32(unsigned long crc, const unsigned char *buf, unsigned int len);
unsigned long adler32(unsigned long adler, const unsigned char *buf, unsigned int len);

/* Convenience macros */
#define deflateInit(strm, level) \
    deflateInit_((strm), (level), ZLIB_VERSION, (int)sizeof(z_stream))
#define inflateInit(strm) \
    inflateInit_((strm), ZLIB_VERSION, (int)sizeof(z_stream))
#define deflateInit2(strm, level, method, windowBits, memLevel, strategy) \
    deflateInit2_((strm), (level), (method), (windowBits), (memLevel), \
                  (strategy), ZLIB_VERSION, (int)sizeof(z_stream))
#define inflateInit2(strm, windowBits) \
    inflateInit2_((strm), (windowBits), ZLIB_VERSION, (int)sizeof(z_stream))

/* Utility */
unsigned long crc32(unsigned long crc, const unsigned char *buf, unsigned int len);
unsigned long adler32(unsigned long adler, const unsigned char *buf, unsigned int len);
int           compress(unsigned char *dest, unsigned long *destLen,
                       const unsigned char *source, unsigned long sourceLen);
int           uncompress(unsigned char *dest, unsigned long *destLen,
                         const unsigned char *source, unsigned long sourceLen);

#endif /* _SPHRAGIS_ZLIB_H */
