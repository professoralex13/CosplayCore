use esp_idf_hal::{
    delay::FreeRtos, modem::BluetoothModemPeripheral, peripheral::Peripheral, sys::EspError,
};
use esp_idf_svc::{
    bt::{
        a2dp::{A2dpEvent, EspA2dp},
        gap::EspGap,
        BtClassic, BtDriver,
    },
    nvs::EspDefaultNvsPartition,
};

pub struct Bluetooth<'a> {
    driver: BtDriver<'a, BtClassic>,

    bluetooth_sender: &'static global_channel::crossbeam_channel::Sender<Vec<u8>>,
}

impl<'a> Bluetooth<'a> {
    pub fn new(
        modem: impl Peripheral<P = impl BluetoothModemPeripheral> + 'a,
        bluetooth_sender: &'static global_channel::crossbeam_channel::Sender<Vec<u8>>,
    ) -> Result<Self, EspError> {
        let nvs = EspDefaultNvsPartition::take()?;

        Ok(Self {
            driver: BtDriver::<BtClassic>::new(modem, Some(nvs))?,
            bluetooth_sender,
        })
    }

    pub fn entrypoint(mut self) -> Result<(), EspError> {
        self.driver.set_device_name("CosplayCore")?;

        let gap_server = EspGap::new(&self.driver)?;

        gap_server.set_scan_mode(true, esp_idf_svc::bt::gap::DiscoveryMode::Discoverable)?;

        let a2dp_sink = EspA2dp::new_sink(&self.driver)?;

        a2dp_sink.subscribe(move |event| {
            match event {
                A2dpEvent::SinkData(data) => {
                    if let Err(_) = self.bluetooth_sender.try_send(data.to_vec()) {
                        log::warn!("Bluetooth buffer full");
                    }
                }
                A2dpEvent::ConnectionState { status, .. } => {
                    // if status == ConnectionStatus::Connected {
                    //     log::info!("Bluetooth device connected");

                    //     codec1.set_dac_mute(false).unwrap();
                    // } else {
                    //     codec1.set_dac_mute(true).unwrap();
                    // }
                }
                A2dpEvent::AudioCodecConfigured { codec, .. } => {
                    log::info!("Connected to codec: {codec:?}");
                }
                _ => {}
            };
            0
        })?;

        loop {
            FreeRtos::delay_ms(10);
        }
    }
}
