#include "i2s.h"
#include "driver/i2s_std.h"
#include "esp_log.h"
#include "filters.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

static const char *TAG = "WM8988";

static i2s_chan_handle_t g_tx_handle = NULL;
static i2s_chan_handle_t g_rx_handle = NULL;
static TaskHandle_t g_loopback_task = NULL;
static bool g_loopback_running = false;
static audio_filters_t g_audio_filters;

#define DEFAULT_SAMPLE_RATE 44100
#define LOOPBACK_BUFFER_SIZE 512 // Buffer size in samples (stereo 16-bit)

esp_err_t i2s_device_init(i2s_chan_handle_t *tx_handle,
                          i2s_chan_handle_t *rx_handle, uint8_t bclk_pin,
                          uint8_t ws_pin, uint8_t dout_pin, uint8_t din_pin) {
    i2s_chan_config_t chan_cfg =
        I2S_CHANNEL_DEFAULT_CONFIG(I2S_NUM_0, I2S_ROLE_MASTER);

    esp_err_t result = i2s_new_channel(&chan_cfg, tx_handle, rx_handle);

    if (result != ESP_OK)
        return result;

    // Store handles for dynamic reconfiguration and loopback
    g_tx_handle = *tx_handle;
    g_rx_handle = *rx_handle;

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

    // Initialize RX channel with same configuration
    result = i2s_channel_init_std_mode(*rx_handle, &std_cfg);
    if (result != ESP_OK)
        return result;

    // Enable both channels
    result = i2s_channel_enable(*tx_handle);
    if (result != ESP_OK)
        return result;

    result = i2s_channel_enable(*rx_handle);
    if (result != ESP_OK)
        return result;

    // Initialize audio filters for static removal
    audio_filters_init(&g_audio_filters, DEFAULT_SAMPLE_RATE);

    return ESP_OK;
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

    // Reinitialize filters with new sample rate
    audio_filters_init(&g_audio_filters, sample_rate);

    ESP_LOGI(TAG, "I2S sample rate changed to %lu Hz", sample_rate);
    return ESP_OK;
}

static void i2s_loopback_task(void *arg) {
    int16_t *buffer =
        malloc(LOOPBACK_BUFFER_SIZE * 2 * sizeof(int16_t)); // Stereo buffer
    if (buffer == NULL) {
        ESP_LOGE(TAG, "Failed to allocate loopback buffer");
        vTaskDelete(NULL);
        return;
    }

    size_t bytes_read, bytes_written;
    const size_t buffer_bytes = LOOPBACK_BUFFER_SIZE * 2 * sizeof(int16_t);

    ESP_LOGI(TAG, "I2S loopback task started");

    while (g_loopback_running) {
        // Read from RX channel
        esp_err_t read_result = i2s_channel_read(
            g_rx_handle, buffer, buffer_bytes, &bytes_read, portMAX_DELAY);
        if (read_result == ESP_OK && bytes_read > 0) {
            // Apply filters for static removal (high-pass and low-pass)
            size_t samples = bytes_read / sizeof(int16_t);

            audio_filters_process(&g_audio_filters, buffer, samples);

            // Write to TX channel after filtering
            esp_err_t write_result = i2s_channel_write(
                g_tx_handle, buffer, bytes_read, &bytes_written, portMAX_DELAY);
            if (write_result != ESP_OK || bytes_written != bytes_read) {
                ESP_LOGW(TAG, "I2S loopback write mismatch: read %d, wrote %d",
                         bytes_read, bytes_written);
            }
        } else {
            ESP_LOGW(TAG, "I2S loopback read failed: %s",
                     esp_err_to_name(read_result));
            vTaskDelay(pdMS_TO_TICKS(10)); // Brief delay on error
        }
    }

    free(buffer);
    ESP_LOGI(TAG, "I2S loopback task stopped");
    g_loopback_task = NULL;
    vTaskDelete(NULL);
}

esp_err_t i2s_start_loopback(void) {
    if (g_tx_handle == NULL || g_rx_handle == NULL) {
        ESP_LOGE(TAG, "I2S not initialized");
        return ESP_ERR_INVALID_STATE;
    }

    if (g_loopback_running) {
        ESP_LOGW(TAG, "I2S loopback already running");
        return ESP_OK;
    }

    g_loopback_running = true;
    BaseType_t result = xTaskCreate(i2s_loopback_task, "i2s_loopback", 4096,
                                    NULL, 5, &g_loopback_task);

    if (result != pdTRUE) {
        g_loopback_running = false;
        ESP_LOGE(TAG, "Failed to create I2S loopback task");
        return ESP_ERR_NO_MEM;
    }

    ESP_LOGI(TAG, "I2S loopback started");
    return ESP_OK;
}

esp_err_t i2s_stop_loopback(void) {
    if (!g_loopback_running) {
        ESP_LOGW(TAG, "I2S loopback not running");
        return ESP_OK;
    }

    g_loopback_running = false;

    // Wait for task to finish
    while (g_loopback_task != NULL) {
        vTaskDelay(pdMS_TO_TICKS(10));
    }

    ESP_LOGI(TAG, "I2S loopback stopped");
    return ESP_OK;
}

esp_err_t i2s_reset_filters(void) {
    audio_filters_reset(&g_audio_filters);
    ESP_LOGI(TAG, "Audio filters reset");
    return ESP_OK;
}
