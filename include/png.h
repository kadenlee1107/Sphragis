/*
 * Bat_OS — png.h stub for NetSurf
 * Bat_OS has its own PNG decoder; this satisfies #include only.
 */
#ifndef _BATOS_PNG_H
#define _BATOS_PNG_H

#include <stddef.h>
#include <stdint.h>

#define PNG_LIBPNG_VER_STRING "1.6.0-batos-stub"
#define PNG_LIBPNG_VER        10600

/* Color type masks */
#define PNG_COLOR_MASK_PALETTE  1
#define PNG_COLOR_MASK_COLOR    2
#define PNG_COLOR_MASK_ALPHA    4

/* Color types */
#define PNG_COLOR_TYPE_GRAY       0
#define PNG_COLOR_TYPE_PALETTE    (PNG_COLOR_MASK_COLOR | PNG_COLOR_MASK_PALETTE)
#define PNG_COLOR_TYPE_RGB        PNG_COLOR_MASK_COLOR
#define PNG_COLOR_TYPE_RGB_ALPHA  (PNG_COLOR_MASK_COLOR | PNG_COLOR_MASK_ALPHA)
#define PNG_COLOR_TYPE_GRAY_ALPHA PNG_COLOR_MASK_ALPHA
#define PNG_COLOR_TYPE_RGBA       PNG_COLOR_TYPE_RGB_ALPHA
#define PNG_COLOR_TYPE_GA         PNG_COLOR_TYPE_GRAY_ALPHA

/* Interlace types */
#define PNG_INTERLACE_NONE  0
#define PNG_INTERLACE_ADAM7 1

/* Transform flags */
#define PNG_TRANSFORM_IDENTITY       0x0000
#define PNG_TRANSFORM_STRIP_16       0x0001
#define PNG_TRANSFORM_STRIP_ALPHA    0x0002
#define PNG_TRANSFORM_PACKING        0x0004
#define PNG_TRANSFORM_PACKSWAP       0x0008
#define PNG_TRANSFORM_EXPAND         0x0010
#define PNG_TRANSFORM_INVERT_MONO    0x0020
#define PNG_TRANSFORM_SHIFT          0x0040
#define PNG_TRANSFORM_BGR            0x0080
#define PNG_TRANSFORM_SWAP_ALPHA     0x0100
#define PNG_TRANSFORM_SWAP_ENDIAN    0x0200
#define PNG_TRANSFORM_INVERT_ALPHA   0x0400
#define PNG_TRANSFORM_STRIP_FILLER   0x0800

/* Info flags */
#define PNG_INFO_tRNS 0x0010

/* Filler position */
#define PNG_FILLER_AFTER 1

/* Opaque types */
typedef struct png_struct_def  png_struct;
typedef struct png_info_def    png_info;
typedef        png_struct     *png_structp;
typedef const  png_struct     *png_const_structp;
typedef        png_info       *png_infop;
typedef const  png_info       *png_const_infop;
typedef        png_struct    **png_structpp;
typedef        png_info      **png_infopp;
typedef        uint8_t        png_byte;
typedef        uint8_t       *png_bytep;
typedef const  uint8_t       *png_const_bytep;
typedef        uint32_t       png_uint_32;
typedef        int32_t        png_int_32;
typedef        size_t         png_size_t;
typedef        uint8_t      **png_bytepp;
typedef        char           png_char;
typedef const  char          *png_const_charp;
typedef        void          (*png_error_ptr)(png_structp, png_const_charp);
typedef        void          (*png_rw_ptr)(png_structp, png_bytep, png_size_t);

/* Core API stubs */
png_structp png_create_read_struct(const char *user_png_ver,
                                   void *error_ptr,
                                   png_error_ptr error_fn,
                                   png_error_ptr warn_fn);
png_infop   png_create_info_struct(png_structp png_ptr);
void        png_destroy_read_struct(png_structpp png_ptr_ptr,
                                    png_infopp info_ptr_ptr,
                                    png_infopp end_info_ptr_ptr);
void        png_set_read_fn(png_structp png_ptr, void *io_ptr,
                            png_rw_ptr read_data_fn);
void        png_read_info(png_structp png_ptr, png_infop info_ptr);
void        png_read_image(png_structp png_ptr, png_bytepp image);
void        png_read_end(png_structp png_ptr, png_infop info_ptr);
void        png_read_update_info(png_structp png_ptr, png_infop info_ptr);

png_uint_32 png_get_IHDR(png_structp png_ptr, png_infop info_ptr,
                          png_uint_32 *width, png_uint_32 *height,
                          int *bit_depth, int *color_type,
                          int *interlace_type, int *compression_type,
                          int *filter_type);
png_uint_32 png_get_valid(png_structp png_ptr, png_infop info_ptr,
                           png_uint_32 flag);
png_uint_32 png_get_rowbytes(png_structp png_ptr, png_infop info_ptr);
png_uint_32 png_get_image_width(png_structp png_ptr, png_infop info_ptr);
png_uint_32 png_get_image_height(png_structp png_ptr, png_infop info_ptr);
png_byte    png_get_color_type(png_structp png_ptr, png_infop info_ptr);
png_byte    png_get_bit_depth(png_structp png_ptr, png_infop info_ptr);

void        png_set_expand(png_structp png_ptr);
void        png_set_strip_16(png_structp png_ptr);
void        png_set_gray_to_rgb(png_structp png_ptr);
void        png_set_add_alpha(png_structp png_ptr, png_uint_32 filler, int flags);
void        png_set_filler(png_structp png_ptr, png_uint_32 filler, int flags);
void        png_set_tRNS_to_alpha(png_structp png_ptr);
void        png_set_palette_to_rgb(png_structp png_ptr);
void        png_set_expand_gray_1_2_4_to_8(png_structp png_ptr);
void        png_set_interlace_handling(png_structp png_ptr);

void       *png_get_io_ptr(png_structp png_ptr);
void        png_set_progressive_read_fn(png_structp png_ptr, void *progressive_ptr,
                                         void (*info_fn)(png_structp, png_infop),
                                         void (*row_fn)(png_structp, png_bytep,
                                                        png_uint_32, int),
                                         void (*end_fn)(png_structp, png_infop));
void        png_process_data(png_structp png_ptr, png_infop info_ptr,
                              png_bytep buffer, png_size_t buffer_size);

#endif /* _BATOS_PNG_H */
