use crate::error;
use crate::usb;
use rusb::UsbContext;

pub trait Usb: Sized {
    type Adapter;
    type Configuration;
    type Error;
    type Properties;

    const VENDOR_ID: u16;

    const PRODUCT_ID: u16;

    const PROPERTIES: Self::Properties;

    const DEFAULT_USB_CONFIGURATION: usb::Configuration;

    fn read_serial(handle: &mut rusb::DeviceHandle<rusb::Context>) -> rusb::Result<String>;

    fn update_configuration(&self, configuration: Self::Configuration);

    fn open<IntoError>(
        serial: &Option<&str>,
        configuration: Self::Configuration,
        usb_configuration: &usb::Configuration,
        event_loop: std::sync::Arc<usb::EventLoop>,
        error_flag: error::Flag<IntoError>,
    ) -> Result<Self, Self::Error>
    where
        IntoError: From<Self::Error> + Clone + Send + 'static;

    fn next_with_timeout(&mut self, timeout: &std::time::Duration) -> Option<usb::BufferView>;

    fn serial(&self) -> String;

    fn speed(&self) -> usb::Speed;

    fn adapter(&self) -> Self::Adapter;

    fn list_serials_and_speeds(
        devices: &rusb::DeviceList<rusb::Context>,
    ) -> rusb::Result<Vec<(String, usb::Speed)>> {
        let mut result = Vec::new();
        for device in devices
            .iter()
            .filter(|device| match device.device_descriptor() {
                Ok(descriptor) => {
                    descriptor.vendor_id() == Self::VENDOR_ID
                        && descriptor.product_id() == Self::PRODUCT_ID
                }
                Err(_) => false,
            })
        {
            result.push((
                Self::read_serial(&mut device.open()?)?,
                device.speed().into(),
            ));
        }
        Ok(result)
    }

    fn handle_from_serial(
        context: &rusb::Context,
        serial: &Option<&str>,
    ) -> Result<rusb::DeviceHandle<rusb::Context>, usb::Error> {
        match context.devices()?.iter().find_map(
            |device| -> Option<rusb::Result<rusb::DeviceHandle<rusb::Context>>> {
                match device.device_descriptor() {
                    Ok(descriptor) => {
                        if descriptor.vendor_id() == Self::VENDOR_ID
                            && descriptor.product_id() == Self::PRODUCT_ID
                        {
                            let mut handle = match device.open() {
                                Ok(handle) => handle,
                                Err(error) => return Some(Err(error)),
                            };
                            match serial {
                                Some(serial) => {
                                    let device_serial = match Self::read_serial(&mut handle) {
                                        Ok(serial) => serial,
                                        Err(error) => return Some(Err(error)),
                                    };
                                    if *serial == device_serial {
                                        Some(Ok(handle))
                                    } else {
                                        None
                                    }
                                }
                                None => Some(Ok(handle)),
                            }
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            },
        ) {
            Some(result) => Ok(result?),
            None => Err(match serial {
                Some(serial) => usb::Error::Serial((*serial).to_owned()),
                None => usb::Error::Device,
            }),
        }
    }
}
