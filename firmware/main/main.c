#include "bluetooth/bluetooth.h"
#include "codec/i2s.h"
#include "codec/settings.h"
#include "codec/spi.h"
#include "hal/spi_types.h"

//! Changed between board versions V1.0 and V1.1 due to unsuitable GPIO issues
#define SPI_MOSI_PIN 22

#define SPI_CLK_PIN 32

#define EXT_INT_CSB_PIN 4

#define EXT_INT_CLK_PIN 16
#define EXT_INT_LRC_PIN 5
#define EXT_INT_DAC_PIN 17
#define EXT_INT_ADC_PIN 15

#define INT_EXT_CSB_PIN 23

#define INT_EXT_CLK_PIN 13
#define INT_EXT_LRC_PIN 14
#define INT_EXT_DAC_PIN 27
#define INT_EXT_ADC_PIN 12

i2s_chan_handle_t ext_int_tx;
i2s_chan_handle_t ext_int_rx;

i2s_chan_handle_t int_ext_tx;
i2s_chan_handle_t int_ext_rx;

void app_main(void) {
    spi_bus_init(SPI2_HOST, SPI_CLK_PIN, SPI_MOSI_PIN);

    spi_codec_device ext_int_codec;
    spi_codec_device int_ext_codec;

    spi_device_init(EXT_INT_CSB_PIN, &ext_int_codec);
    spi_device_init(INT_EXT_CSB_PIN, &int_ext_codec);

    reset_registers(ext_int_codec);
    reset_registers(int_ext_codec);

    set_power_management(ext_int_codec, false, false, true, true, true, true);
    set_power_management(int_ext_codec, false, false, true, true, true, true);

    set_digital_audio_interface(ext_int_codec, 16);
    set_digital_audio_interface(int_ext_codec, 16);

    set_dac_volume(ext_int_codec, Left, MAX_DAC_VOLUME);
    set_dac_volume(ext_int_codec, Right, MAX_DAC_VOLUME);

    set_dac_volume(int_ext_codec, Left, MAX_DAC_VOLUME);
    set_dac_volume(int_ext_codec, Right, MAX_DAC_VOLUME);

    set_dac_mute(ext_int_codec, true);
    set_dac_mute(int_ext_codec, true);

    set_input_volume(ext_int_codec, Left, MAX_INPUT_VOLUME);
    set_input_volume(ext_int_codec, Right, MAX_INPUT_VOLUME);

    set_input_volume(int_ext_codec, Left, MAX_INPUT_VOLUME);
    set_input_volume(int_ext_codec, Right, MAX_INPUT_VOLUME);

    set_output_mix(ext_int_codec, Left, MAX_MIX_VOLUME, 0);
    set_output_mix(ext_int_codec, Right, 0, MAX_MIX_VOLUME);

    set_output_mix(int_ext_codec, Left, MAX_MIX_VOLUME, 0);
    set_output_mix(int_ext_codec, Right, 0, MAX_MIX_VOLUME);

    set_output_volume(ext_int_codec, Left, MAX_OUTPUT_VOLUME);
    set_output_volume(ext_int_codec, Right, MAX_OUTPUT_VOLUME);

    set_output_volume(int_ext_codec, Left, MAX_OUTPUT_VOLUME);
    set_output_volume(int_ext_codec, Right, MAX_OUTPUT_VOLUME);

    i2s_device_init(&ext_int_tx, &ext_int_rx, EXT_INT_CLK_PIN, EXT_INT_LRC_PIN,
                    EXT_INT_DAC_PIN, EXT_INT_ADC_PIN);

    i2s_device_init(&int_ext_tx, &int_ext_rx, INT_EXT_CLK_PIN, INT_EXT_LRC_PIN,
                    INT_EXT_DAC_PIN, INT_EXT_ADC_PIN);

    bluetooth_init(ext_int_tx, ext_int_codec);
}
