use crate::adapters;
use crate::device::Usb;
use crate::error;
use crate::usb;
use rusb::UsbContext;

macro_rules! register {
    ($($module:ident),+) => {
        paste::paste! {
            $(
                pub mod $module;
            )+

            #[derive(Debug, Copy, Clone)]
            pub enum Type {
                $(
                    [<$module:camel>],
                )+
            }

            impl std::fmt::Display for Type {
                fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $(
                            Self::[<$module:camel>] => write!(formatter, stringify!($module)),
                        )+
                    }
                }
            }

            impl Type {
                pub fn name(self) -> &'static str  {
                    match self {
                        $(
                            Type::[<$module:camel>] => $module::Device::PROPERTIES.name,
                        )+
                    }
                }
            }

            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            #[serde(tag = "type", content = "configuration")]
            #[allow(clippy::large_enum_variant)]
            pub enum Configuration {
                $(
                    #[serde(rename = "" $module)]
                    [<$module:camel>]($module::Configuration),
                )+
            }

            impl Configuration {
                pub fn deserialize_bincode(
                    device_type: Type,
                    data: &[u8]
                ) -> bincode::Result<Configuration> {
                    match device_type {
                        $(
                            Type::[<$module:camel>] => Ok(
                                Configuration::[<$module:camel>](bincode::deserialize(data)?)
                            ),
                        )+
                    }
                }
            }

            pub enum Device {
                $(
                    [<$module:camel>]($module::Device),
                )+
            }

            pub struct ListedDevice {
                pub device_type: Type,
                pub serial: String,
                pub speed: usb::Speed,
            }

            pub fn list_devices() -> rusb::Result<Vec<ListedDevice>> {
                let context = rusb::Context::new()?;
                let devices = context.devices()?;
                let mut result = Vec::new();
                $(
                    result.extend(
                        $module::Device::list_serials_and_speeds(&devices)?
                            .into_iter()
                            .map(|(serial, speed)| ListedDevice {
                                device_type: Type::[<$module:camel>],
                                serial,
                                speed,
                            }),
                    );
                )+
                Ok(result)
            }

            pub fn open(
                serial: Option<&str>,
                configuration: Option<Configuration>,
                usb_configuration: Option<usb::Configuration>,
                event_loop: std::sync::Arc<usb::EventLoop>,
                error_flag: error::Flag<Error>,
            ) -> Result<Device, Error>
            {
                match configuration {
                    Some(configuration) => {
                        match configuration {
                            $(
                                Configuration::[<$module:camel>](configuration) => Ok(
                                    $module::Device::open(
                                        &serial,
                                        configuration,
                                        usb_configuration
                                        .as_ref()
                                        .unwrap_or(&$module::Device::DEFAULT_USB_CONFIGURATION),
                                        event_loop.clone(),
                                        error_flag.clone(),
                                    )
                                    .map(|device| paste::paste! {Device::[<$module:camel>](device)})
                                    .map_err(|error| Error::from(error).unpack())?
                                ),
                            )+
                        }
                    },
                    None => {
                        $(
                            match $module::Device::open(
                                &serial,
                                $module::Device::PROPERTIES.default_configuration.clone(),
                                usb_configuration
                                .as_ref()
                                .unwrap_or(&$module::Device::DEFAULT_USB_CONFIGURATION),
                                event_loop.clone(),
                                error_flag.clone(),
                            ) {
                                Ok(device) => return Ok(Device::[<$module:camel>](device)),
                                Err(error) => match Error::from(error).unpack() {
                                    Error::DeviceWithSerial {device_type: _, serial: _} => (),
                                    Error::Device(_) => (),
                                    error => return Err(error.into()),
                                }
                            };
                        )+
                        Err(match serial {
                            Some(serial) => Error::Serial(serial.to_owned()),
                            None => Error::NoDevice
                        })
                    }
                }
            }

            #[derive(Debug, serde::Serialize)]
            pub enum Properties {
                $(
                    #[serde(rename = "" $module)]
                    [<$module:camel>](<$module::Device as Usb>::Properties),
                )+
            }

            impl Device {
                pub fn adapter(&self) -> adapters::Adapter {
                    match self {
                        $(
                            Self::[<$module:camel>](device) => device.adapter().into(),
                        )+
                    }
                }

                pub fn next_with_timeout(&mut self, timeout: &std::time::Duration) -> Option<usb::BufferView> {
                    match self {
                        $(
                            Self::[<$module:camel>](device) => device.next_with_timeout(timeout),
                        )+
                    }
                }

                pub fn properties(&self) -> Properties {
                    match self {
                        $(
                            Self::[<$module:camel>](_) => Properties::[<$module:camel>]($module::Device::PROPERTIES),
                        )+
                    }
                }

                pub fn name(&self) -> &'static str {
                    match self {
                        $(
                            Self::[<$module:camel>](_) => $module::Device::PROPERTIES.name,
                        )+
                    }
                }

                pub fn serial(&self) -> String {
                    match self {
                        $(
                            Self::[<$module:camel>](device) => device.serial(),
                        )+
                    }
                }

                pub fn speed(&self) -> usb::Speed {
                    match self {
                        $(
                            Self::[<$module:camel>](device) => device.speed(),
                        )+
                    }
                }
            }

            #[derive(Debug, PartialEq, Eq)]
            pub struct ParseTypeError {
                on: String
            }

            impl std::fmt::Display for ParseTypeError {
                fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "unknow device type \"{}\"", self.on)
                }
            }

            impl std::str::FromStr for Type {
                type Err = ParseTypeError;

                fn from_str(string: &str) -> Result<Self, Self::Err> {
                    match string {
                        $(
                            stringify!($module) => paste::paste! {Ok(Self::[<$module:camel>])},
                        )+
                        _ => Err(Self::Err {on: string.to_owned()}),
                    }
                }
            }

            #[derive(thiserror::Error, Debug, Clone)]
            pub enum Error {
                #[error("{0}")]
                Usb(#[from] usb::Error),

                #[error("{device_type} with serial \"{serial}\" not found")]
                DeviceWithSerial { device_type: Type, serial: String },

                #[error("no {0} found")]
                Device(Type),

                #[error("serial \"{0}\" not found")]
                Serial(String),

                #[error("no device found")]
                NoDevice,

                #[error("control transfer error (expected {expected:?}, read {read:?})")]
                Mismatch { expected: Vec<u8>, read: Vec<u8> },

                $(
                    #[error(transparent)]
                    [<$module:camel>](#[from] $module::Error),
                )+
            }

            impl Error {
                pub fn unpack(self) -> Self {
                    match self {
                        $(
                            Self::[<$module:camel>](error) => {
                                match error {
                                    $module::Error::Usb(error) => match error {
                                        usb::Error::Serial(serial) => Self::DeviceWithSerial {
                                            device_type: Type::[<$module:camel>],
                                            serial,
                                        },
                                        usb::Error::Device => Self::Device(Type::[<$module:camel>]),
                                        error => Self::[<$module:camel>]($module::Error::Usb(error)),
                                    },
                                    #[allow(unreachable_patterns)]  // devices may not need extra errors besides "usb::Error"
                                    error => Self::[<$module:camel>](error)
                                }
                            }
                        )+
                        error => error
                    }
                }
            }
        }
    };
}

register! { prophesee_evk3_hd, prophesee_evk4 }
