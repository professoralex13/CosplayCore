#ifndef BLUETOOTH_H
#define BLUETOOTH_H

#include "codec/spi.h"
#include "driver/i2s_types.h"

void bluetooth_init(i2s_chan_handle_t i2s_handle, spi_codec_device codec_dev);

#endif
