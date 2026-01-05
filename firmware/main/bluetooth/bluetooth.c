#include "bluetooth.h"

#include "bt_audio.h"
#include "bt_core.h"
#include "bt_pairing.h"

#include "esp_gap_bt_api.h"
#include "esp_log.h"

#define TAG "BT"

#define BT_DEVICE_NAME "CosplayCore"
#define PAIRING_BUTTON_GPIO 21

void bluetooth_init(i2s_chan_handle_t i2s_handle) {
    ESP_LOGI(TAG, "Starting Bluetooth speaker");

    // Initialize Bluetooth stack
    ESP_ERROR_CHECK(bt_core_init());

    // Set device name
    esp_bt_gap_set_device_name(BT_DEVICE_NAME);

    // Initialize A2DP audio
    ESP_ERROR_CHECK(bt_audio_init(i2s_handle));

    // Initialize pairing control
    bt_pairing_init(PAIRING_BUTTON_GPIO);

    ESP_LOGI(TAG, "Bluetooth ready. Device name: %s", BT_DEVICE_NAME);
}
