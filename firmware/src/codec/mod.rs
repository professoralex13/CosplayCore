use core::cell::RefCell;

use embedded_hal::{digital::OutputPin, spi::SpiBus};
use embedded_hal_bus::spi::{DeviceError, RefCellDevice};
use esp_hal::delay::Delay;

pub mod spi;

#[derive(PartialEq, Copy, Clone)]
pub enum AudioChannel {
    Left,
    Right,
}

#[derive(Default, Clone, Copy)]
pub struct ChannelPair<T: Default> {
    left: T,
    right: T,
}

impl<T: Default + Copy> ChannelPair<T> {
    pub fn new(left: T, right: T) -> Self {
        Self { left, right }
    }

    pub fn get(&self, channel: AudioChannel) -> T {
        match channel {
            AudioChannel::Left => self.left,
            AudioChannel::Right => self.right,
        }
    }

    pub fn set(&mut self, channel: AudioChannel, value: T) {
        match channel {
            AudioChannel::Left => self.left = value,
            AudioChannel::Right => self.right = value,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct PowerConfig {
    pub adc_left: bool,
    pub adc_right: bool,
    pub dac_left: bool,
    pub dac_right: bool,
    pub left_out_1: bool,
    pub right_out_1: bool,
    pub pga_left: bool,
    pub pga_right: bool,
}

pub struct Codec<'a, BUS: SpiBus, CS: OutputPin> {
    spi: RefCellDevice<'a, BUS, CS, Delay>,

    input_volume: ChannelPair<u8>,

    output_volume: ChannelPair<u8>,

    mix: ChannelPair<ChannelPair<u8>>,

    power_config: PowerConfig,

    dac_volume: ChannelPair<u8>,
}

impl<'a, BUS: SpiBus, CS: OutputPin> Codec<'a, BUS, CS> {
    pub fn new(
        spi_bus: &'a RefCell<BUS>,
        spi_cs: CS,
    ) -> Result<Self, DeviceError<BUS::Error, CS::Error>> {
        let mut this = Self {
            spi: RefCellDevice::new(spi_bus, spi_cs, Delay::new())
                .map_err(|err| DeviceError::<BUS::Error, CS::Error>::Cs(err))?,
            input_volume: Default::default(),
            output_volume: Default::default(),
            power_config: Default::default(),
            mix: Default::default(),
            dac_volume: Default::default(),
        };

        this.reset_codec()?;

        Ok(this)
    }
}
