#include "bt_audio.h"
#include "codec/i2s.h"
#include "codec/settings.h"
#include "driver/i2s_common.h"
#include "esp_a2dp_api.h"
#include "esp_log.h"
#include "portmacro.h"

#define TAG "BT_AUDIO"

static i2s_chan_handle_t i2s_channel;
static spi_codec_device codec_device;

// A2DP sample frequency enum values from ESP-IDF
typedef enum {
    A2D_SAMP_FREQ_96K = 0,
    A2D_SAMP_FREQ_48K = 1,   // 48Khz and 44.1Khz are the only tested values of
    A2D_SAMP_FREQ_44_1K = 2, // this, but the rest are extrapolated
    A2D_SAMP_FREQ_32K = 3,
    A2D_SAMP_FREQ_16K = 4,
} a2d_samp_freq_t;

// Convert A2DP sample frequency enum to Hz
static uint32_t a2dp_freq_to_hz(uint8_t freq) {
    switch (freq) {
        case A2D_SAMP_FREQ_96K:
            return 96000;
        case A2D_SAMP_FREQ_48K:
            return 48000;
        case A2D_SAMP_FREQ_44_1K:
            return 44100;
        case A2D_SAMP_FREQ_32K:
            return 32000;
        case A2D_SAMP_FREQ_16K:
            return 16000;
        default:
            ESP_LOGW(TAG, "Unknown A2DP sample frequency: %d", freq);
            return 48000; // Default to 48kHz
    }
}

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
                // Unmute DAC when A2DP connects
                set_dac_mute(codec_device, false);
            } else if (param->conn_stat.state ==
                       ESP_A2D_CONNECTION_STATE_DISCONNECTED) {
                ESP_LOGI(TAG, "A2DP disconnected");
                // Mute DAC when A2DP disconnects
                set_dac_mute(codec_device, true);
            }
            break;

        case ESP_A2D_AUDIO_STATE_EVT:
            if (param->audio_stat.state == ESP_A2D_AUDIO_STATE_STARTED) {
                ESP_LOGI(TAG, "Audio playback started");
            } else if (param->audio_stat.state == ESP_A2D_AUDIO_STATE_STOPPED) {
                ESP_LOGI(TAG, "Audio playback stopped");
            }
            break;

        case ESP_A2D_AUDIO_CFG_EVT: {
            uint8_t samp_freq = param->audio_cfg.mcc.cie.sbc_info.samp_freq;
            uint32_t sample_rate_hz = a2dp_freq_to_hz(samp_freq);
            ESP_LOGI(TAG, "Audio config: sample_rate=%d (%lu Hz)", samp_freq,
                     sample_rate_hz);
            // Update I2S sample rate to match A2DP
            i2s_set_sample_rate(sample_rate_hz);
            break;
        }

        default:
            ESP_LOGD(TAG, "A2DP event: %d", event);
            break;
    }
}

esp_err_t bt_audio_init(i2s_chan_handle_t i2s_handle,
                        spi_codec_device codec_dev) {
    esp_err_t ret;

    i2s_channel = i2s_handle;
    codec_device = codec_dev;

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
