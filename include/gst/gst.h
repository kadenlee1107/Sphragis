/*
 * Sphragis — gst/gst.h stub for NetSurf
 * GStreamer is not used; this satisfies #include only.
 */
#ifndef _SPHRAGIS_GST_GST_H
#define _SPHRAGIS_GST_GST_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

/* Minimal type stubs so code referencing GStreamer compiles */
typedef void        GstElement;
typedef void        GstPipeline;
typedef void        GstBus;
typedef void        GstMessage;
typedef void        GstPad;
typedef void        GstCaps;
typedef void        GstBuffer;
typedef void        GstSample;
typedef void        GstBin;
typedef int64_t     GstClockTime;
typedef uint32_t    GstState;
typedef uint32_t    GstStateChangeReturn;
typedef uint32_t    GstFormat;
typedef uint32_t    GstMessageType;

#define GST_CLOCK_TIME_NONE ((GstClockTime)-1)
#define GST_SECOND          ((GstClockTime)1000000000)

#define GST_STATE_NULL     0
#define GST_STATE_READY    1
#define GST_STATE_PAUSED   2
#define GST_STATE_PLAYING  3

#define GST_STATE_CHANGE_FAILURE  0
#define GST_STATE_CHANGE_SUCCESS  1
#define GST_STATE_CHANGE_ASYNC    2

void gst_init(int *argc, char **argv[]);

GstElement *gst_element_factory_make(const char *factoryname,
                                      const char *name);
GstElement *gst_pipeline_new(const char *name);
GstElement *gst_parse_launch(const char *pipeline_description,
                              void **error);

GstStateChangeReturn gst_element_set_state(GstElement *element,
                                            GstState state);
GstStateChangeReturn gst_element_get_state(GstElement *element,
                                            GstState *state,
                                            GstState *pending,
                                            GstClockTime timeout);

GstBus     *gst_element_get_bus(GstElement *element);
GstMessage *gst_bus_timed_pop_filtered(GstBus *bus, GstClockTime timeout,
                                        GstMessageType types);

void        gst_object_unref(void *object);
void        gst_message_unref(GstMessage *msg);

typedef int gboolean;
#define TRUE 1
#define FALSE 0
typedef struct {} cairo_surface_t;
typedef struct {} cairo_t;
#endif /* _SPHRAGIS_GST_GST_H */
