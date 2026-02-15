use esp_idf_hal::{
    gpio::OutputPin,
    i2s::{I2sBiDir, I2sDriver},
    peripheral::Peripheral,
    spi::{config::Config as SpiConfig, SpiDeviceDriver, SpiDriver},
};
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

pub struct Codec<'a> {
    spi_device: SpiDeviceDriver<'a, &'a SpiDriver<'a>>,

    pub i2s_driver: I2sDriver<'a, I2sBiDir>,

    input_volume: ChannelPair<u8>,

    output_volume: ChannelPair<u8>,

    mix: ChannelPair<ChannelPair<u8>>,

    power_config: PowerConfig,

    dac_volume: ChannelPair<u8>,
}

impl<'a> Codec<'a> {
    pub fn new(
        spi_driver: &'a SpiDriver,
        spi_cs: impl Peripheral<P = impl OutputPin> + 'a,
        i2s_driver: I2sDriver<'a, I2sBiDir>,
    ) -> anyhow::Result<Self> {
        let mut this = Self {
            spi_device: SpiDeviceDriver::new(spi_driver, Some(spi_cs), &SpiConfig::new())?,
            i2s_driver,
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
