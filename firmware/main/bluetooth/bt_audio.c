#include "bt_audio.h"
#include "driver/i2s_common.h"
#include "esp_a2dp_api.h"
#include "esp_log.h"
#include "portmacro.h"

#define TAG "BT_AUDIO"

static i2s_chan_handle_t i2s_channel;

static void audio_data_callback(const uint8_t *data, uint32_t len) {
    size_t bytes_written = 0;

    i2s_channel_write(i2s_channel, data, len, &bytes_written, portMAX_DELAY);

    if (bytes_written < len) {
        ESP_LOGW(TAG, "I2S underrun: %d/%d bytes written", bytes_written, len);
    }
}

static void a2dp_callback(esp_a2d_cb_event_t event, esp_a2d_cb_param_t *param) {
    switch (event) {
        case ESP_A2D_CONNECTION_STATE_EVT:
            if (param->conn_stat.state == ESP_A2D_CONNECTION_STATE_CONNECTED) {
                ESP_LOGI(TAG, "A2DP connected");
            } else if (param->conn_stat.state ==
                       ESP_A2D_CONNECTION_STATE_DISCONNECTED) {
                ESP_LOGI(TAG, "A2DP disconnected");
            }
            break;

        case ESP_A2D_AUDIO_STATE_EVT:
            if (param->audio_stat.state == ESP_A2D_AUDIO_STATE_STARTED) {
                ESP_LOGI(TAG, "Audio playback started");
            } else if (param->audio_stat.state == ESP_A2D_AUDIO_STATE_STOPPED) {
                ESP_LOGI(TAG, "Audio playback stopped");
            }
            break;

        case ESP_A2D_AUDIO_CFG_EVT:
            ESP_LOGI(TAG, "Audio config: sample_rate=%d",
                     param->audio_cfg.mcc.cie.sbc_info.samp_freq);
            break;

        default:
            ESP_LOGD(TAG, "A2DP event: %d", event);
            break;
    }
}

esp_err_t bt_audio_init(i2s_chan_handle_t i2s_handle) {
    esp_err_t ret;

    i2s_channel = i2s_handle;

    // Register A2DP callback
    ret = esp_a2d_register_callback(a2dp_callback);
    if (ret != ESP_OK) {
        ESP_LOGE(TAG, "A2DP callback register failed: %s",
                 esp_err_to_name(ret));
        return ret;
    }

    // Register A2DP data callback
    ret = esp_a2d_sink_register_data_callback(audio_data_callback);
    if (ret != ESP_OK) {
        ESP_LOGE(TAG, "A2DP data callback register failed: %s",
                 esp_err_to_name(ret));
        return ret;
    }

    // Initialize A2DP sink
    ret = esp_a2d_sink_init();
    if (ret != ESP_OK) {
        ESP_LOGE(TAG, "A2DP sink init failed: %s", esp_err_to_name(ret));
        return ret;
    }

    ESP_LOGI(TAG, "A2DP audio initialized");
    return ESP_OK;
}
