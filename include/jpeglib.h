#ifndef _JPEGLIB_H
#define _JPEGLIB_H
#include <stdio.h>
typedef unsigned int JDIMENSION;
typedef int boolean;
typedef enum { JCS_UNKNOWN, JCS_GRAYSCALE, JCS_RGB, JCS_YCbCr, JCS_CMYK } J_COLOR_SPACE;
struct jpeg_decompress_struct { int image_width; int image_height; int num_components; J_COLOR_SPACE out_color_space; JDIMENSION output_width; JDIMENSION output_height; int output_components; int output_scanline; };
struct jpeg_error_mgr { void (*error_exit)(void*); char msg[200]; };
typedef struct jpeg_decompress_struct *j_decompress_ptr;
void jpeg_CreateDecompress(j_decompress_ptr cinfo, int version, size_t structsize);
struct jpeg_error_mgr *jpeg_std_error(struct jpeg_error_mgr *err);
void jpeg_stdio_src(j_decompress_ptr cinfo, FILE *infile);
int jpeg_read_header(j_decompress_ptr cinfo, boolean require_image);
boolean jpeg_start_decompress(j_decompress_ptr cinfo);
JDIMENSION jpeg_read_scanlines(j_decompress_ptr cinfo, unsigned char **scanlines, JDIMENSION max_lines);
boolean jpeg_finish_decompress(j_decompress_ptr cinfo);
void jpeg_destroy_decompress(j_decompress_ptr cinfo);
void jpeg_mem_src(j_decompress_ptr cinfo, const unsigned char *buf, unsigned long bufsize);
#define JPEG_LIB_VERSION 80
#define jpeg_create_decompress(cinfo) jpeg_CreateDecompress((cinfo), JPEG_LIB_VERSION, sizeof(struct jpeg_decompress_struct))
#define JMSG_LENGTH_MAX 200
#define JPEG_HEADER_OK 1
#endif
