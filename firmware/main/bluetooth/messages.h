#ifndef BT_MESSAGES_H
#define BT_MESSAGES_H

#include "codec/spi.h"
#include "stdint.h"

void on_message_received(uint8_t *data);
void init_bt_control(spi_codec_device _ext_int_codec);

#endif