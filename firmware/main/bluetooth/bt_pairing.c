#include "bt_pairing.h"
#include "driver/gpio.h"
#include "esp_gap_bt_api.h"
#include "esp_log.h"
#include "freertos/FreeRTOS.h"

#define BUTTON_DEBOUNCE_MS 50
#define BUTTON_LONG_PRESS_MS 3000
#define BUTTON_POLL_INTERVAL_MS 10

#define TAG "BT_PAIRING"

static bool discoverable = false;

static uint32_t button_press_time = 0;
static uint8_t button_gpio = 0;

static void gap_callback(esp_bt_gap_cb_event_t event,
                         esp_bt_gap_cb_param_t *param) {
    switch (event) {
        case ESP_BT_GAP_AUTH_CMPL_EVT:
            if (param->auth_cmpl.stat == ESP_BT_STATUS_SUCCESS) {
                ESP_LOGI(TAG, "Authentication success: %s",
                         param->auth_cmpl.device_name);
            } else {
                ESP_LOGE(TAG, "Authentication failed, status: %d",
                         param->auth_cmpl.stat);
            }
            break;

        case ESP_BT_GAP_MODE_CHG_EVT:
            ESP_LOGI(TAG, "GAP mode change: %d", param->mode_chg.mode);
            break;

        default:
            ESP_LOGD(TAG, "GAP event: %d", event);
            break;
    }
}

void bt_pairing_enter(void) {
    if (!discoverable) {
        ESP_LOGI(TAG, "Entering pairing mode");
        esp_bt_gap_set_scan_mode(ESP_BT_CONNECTABLE,
                                 ESP_BT_GENERAL_DISCOVERABLE);
        discoverable = true;
    }
}

void bt_pairing_exit(void) {
    if (discoverable) {
        ESP_LOGI(TAG, "Exiting pairing mode");
        esp_bt_gap_set_scan_mode(ESP_BT_CONNECTABLE, ESP_BT_NON_DISCOVERABLE);
        discoverable = false;
    }
}

bool bt_pairing_is_active(void) { return discoverable; }

static void button_task(void *pvParameters) {
    bool last_state = true;
    bool current_state;
    uint32_t press_duration;

    while (true) {
        current_state = gpio_get_level(button_gpio);

        // Button press (active low)
        if (!current_state && last_state) {
            vTaskDelay(pdMS_TO_TICKS(BUTTON_DEBOUNCE_MS));
            current_state = gpio_get_level(button_gpio);

            if (!current_state) {
                button_press_time = xTaskGetTickCount() * portTICK_PERIOD_MS;
            }
        }
        // Button release
        else if (current_state && !last_state) {
            vTaskDelay(pdMS_TO_TICKS(BUTTON_DEBOUNCE_MS));
            current_state = gpio_get_level(button_gpio);

            if (current_state) {
                press_duration = (xTaskGetTickCount() * portTICK_PERIOD_MS) -
                                 button_press_time;

                // Short press: toggle pairing mode
                if (press_duration < BUTTON_LONG_PRESS_MS) {
                    if (discoverable) {
                        bt_pairing_exit();
                    } else {
                        bt_pairing_enter();
                    }
                }
            }
        }

        last_state = current_state;
        vTaskDelay(pdMS_TO_TICKS(BUTTON_POLL_INTERVAL_MS));
    }
}

void bt_pairing_init(uint8_t gpio_num) {
    button_gpio = gpio_num;

    // Configure button GPIO
    gpio_config_t io_conf = {.pin_bit_mask = (1ULL << button_gpio),
                             .mode = GPIO_MODE_INPUT,
                             .pull_up_en = GPIO_PULLUP_ENABLE,
                             .pull_down_en = GPIO_PULLDOWN_DISABLE,
                             .intr_type = GPIO_INTR_DISABLE};
    gpio_config(&io_conf);

    // Register GAP callback
    esp_bt_gap_register_callback(gap_callback);

    // Set authentication mode (SSP with no I/O capabilities)
    esp_bt_sp_param_t param_type = ESP_BT_SP_IOCAP_MODE;
    esp_bt_io_cap_t iocap = ESP_BT_IO_CAP_NONE;
    esp_bt_gap_set_security_param(param_type, &iocap, sizeof(uint8_t));

    // Set non-discoverable by default
    esp_bt_gap_set_scan_mode(ESP_BT_CONNECTABLE, ESP_BT_NON_DISCOVERABLE);

    // Create button monitoring task
    xTaskCreate(button_task, "bt_button", 2048, NULL, 5, NULL);

    ESP_LOGI(TAG, "Pairing control initialized (GPIO %d)", button_gpio);
}
