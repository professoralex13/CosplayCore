#include "i2s.h"
#include "driver/i2s_std.h"
#include "esp_log.h"

static const char *TAG = "WM8988";

static i2s_chan_handle_t g_tx_handle = NULL;

#define DEFAULT_SAMPLE_RATE 44100

esp_err_t i2s_device_init(i2s_chan_handle_t *tx_handle,
                          i2s_chan_handle_t *rx_handle, uint8_t bclk_pin,
                          uint8_t ws_pin, uint8_t dout_pin, uint8_t din_pin) {
    i2s_chan_config_t chan_cfg =
        I2S_CHANNEL_DEFAULT_CONFIG(I2S_NUM_0, I2S_ROLE_MASTER);

    esp_err_t result = i2s_new_channel(&chan_cfg, tx_handle, rx_handle);

    if (result != ESP_OK)
        return result;

    // Store tx handle for dynamic reconfiguration
    g_tx_handle = *tx_handle;

    i2s_std_config_t std_cfg = {
        .clk_cfg = I2S_STD_CLK_DEFAULT_CONFIG(DEFAULT_SAMPLE_RATE),
        .slot_cfg = I2S_STD_PHILIPS_SLOT_DEFAULT_CONFIG(
            I2S_DATA_BIT_WIDTH_16BIT, I2S_SLOT_MODE_STEREO),
        .gpio_cfg =
            {
                .mclk = 0,
                .bclk = bclk_pin,
                .ws = ws_pin,
                .dout = dout_pin,
                .din = din_pin,
                .invert_flags =
                    {
                        .mclk_inv = false,
                        .bclk_inv = false,
                        .ws_inv = false,
                    },
            },
    };

    result = i2s_channel_init_std_mode(*tx_handle, &std_cfg);
    if (result != ESP_OK)
        return result;

    return i2s_channel_enable(*tx_handle);
}

esp_err_t i2s_set_sample_rate(uint32_t sample_rate) {
    if (g_tx_handle == NULL) {
        ESP_LOGE(TAG, "I2S not initialized");
        return ESP_ERR_INVALID_STATE;
    }

    // Disable the channel first
    esp_err_t result = i2s_channel_disable(g_tx_handle);
    if (result != ESP_OK) {
        ESP_LOGE(TAG, "Failed to disable I2S channel: %s",
                 esp_err_to_name(result));
        return result;
    }

    // Configure new clock settings
    i2s_std_clk_config_t clk_cfg = I2S_STD_CLK_DEFAULT_CONFIG(sample_rate);

    result = i2s_channel_reconfig_std_clock(g_tx_handle, &clk_cfg);
    if (result != ESP_OK) {
        ESP_LOGE(TAG, "Failed to reconfigure I2S clock: %s",
                 esp_err_to_name(result));
        return result;
    }

    // Re-enable the channel
    result = i2s_channel_enable(g_tx_handle);
    if (result != ESP_OK) {
        ESP_LOGE(TAG, "Failed to enable I2S channel: %s",
                 esp_err_to_name(result));
        return result;
    }

    ESP_LOGI(TAG, "I2S sample rate changed to %lu Hz", sample_rate);
    return ESP_OK;
}
