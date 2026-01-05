#ifndef BT_AUDIO_H
#define BT_AUDIO_H

#include "driver/i2s_types.h"
#include "esp_err.h"

esp_err_t bt_audio_init(i2s_chan_handle_t i2s_handle);

#endif
