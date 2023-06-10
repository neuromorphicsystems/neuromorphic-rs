use crate::adapters;
use crate::device;
use crate::error;
use crate::properties;
use crate::usb;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Configuration {}

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("{0}")]
    Usb(#[from] usb::Error),
}

impl From<rusb::Error> for Error {
    fn from(error: rusb::Error) -> Self {
        usb::Error::from(error).into()
    }
}
pub struct Device {
    handle: std::sync::Arc<rusb::DeviceHandle<rusb::Context>>,
    runtime_error: std::sync::Arc<std::sync::Mutex<Option<Error>>>,
    serial: String,
}

impl device::Usb for Device {
    type Adapter = adapters::evt3::Adapter;

    type Configuration = Configuration;

    type Error = Error;

    type Properties = properties::Camera<Self::Configuration>;

    const VENDOR_ID: u16 = 0x04b4;

    const PRODUCT_ID: u16 = 0x00f4;

    const PROPERTIES: Self::Properties = properties::Camera::<Self::Configuration> {
        name: "Prophesee EVK3 HD",
        width: 1280,
        height: 720,
        default_configuration: Self::Configuration {},
    };

    const DEFAULT_USB_CONFIGURATION: usb::Configuration = usb::Configuration {
        buffer_size: 1 << 17,
        ring_size: 1 << 12,
        transfer_queue_size: 1 << 5,
    };

    fn read_serial(handle: &mut rusb::DeviceHandle<rusb::Context>) -> rusb::Result<String> {
        Ok("".to_owned())
    }

    fn update_configuration(&self, configuration: Self::Configuration) {}

    fn open<IntoError>(
        serial: &Option<&str>,
        configuration: Self::Configuration,
        usb_configuration: &usb::Configuration,
        event_loop: std::sync::Arc<usb::EventLoop>,
        error_flag: error::Flag<IntoError>,
    ) -> Result<Self, Self::Error>
    where
        IntoError: From<Self::Error> + Clone + Send,
    {
        let mut handle = Self::handle_from_serial(event_loop.context(), serial)?;
        handle.claim_interface(0)?;
        let handle = std::sync::Arc::new(handle);
        Ok(Device {
            handle,
            runtime_error: std::sync::Arc::new(std::sync::Mutex::new(None)),
            serial: "".to_owned(),
        })
    }

    fn next_with_timeout(&mut self, timeout: &std::time::Duration) -> Option<usb::BufferView> {
        None
    }

    fn serial(&self) -> String {
        self.serial.clone()
    }

    fn speed(&self) -> usb::Speed {
        self.handle.device().speed().into()
    }

    fn adapter(&self) -> Self::Adapter {
        Self::Adapter::from_dimensions(Self::PROPERTIES.width, Self::PROPERTIES.height)
    }
}
