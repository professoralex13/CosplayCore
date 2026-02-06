#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::cell::RefCell;

use esp_hal::{clock::CpuClock, gpio::Level};

mod codec;
use codec::Codec;
use esp_hal::{
    gpio::Output,
    main, spi,
    spi::master::Spi,
    time::{Duration, Instant},
};

use crate::codec::{
    AudioChannel, MAX_INPUT_VOLUME, MAX_MIX_VOLUME, MAX_OUTPUT_VOLUME, PowerConfig,
};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // generator version: 1.1.0
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);

    let ext_int_csb = Output::new(_peripherals.GPIO4, Level::High, Default::default());

    let spi_bus = RefCell::new(
        Spi::new(_peripherals.SPI2, spi::master::Config::default())
            .unwrap()
            .with_mosi(_peripherals.GPIO22)
            .with_sck(_peripherals.GPIO32),
    );

    let mut codec1 = Codec::new(&spi_bus, ext_int_csb).unwrap();

    codec1
        .set_power_management(PowerConfig {
            adc_left: false,
            adc_right: false,
            dac_left: true,
            dac_right: true,
            left_out_1: true,
            right_out_1: true,
            pga_left: true,
            pga_right: true,
        })
        .unwrap();

    codec1
        .set_input_volume(AudioChannel::Left, MAX_INPUT_VOLUME)
        .unwrap();
    codec1
        .set_input_volume(AudioChannel::Right, MAX_INPUT_VOLUME)
        .unwrap();

    codec1
        .set_output_volume(AudioChannel::Left, MAX_OUTPUT_VOLUME)
        .unwrap();
    codec1
        .set_output_volume(AudioChannel::Right, MAX_OUTPUT_VOLUME)
        .unwrap();

    codec1
        .set_mix(AudioChannel::Left, MAX_MIX_VOLUME, 0)
        .unwrap();
    codec1
        .set_mix(AudioChannel::Right, 0, MAX_MIX_VOLUME)
        .unwrap();

    loop {
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v~1.0/examples
}
