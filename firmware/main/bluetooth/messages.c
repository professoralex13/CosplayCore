#include "codec/settings.h"
#include "esp_log.h"
#include "stdint.h"

typedef enum : uint8_t {
    SetInputVolume = 0,
    SetOutputVolume = 1,
    SetDACVolume = 2,
} MessageType;

typedef struct {
    uint8_t volume;
} VolumeMessage;

spi_codec_device ext_int_codec;

void on_message_received(uint8_t *data) {
    MessageType messageType = data[0];

    switch (messageType) {
        case SetInputVolume:
            VolumeMessage message1;
            memcpy(&message1, &data[1], sizeof(VolumeMessage));

            ESP_LOGI("MESSAGES", "Setting input volume to %d", message1.volume);

            set_input_volume(ext_int_codec, Left, message1.volume);
            set_input_volume(ext_int_codec, Right, message1.volume);

            break;
        case SetOutputVolume:
            VolumeMessage message2;
            memcpy(&message2, &data[1], sizeof(VolumeMessage));

            ESP_LOGI("MESSAGES", "Setting output volume to %d",
                     message2.volume);

            set_output_volume(ext_int_codec, Left, message2.volume);
            set_output_volume(ext_int_codec, Right, message2.volume);

            break;
        case SetDACVolume:
            VolumeMessage message3;
            memcpy(&message3, &data[1], sizeof(VolumeMessage));

            ESP_LOGI("MESSAGES", "Setting DAC volume to %d", message3.volume);

            set_dac_volume(ext_int_codec, Left, message3.volume);
            set_dac_volume(ext_int_codec, Right, message3.volume);

            break;
    }
}

void init_bt_control(spi_codec_device _ext_int_codec) {
    ext_int_codec = _ext_int_codec;
}