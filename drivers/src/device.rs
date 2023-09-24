use crate::error;
use crate::usb;
use rusb::UsbContext;

pub struct ListedDevice {
    pub speed: usb::Speed,
    pub serial: Result<String, usb::Error>,
}

pub trait Usb: Sized {
    type Adapter;
    type Configuration;
    type Error;
    type Properties;

    const VENDOR_ID: u16;

    const PRODUCT_ID: u16;

    const PROPERTIES: Self::Properties;

    const DEFAULT_USB_CONFIGURATION: usb::Configuration;

    // read_serial must claim bulk transfer interface(s)
    // this is required even if read_serial does not use bulk transfers
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

    fn next_with_timeout(&self, timeout: &std::time::Duration) -> Option<usb::BufferView>;

    fn serial(&self) -> String;

    fn chip_firmware_configuration(&self) -> Self::Configuration;

    fn speed(&self) -> usb::Speed;

    fn adapter(&self) -> Self::Adapter;

    fn list_devices(devices: &rusb::DeviceList<rusb::Context>) -> rusb::Result<Vec<ListedDevice>> {
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
            result.push(ListedDevice {
                speed: device.speed().into(),
                serial: Self::read_serial(&mut device.open()?).map_err(|error| error.into()),
            });
        }
        Ok(result)
    }

    fn handle_from_serial(
        context: &rusb::Context,
        serial: &Option<&str>,
    ) -> Result<(rusb::DeviceHandle<rusb::Context>, String), usb::Error> {
        match context.devices()?.iter().find_map(
            |device| -> Option<rusb::Result<(rusb::DeviceHandle<rusb::Context>, String)>> {
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
                                        Err(_) => return None, // ignore errors to support devices that are already open
                                    };
                                    if *serial == device_serial {
                                        let _ = handle.set_auto_detach_kernel_driver(true);
                                        Some(Ok((handle, device_serial)))
                                    } else {
                                        None
                                    }
                                }
                                None => {
                                    let device_serial = match Self::read_serial(&mut handle) {
                                        Ok(serial) => serial,
                                        Err(_) => return None, // ignore errors to support devices that are already open
                                    };
                                    let _ = handle.set_auto_detach_kernel_driver(true);
                                    Some(Ok((handle, device_serial)))
                                }
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
