#ifndef _JXL_DECODE_H
#define _JXL_DECODE_H
typedef struct JxlDecoderStruct JxlDecoder;
typedef enum { JXL_DEC_SUCCESS=0, JXL_DEC_ERROR=1 } JxlDecoderStatus;
typedef struct { int num_channels; int data_type; int endianness; int align; } JxlPixelFormat;
typedef struct { int xsize; int ysize; } JxlBasicInfo;
JxlDecoder *JxlDecoderCreate(void *alloc);
void JxlDecoderDestroy(JxlDecoder *dec);
#endif
