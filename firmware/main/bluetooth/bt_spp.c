#include "esp_err.h"
#include "esp_log.h"
#include "esp_spp_api.h"
#include <stdint.h>

#include "bt_spp.h"

#define TAG "BT_SPP"

#define SERVE_NAME "CosplayCore"

uint32_t spp_handle;

static void esp_spp_cb(esp_spp_cb_event_t event, esp_spp_cb_param_t *param) {
    switch (event) {
        case ESP_SPP_INIT_EVT:
            if (param->init.status == ESP_SPP_SUCCESS) {
                ESP_LOGI(TAG, "ESP_SPP_INIT_EVT");
                esp_spp_start_srv(ESP_SPP_SEC_AUTHENTICATE, ESP_SPP_ROLE_SLAVE,
                                  0, SERVE_NAME);
            } else {
                ESP_LOGE(TAG, "ESP_SPP_INIT_EVT status:%d", param->init.status);
            }
            break;
        case ESP_SPP_DISCOVERY_COMP_EVT:
            ESP_LOGI(TAG, "ESP_SPP_DISCOVERY_COMP_EVT");
            break;
        case ESP_SPP_OPEN_EVT:
            ESP_LOGI(TAG, "ESP_SPP_OPEN_EVT");
            break;
        case ESP_SPP_CLOSE_EVT:
            ESP_LOGI(TAG,
                     "ESP_SPP_CLOSE_EVT status:%d handle:%" PRIu32
                     " close_by_remote:%d",
                     param->close.status, param->close.handle,
                     param->close.async);
            break;
        case ESP_SPP_START_EVT:
            if (param->start.status == ESP_SPP_SUCCESS) {
                ESP_LOGI(
                    TAG,
                    "ESP_SPP_START_EVT handle:%" PRIu32 " sec_id:%d scn:%d",
                    param->start.handle, param->start.sec_id, param->start.scn);
                spp_handle = param->start.handle;
            } else {
                ESP_LOGE(TAG, "ESP_SPP_START_EVT status:%d",
                         param->start.status);
            }
            break;
        case ESP_SPP_CL_INIT_EVT:
            ESP_LOGI(TAG, "ESP_SPP_CL_INIT_EVT");
            break;
        case ESP_SPP_DATA_IND_EVT:
            ESP_LOGI(TAG, "ESP_SPP_DATA_IND_EVT: %s", param->data_ind.data);
            break;
        case ESP_SPP_CONG_EVT:
            ESP_LOGI(TAG, "ESP_SPP_CONG_EVT");
            break;
        case ESP_SPP_WRITE_EVT:
            ESP_LOGI(TAG, "ESP_SPP_WRITE_EVT");
            break;
        case ESP_SPP_SRV_OPEN_EVT:
            ESP_LOGI(TAG, "ESP_SPP_SRV_OPEN_EVT status:%d",
                     param->srv_open.status);
            break;
        case ESP_SPP_SRV_STOP_EVT:
            ESP_LOGI(TAG, "ESP_SPP_SRV_STOP_EVT");
            break;
        case ESP_SPP_UNINIT_EVT:
            ESP_LOGI(TAG, "ESP_SPP_UNINIT_EVT");
            break;
        default:
            break;
    }
}

esp_err_t bt_spp_init() {
    esp_err_t ret;

    esp_spp_cfg_t bt_spp_cfg = {
        .mode = ESP_SPP_MODE_CB,
        .enable_l2cap_ertm = true,
        .tx_buffer_size = 0,
    };

    if ((ret = esp_spp_register_callback(esp_spp_cb)) != ESP_OK) {
        ESP_LOGE(TAG, "%s spp register failed: %s", __func__,
                 esp_err_to_name(ret));
        return ret;
    }

    if ((ret = esp_spp_enhanced_init(&bt_spp_cfg)) != ESP_OK) {
        ESP_LOGE(TAG, "%s spp init failed: %s", __func__, esp_err_to_name(ret));
        return ret;
    }

    return ESP_OK;
}