use core::ops::Range;

use embedded_hal::{
    digital::OutputPin,
    spi::{SpiBus, SpiDevice},
};
use embedded_hal_bus::spi::DeviceError;

use self::consts::{AudioWordLength, MAX_MIX_VOLUME, RegisterAddress};
use super::{AudioChannel, ChannelPair, Codec, PowerConfig};

pub mod consts;

fn mask(n: u8) -> u16 {
    if n >= 16 {
        u16::MAX
    } else if n == 0 {
        0
    } else {
        (1u16 << n) - 1
    }
}

fn pack<T: Into<u16>>(value: T, bits: Range<u8>) -> u16 {
    let repr: u16 = value.into();
    let n = bits.end - bits.start;

    let max = 2_u16.pow(n as u32) - 1;

    if repr > max {
        panic!("Attempted to mask integer with value {repr} to {n} bits, whose maximum is {max}");
    }

    return (repr & mask(n)) << bits.start;
}

impl<'a, BUS: SpiBus, CS: OutputPin> Codec<'a, BUS, CS> {
    fn write_register(
        &mut self,
        address: RegisterAddress,
        value: u16,
    ) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        let data = pack(address as u16, 9..16) | pack(value, 0..9);

        self.spi.write(&data.to_be_bytes())
    }

    pub fn get_input_volume(&self, channel: AudioChannel) -> u8 {
        self.input_volume.get(channel)
    }

    pub fn set_input_volume(
        &mut self,
        channel: AudioChannel,
        volume: u8,
    ) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        let address = match channel {
            AudioChannel::Left => RegisterAddress::LeftInputVolume,
            AudioChannel::Right => RegisterAddress::RightInputVolume,
        };

        let update_immediate = true;
        let mute = volume == 0;
        let zero_cross_detector = false;

        let data = pack(update_immediate, 8..9)
            | pack(mute, 7..8)
            | pack(zero_cross_detector, 6..7)
            | pack(volume, 0..6);

        self.write_register(address, data)?;

        self.input_volume.set(channel, volume);

        Ok(())
    }

    pub fn get_output_volume(&self, channel: AudioChannel) -> u8 {
        self.output_volume.get(channel)
    }

    pub fn set_output_volume(
        &mut self,
        channel: AudioChannel,
        volume: u8,
    ) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        let address = match channel {
            AudioChannel::Left => RegisterAddress::LeftOutput1Volume,
            AudioChannel::Right => RegisterAddress::RightOutput1Volume,
        };

        let update_immediate = true;

        self.write_register(address, pack(update_immediate, 8..9) | pack(volume, 0..8))?;

        self.output_volume.set(channel, volume);

        Ok(())
    }

    pub fn get_power_config(&self) -> PowerConfig {
        self.power_config
    }

    pub fn set_power_management(
        &mut self,
        config: PowerConfig,
    ) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        let vmid_selection: u8 = 0b01;
        let vref = true;

        let lout2 = false;
        let rout2 = false;

        let master_clk_disabled = false;

        let data1 = pack(vmid_selection, 7..9)
            | pack(vref, 6..7)
            | pack(config.pga_left, 5..6)
            | pack(config.pga_right, 4..5)
            | pack(config.adc_left, 3..4)
            | pack(config.adc_right, 2..3)
            | pack(master_clk_disabled, 0..1);

        let data2 = pack(config.dac_left, 8..9)
            | pack(config.dac_right, 7..8)
            | pack(config.left_out_1, 6..7)
            | pack(config.right_out_1, 5..6)
            | pack(lout2, 4..5)
            | pack(rout2, 3..4);

        self.write_register(RegisterAddress::PowerManagement1, data1)?;

        self.write_register(RegisterAddress::PowerManagement2, data2)?;

        self.power_config = config;

        Ok(())
    }

    pub fn get_mix(&self, channel: AudioChannel) -> ChannelPair<u8> {
        self.mix.get(channel)
    }

    pub fn set_mix(
        &mut self,
        channel: AudioChannel,
        left_volume: u8,
        right_volume: u8,
    ) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        let address1 = if channel == AudioChannel::Left {
            RegisterAddress::LeftOutMix1
        } else {
            RegisterAddress::RightOutMix1
        };
        let address2 = if channel == AudioChannel::Left {
            RegisterAddress::LeftOutMix2
        } else {
            RegisterAddress::RightOutMix2
        };

        pub enum MixInputSelection {
            Input1 = 0b000,
            Input2 = 0b001,
            MicBoost = 0b011,
            Differential = 0b100,
        }

        let mix_input_selection = MixInputSelection::MicBoost;

        let left_dac = channel == AudioChannel::Left;
        let right_dac = channel == AudioChannel::Right;

        let left_mix_enabled = left_volume != 0;
        let right_mix_enabled = right_volume != 0;

        let data1 = pack(left_dac, 8..9)
            | pack(left_mix_enabled, 7..8)
            | pack(MAX_MIX_VOLUME - left_volume, 4..7)
            | pack(mix_input_selection as u8, 0..3);

        let data2 = pack(right_dac, 8..9)
            | pack(right_mix_enabled, 7..8)
            | pack(MAX_MIX_VOLUME - right_volume, 4..7);

        self.write_register(address1, data1)?;
        self.write_register(address2, data2)?;

        self.mix
            .set(channel, ChannelPair::new(left_volume, right_volume));

        Ok(())
    }

    pub fn set_digital_audio_interface(
        &mut self,
        word_length: AudioWordLength,
    ) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        let invert_bclk = false;
        let master_mode = false;
        let swap_left_right = false;
        let invert_lrc_polarity = false;

        enum AudioDataFormat {
            LeftJustified = 0b01,
            I2S = 0b10,
            DSP = 0b11,
        }

        let audio_format = AudioDataFormat::I2S;

        let data = pack(invert_bclk, 7..8)
            | pack(master_mode, 6..7)
            | pack(swap_left_right, 5..6)
            | pack(invert_lrc_polarity, 4..5)
            | pack(word_length as u8, 2..4)
            | pack(audio_format as u8, 0..2);

        self.write_register(RegisterAddress::AudioInterface, data)?;

        Ok(())
    }

    pub fn get_dac_volume(&self, channel: AudioChannel) -> u8 {
        self.dac_volume.get(channel)
    }

    pub fn set_dac_volume(
        &mut self,
        channel: AudioChannel,
        volume: u8,
    ) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        let address = if channel == AudioChannel::Left {
            RegisterAddress::LeftDACVolume
        } else {
            RegisterAddress::RightDACVolume
        };

        let update_immediate = true;

        let data = pack(update_immediate, 8..9) | pack(volume, 0..8);

        self.write_register(address, data)?;

        self.dac_volume.set(channel, volume);

        Ok(())
    }

    pub fn set_dac_mute(&mut self, mute: bool) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        let adc_attenuate = false;
        let dac_attenuate = false;

        let adchpd = false;

        let hpor = false;
        let adcpol: u8 = 0b00;

        let demphasis: u8 = 0b00;

        let data = pack(adc_attenuate, 8..9)
            | pack(dac_attenuate, 7..8)
            | pack(adcpol, 5..7)
            | pack(hpor, 4..5)
            | pack(mute, 3..4)
            | pack(demphasis, 1..3)
            | pack(adchpd, 0..1);

        self.write_register(RegisterAddress::ADCDACControl, data)?;

        Ok(())
    }

    pub fn reset_codec(&mut self) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        self.write_register(RegisterAddress::Reset, 0)?;

        self.set_input_volume(AudioChannel::Left, 0)?;
        self.set_input_volume(AudioChannel::Right, 0)?;

        self.set_output_volume(AudioChannel::Left, 0)?;
        self.set_output_volume(AudioChannel::Right, 0)?;

        self.set_power_management(Default::default())?;

        self.set_mix(AudioChannel::Left, 0, 0)?;
        self.set_mix(AudioChannel::Right, 0, 0)?;

        self.set_digital_audio_interface(AudioWordLength::Bit16)?;

        self.set_dac_volume(AudioChannel::Left, 0)?;
        self.set_dac_volume(AudioChannel::Right, 0)?;

        self.set_dac_mute(false)?;

        Ok(())
    }
}
