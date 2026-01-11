#include "bluetooth.h"

#include "bluetooth/bt_spp.h"
#include "bt_audio.h"
#include "bt_core.h"
#include "bt_pairing.h"

#include "esp_gap_bt_api.h"
#include "esp_log.h"

#include "messages.h"

#define TAG "BT"

#define BT_DEVICE_NAME "CosplayCore"
#define PAIRING_BUTTON_GPIO 21

void bluetooth_init(i2s_chan_handle_t i2s_handle, spi_codec_device codec_dev) {
    ESP_LOGI(TAG, "Starting Bluetooth speaker");

    // Initialize Bluetooth stack
    ESP_ERROR_CHECK(bt_core_init());

    // Set device name
    esp_bt_gap_set_device_name(BT_DEVICE_NAME);

    // Initialize A2DP audio
    ESP_ERROR_CHECK(bt_audio_init(i2s_handle, codec_dev));

    // Initialize pairing control
    bt_pairing_init(PAIRING_BUTTON_GPIO);

    bt_spp_init();

    ESP_LOGI(TAG, "Bluetooth ready. Device name: %s", BT_DEVICE_NAME);

    init_bt_control(codec_dev);
}
