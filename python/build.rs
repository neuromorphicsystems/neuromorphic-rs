use neuromorphic_drivers::device::Usb;
use std::io::Write;

fn quote_type(
    format: &serde_reflection::Format,
    name_to_new_name: &std::collections::HashMap<String, String>,
) -> String {
    match format {
        serde_reflection::Format::TypeName(name) => {
            name_to_new_name.get(name).unwrap_or_else(|| &name).clone()
        }
        serde_reflection::Format::Unit => "serde.type.unit".into(),
        serde_reflection::Format::Bool => "bool".into(),
        serde_reflection::Format::I8 => "serde.type.int8".into(),
        serde_reflection::Format::I16 => "serde.type.int16".into(),
        serde_reflection::Format::I32 => "serde.type.int32".into(),
        serde_reflection::Format::I64 => "serde.type.int64".into(),
        serde_reflection::Format::I128 => "serde.type.int128".into(),
        serde_reflection::Format::U8 => "serde.type.uint8".into(),
        serde_reflection::Format::U16 => "serde.type.uint16".into(),
        serde_reflection::Format::U32 => "serde.type.uint32".into(),
        serde_reflection::Format::U64 => "serde.type.uint64".into(),
        serde_reflection::Format::U128 => "serde.type.uint128".into(),
        serde_reflection::Format::F32 => "serde.type.float32".into(),
        serde_reflection::Format::F64 => "serde.type.float64".into(),
        serde_reflection::Format::Char => "serde.type.char".into(),
        serde_reflection::Format::Str => "str".into(),
        serde_reflection::Format::Bytes => "bytes".into(),
        serde_reflection::Format::Option(format) => {
            format!("{} | None", quote_type(format, name_to_new_name))
        }
        serde_reflection::Format::Seq(format) => {
            format!("list[{}]", quote_type(format, name_to_new_name))
        }
        serde_reflection::Format::Map { key, value } => {
            format!(
                "dict[{}, {}]",
                quote_type(key, name_to_new_name),
                quote_type(value, name_to_new_name)
            )
        }
        serde_reflection::Format::Tuple(formats) => {
            if formats.is_empty() {
                "tuple[()]".into()
            } else {
                format!("tuple[{}]", quote_types(formats, name_to_new_name))
            }
        }
        serde_reflection::Format::TupleArray { content, size } => format!(
            "tuple[\n        {}\n    ]",
            quote_types(&vec![content.as_ref().clone(); *size], name_to_new_name)
        ),
        serde_reflection::Format::Variable(_) => panic!("unexpected value"),
    }
}

fn quote_types(
    formats: &[serde_reflection::Format],
    name_to_new_name: &std::collections::HashMap<String, String>,
) -> String {
    format!(
        "{},",
        formats
            .iter()
            .map(|format| quote_type(format, name_to_new_name))
            .collect::<Vec<_>>()
            .join(",\n        ")
    )
}

fn value_to_string(value: &serde_reflection::Value) -> String {
    match value {
        serde_reflection::Value::Unit => "()".to_owned(),
        serde_reflection::Value::Bool(value) => (if *value { "True" } else { "False" }).to_owned(),
        serde_reflection::Value::I8(value) => value.to_string(),
        serde_reflection::Value::I16(value) => value.to_string(),
        serde_reflection::Value::I32(value) => value.to_string(),
        serde_reflection::Value::I64(value) => value.to_string(),
        serde_reflection::Value::I128(value) => value.to_string(),
        serde_reflection::Value::U8(value) => format!("{:#04X}", value),
        serde_reflection::Value::U16(value) => value.to_string(),
        serde_reflection::Value::U32(value) => value.to_string(),
        serde_reflection::Value::U64(value) => value.to_string(),
        serde_reflection::Value::U128(value) => value.to_string(),
        serde_reflection::Value::F32(value) => value.to_string(),
        serde_reflection::Value::F64(value) => value.to_string(),
        serde_reflection::Value::Char(value) => value.to_string(),
        serde_reflection::Value::Str(value) => format!("\"{}\"", value.replace("\"", "\\\"")),
        serde_reflection::Value::Bytes(value) => format!(
            "[{}]",
            value
                .iter()
                .map(|value| format!("{:#04x}", value))
                .collect::<Vec<String>>()
                .join(", ")
        ),
        serde_reflection::Value::Option(value) => value
            .as_ref()
            .map_or_else(|| "None".to_owned(), |value| value_to_string(&value)),
        serde_reflection::Value::Variant(_, value) => value_to_string(&value),
        serde_reflection::Value::Seq(value) => format!(
            "({})",
            value
                .into_iter()
                .map(value_to_string)
                .collect::<Vec<String>>()
                .join(", ")
        ),
    }
}

struct Node {
    name: String,
    children: std::collections::HashSet<String>,
    fields: Vec<serde_reflection::Named<serde_reflection::Format>>,
    required: bool,
}

struct DataclassParameters {
    frozen: bool,
    serializable: bool,
    module_name: Option<String>,
    skip_fields: std::collections::HashSet<String>,
    name_to_new_name: std::collections::HashMap<String, String>,
    new_root_name: Option<String>,
}

fn generate_dataclasses<Writer: std::io::Write, Structure>(
    writer: &mut Writer,
    default_structure: &Structure,
    mut parameters: DataclassParameters,
) where
    Structure: serde::Serialize,
{
    let mut tracer = serde_reflection::Tracer::new(
        serde_reflection::TracerConfig::default().record_samples_for_structs(true),
    );
    let mut samples = serde_reflection::Samples::new();
    let root_name = match tracer
        .trace_value(&mut samples, default_structure)
        .unwrap()
        .0
    {
        serde_reflection::Format::TypeName(name) => name,
        _ => panic!("the root format is not a type name"),
    };
    if let Some(new_root_name) = parameters.new_root_name {
        parameters
            .name_to_new_name
            .insert(root_name.clone(), new_root_name);
    }
    let registry = tracer.registry().unwrap();
    let mut nodes: Vec<Node> = Vec::new();
    for (name, format) in registry.iter() {
        if nodes.iter().find(|node| node.name == *name).is_some() {
            panic!("multiple nodes have the same name \"{}\"", name);
        }
        let fields = match format {
            serde_reflection::ContainerFormat::UnitStruct => Vec::new(),
            serde_reflection::ContainerFormat::NewTypeStruct(format) => {
                vec![serde_reflection::Named {
                    name: "value".to_string(),
                    value: format.as_ref().clone(),
                }]
            }
            serde_reflection::ContainerFormat::TupleStruct(formats) => {
                vec![serde_reflection::Named {
                    name: "value".to_string(),
                    value: serde_reflection::Format::Tuple(formats.clone()),
                }]
            }
            serde_reflection::ContainerFormat::Struct(fields) => fields.clone(),
            serde_reflection::ContainerFormat::Enum(_) => {
                panic!("{} uses an unsupported root type", name);
            }
        };
        let mut children = std::collections::HashSet::new();
        for field in fields.iter() {
            if !parameters.skip_fields.contains(&field.name) {
                match &field.value {
                    serde_reflection::Format::TypeName(name) => {
                        children.insert(name.to_owned());
                    }
                    _ => (),
                }
            }
        }
        nodes.push(Node {
            name: name.clone(),
            children,
            fields,
            required: false,
        });
    }
    {
        let mut nodes_names_updated = std::collections::HashSet::new();
        let mut nodes_names_to_update = std::collections::HashSet::from([root_name.clone()]);
        while !nodes_names_to_update.is_empty() {
            let mut new_nodes_names_to_update = std::collections::HashSet::new();
            for node_name in nodes_names_to_update {
                let node = nodes
                    .iter_mut()
                    .find(|node| node.name == node_name)
                    .unwrap();
                node.required = true;
                new_nodes_names_to_update.extend(
                    node.children
                        .iter()
                        .filter(|child| !nodes_names_updated.contains(*child))
                        .map(|child| child.clone()),
                );
                nodes_names_updated.insert(node.name.clone());
            }
            nodes_names_to_update = new_nodes_names_to_update;
        }
    }
    {
        let mut generated_nodes = std::collections::HashSet::new();
        loop {
            let mut nodes_to_generate = Vec::new();
            for node in nodes.iter_mut() {
                if node.required
                    && !generated_nodes.contains(&node.name)
                    && node
                        .children
                        .iter()
                        .all(|child| generated_nodes.contains(child))
                {
                    nodes_to_generate.push(node);
                }
            }
            if nodes_to_generate.is_empty() {
                if nodes
                    .iter()
                    .any(|node| node.required && !generated_nodes.contains(&node.name))
                {
                    panic!("circular dependency in {root_name}");
                }
                break;
            } else {
                nodes_to_generate.sort_by(|a, b| a.name.cmp(&b.name));
                for node in nodes_to_generate {
                    // create a (leaked) &'static str for compatibility with samples.value.
                    let static_name = Box::leak(node.name.clone().into_boxed_str());
                    let values = match samples.value(static_name).unwrap() {
                        serde_reflection::Value::Seq(values) => values,
                        _ => panic!("{} is not a sequence or dictionary", node.name),
                    };
                    let name = parameters
                        .name_to_new_name
                        .get(&node.name)
                        .unwrap_or_else(|| &node.name)
                        .clone();
                    writeln!(
                        writer,
                        concat!("\n", "\n", "@dataclasses.dataclass{}\n", "class {}:",),
                        if parameters.frozen {
                            "(frozen=True)"
                        } else {
                            ""
                        },
                        name,
                    )
                    .unwrap();
                    for (index, field) in node.fields.iter().enumerate() {
                        if !parameters.skip_fields.contains(&field.name) {
                            writeln!(
                                writer,
                                "    {}: {} = {}",
                                field.name,
                                quote_type(&field.value, &parameters.name_to_new_name),
                                match &field.value {
                                    serde_reflection::Format::TypeName(name) =>
                                        format!("dataclasses.field(default_factory={name})"),
                                    _ => value_to_string(&values[index]),
                                },
                            )
                            .unwrap();
                        }
                    }
                    if parameters.serializable {
                        writeln!(
                            writer,
                            concat!(
                                "\n",
                                "    def serialize(self) -> bytes:\n",
                                "        return serde.bincode.serialize(self, {})",
                            ),
                            name
                        )
                        .unwrap();
                    }
                    if name == root_name {
                        if let Some(module_name) = parameters.module_name.as_ref() {
                            writeln!(
                                writer,
                                concat!(
                                    "\n",
                                    "    @staticmethod\n",
                                    "    def type() -> str:\n",
                                    "        return \"{}\"",
                                ),
                                module_name
                            )
                            .unwrap();
                        }
                    }
                    generated_nodes.insert(node.name.clone());
                }
            }
        }
    }
}

macro_rules! generate {
    ($($module:ident),+) => {
        let python_generated_directory = std::path::Path::new("python/neuromorphic_drivers/generated");
        let _ = std::fs::remove_dir_all(python_generated_directory);
        std::fs::create_dir(python_generated_directory).unwrap();
        let devices_directory = python_generated_directory.join("devices");
        std::fs::create_dir(&devices_directory).unwrap();
        paste::paste! {
            let mut writer = std::io::BufWriter::new(
                std::fs::File::create(python_generated_directory.join("enums.py")).unwrap(),
            );
            writeln!(writer, concat!(
                "import enum\n",
                "\n",
                "\n",
                "class Speed(enum.Enum):"
            )).unwrap();
            {
                let mut tracer = serde_reflection::Tracer::new(serde_reflection::TracerConfig::default());
                let (_, samples) = tracer.trace_simple_type::<neuromorphic_drivers::usb::Speed>().unwrap();
                let registry = tracer.registry().unwrap();
                for (name, format) in registry {
                    match format {
                        serde_reflection::ContainerFormat::Enum(variants) => {
                            for (variant, sample) in variants.iter().zip(samples.iter()) {
                                writeln!(
                                    writer,
                                    "    {} = \"{}\"",
                                    variant.1.name.to_uppercase(),
                                    sample.to_string(),
                                ).unwrap();
                            }
                        }
                        _ => {
                            panic!("{name} uses an unsupported root type");
                        }
                    }
                }
            }
            writeln!(writer, "\n\nclass Name(enum.Enum):").unwrap();
            $(
                writeln!(
                    writer,
                    "    {} = \"{}\"",
                    stringify!([<$module:upper>]),
                    neuromorphic_drivers::devices::$module::Device::PROPERTIES.name,
                ).unwrap();
            )+
        }
        $(
            paste::paste! {
                let mut writer = std::io::BufWriter::new(
                    std::fs::File::create(devices_directory.join(format!("{}.py", stringify!($module)))).unwrap(),
                );
                writeln!(writer, concat!(
                    "from __future__ import annotations\n",
                    "\n",
                    "import dataclasses\n",
                    "import types\n",
                    "import typing\n",
                    "\n",
                    "import numpy\n",
                    "\n",
                    "from ... import serde\n",
                    "from ... import status\n",
                    "from .. import enums",
                )).unwrap();
                generate_dataclasses(
                    &mut writer,
                    &neuromorphic_drivers::devices::$module::Device::PROPERTIES.default_configuration,
                    DataclassParameters {
                        frozen: false,
                        serializable: true,
                        module_name: Some(stringify!($module).to_owned()),
                        skip_fields: std::collections::HashSet::new(),
                        name_to_new_name: std::collections::HashMap::new(),
                        new_root_name: None,
                    },
                );
                generate_dataclasses(
                    &mut writer,
                    &neuromorphic_drivers::devices::$module::Device::DEFAULT_USB_CONFIGURATION,
                    DataclassParameters {
                        frozen: false,
                        serializable: true,
                        module_name: None,
                        skip_fields: std::collections::HashSet::new(),
                        name_to_new_name: std::collections::HashMap::new(),
                        new_root_name: Some("UsbConfiguration".to_owned()),
                    },
                );
                generate_dataclasses(
                    &mut writer,
                    &neuromorphic_drivers::devices::$module::Device::PROPERTIES,
                    DataclassParameters {
                        frozen: true,
                        serializable: false,
                        module_name: None,
                        skip_fields: std::collections::HashSet::from([
                            "name".to_owned(),
                            "default_configuration".to_owned(),
                        ]),
                        name_to_new_name: std::collections::HashMap::new(),
                        new_root_name: Some("Properties".into()),
                    },
                );
                for (class_name, iter_data_right) in [
                    ("Device", "dict[str, numpy.ndarray[typing.Any, numpy.dtype[numpy.void]]]"),
                    ("DeviceRaw", "bytes"),
                ] {
                    for (class_suffix, iter_data_left, iter_data_right_prefix, iter_data_right_suffix) in [
                        ("", "status.StatusNonOptional", "", ""),
                        ("Optional", "status.Status", "typing.Optional[", "]"),
                    ] {
                        writeln!(
                            writer,
                            concat!(
                                "\n",
                                "\n",
                                "class {}{}(typing.Protocol):\n",
                                "    def __enter__(self) -> \"{}{}\":\n",
                                "        ...\n",
                                "\n",
                                "    def __exit__(\n",
                                "        self,\n",
                                "        exception_type: typing.Optional[typing.Type[BaseException]],\n",
                                "        value: typing.Optional[BaseException],\n",
                                "        traceback: typing.Optional[types.TracebackType],\n",
                                "    ) -> bool:\n",
                                "        ...\n",
                                "\n",
                                "    def __iter__(self) -> \"{}{}\":\n",
                                "        ...\n",
                                "\n",
                                "    def __next__(self) -> tuple[{}, {}{}{}]:\n",
                                "        ...\n",
                                "\n",
                                "    def clear_backlog(self, until: int):\n",
                                "        ...\n",
                                "\n",
                                "    def name(self) -> typing.Literal[enums.Name.{}]:\n",
                                "        ...\n",
                                "\n",
                                "    def properties(self) -> Properties:\n",
                                "        ...\n",
                                "\n",
                                "    def serial(self) -> str:\n",
                                "        ...\n",
                                "\n",
                                "    def speed(self) -> enums.Speed:\n",
                                "        ...",
                            ),
                            class_name,
                            class_suffix,
                            class_name,
                            class_suffix,
                            class_name,
                            class_suffix,
                            iter_data_left,
                            iter_data_right_prefix,
                            iter_data_right,
                            iter_data_right_suffix,
                            stringify!([<$module:upper>]),
                        ).unwrap();
                    }
                }
            }
        )+
        paste::paste! {
            let mut writer = std::io::BufWriter::new(
                std::fs::File::create(python_generated_directory.join("unions.py")).unwrap(),
            );
            writeln!(
                writer,
                concat!(
                    "import typing\n",
                    "\n",
                    "from . import enums",
                ),
            ).unwrap();
            $(
                writeln!(writer,  "from .devices import {}", stringify!($module)).unwrap();
            )+
            writeln!(writer, "\n\nProperties = typing.Union[").unwrap();
            $(
                writeln!(writer,  "    {}.Properties,", stringify!($module)).unwrap();
            )+
            writeln!(writer, "]").unwrap();
            writeln!(writer, "\nConfiguration = typing.Union[").unwrap();
            $(
                writeln!(writer,  "    {}.Configuration,", stringify!($module)).unwrap();
            )+
            writeln!(writer, "]").unwrap();
            writeln!(writer, "\nUsbConfiguration = typing.Union[").unwrap();
            $(
                writeln!(writer,  "    {}.UsbConfiguration,", stringify!($module)).unwrap();
            )+
            writeln!(writer, "]").unwrap();
            writeln!(
                writer,
                concat!(
                    "\n",
                    "\n",
                    "def name_to_properties(name: enums.Name) -> Properties:",
                ),
            ).unwrap();
            $(
                writeln!(
                    writer,
                    concat!(
                        "    if name == enums.Name.{}:\n",
                        "        return {}.Properties()",
                    ),
                    stringify!([<$module:upper>]),
                    stringify!($module),
                ).unwrap();
            )+
            writeln!(
                writer,
                "    raise Exception(f\"unknown name {{name}}\")",
            ).unwrap();
        }
        {
            let mut writer = std::io::BufWriter::new(
                std::fs::File::create(python_generated_directory.join("devices_types.py")).unwrap(),
            );
            writeln!(
                writer,
                concat!(
                    "from __future__ import annotations\n",
                    "\n",
                    "import types\n",
                    "import typing\n",
                    "\n",
                    "import numpy\n",
                    "\n",
                    "from .. import device\n",
                    "from .. import status",
                ),
            ).unwrap();
            $(
                writeln!(writer,  "from .devices import {} as {}", stringify!($module), stringify!($module)).unwrap();
            )+
            writeln!(
                writer,
                concat!(
                    "from .enums import *\n",
                    "from .unions import *\n",
                ),
            ).unwrap();
            for (class_name, iter_data_right) in [
                ("GenericDevice", "dict[str, numpy.ndarray[typing.Any, numpy.dtype[numpy.void]]]"),
                ("GenericDeviceRaw", "bytes"),
            ] {
                for (class_suffix, iter_data_left, iter_data_right_prefix, iter_data_right_suffix) in [
                    ("", "status.StatusNonOptional", "", ""),
                    ("Optional", "status.Status", "typing.Optional[", "]"),
                ] {
                    writeln!(
                        writer,
                        concat!(
                            "\n",
                            "\n",
                            "class {}{}(typing.Protocol):\n",
                            "    def __enter__(self) -> \"{}{}\":\n",
                            "        ...\n",
                            "\n",
                            "    def __exit__(\n",
                            "        self,\n",
                            "        exception_type: typing.Optional[typing.Type[BaseException]],\n",
                            "        value: typing.Optional[BaseException],\n",
                            "        traceback: typing.Optional[types.TracebackType],\n",
                            "    ) -> bool:\n",
                            "        ...\n",
                            "\n",
                            "    def __iter__(self) -> \"{}{}\":\n",
                            "        ...\n",
                            "\n",
                            "    def __next__(self) -> tuple[{}, {}{}{}]:\n",
                            "        ...\n",
                            "\n",
                            "    def clear_backlog(self, until: int):\n",
                            "        ...\n",
                            "\n",
                            "    def name(self) -> Name:\n",
                            "        ...\n",
                            "\n",
                            "    def properties(self) -> Properties:\n",
                            "        ...\n",
                            "\n",
                            "    def serial(self) -> str:\n",
                            "        ...\n",
                            "\n",
                            "    def speed(self) -> Speed:\n",
                            "        ...",
                        ),
                        class_name,
                        class_suffix,
                        class_name,
                        class_suffix,
                        class_name,
                        class_suffix,
                        iter_data_left,
                        iter_data_right_prefix,
                        iter_data_right,
                        iter_data_right_suffix,
                    ).unwrap();
                }
            }
            $(
                for (class_name, raw) in [
                    ("Device", "False"),
                    ("DeviceRaw", "True"),
                ] {
                    for (class_suffix, iterator_timeout) in [
                        ("", "typing.Literal[None]"),
                        ("Optional", "typing.Optional[float]"),
                    ] {
                        writeln!(
                            writer,
                            concat!(
                                "\n",
                                "\n",
                                "@typing.overload\n",
                                "def open(\n",
                                "    configuration: {}.Configuration,\n",
                                "    iterator_timeout: {} = None,\n",
                                "    raw: typing.Literal[{}] = {},\n",
                                "    serial: typing.Optional[str] = None,\n",
                                "    usb_configuration: typing.Optional[UsbConfiguration] = None,\n",
                                ") -> {}.{}{}:\n",
                                "    ...",
                            ),
                            stringify!($module),
                            iterator_timeout,
                            raw,
                            raw,
                            stringify!($module),
                            class_name,
                            class_suffix,
                        ).unwrap();
                    }
                }
            )+
            for (class_name, raw) in [
                ("GenericDevice", "False"),
                ("GenericDeviceRaw", "True"),
            ] {
                for (class_suffix, iterator_timeout) in [
                    ("", "typing.Literal[None]"),
                    ("Optional", "typing.Optional[float]"),
                ] {
                    writeln!(
                        writer,
                        concat!(
                            "\n",
                            "\n",
                            "@typing.overload\n",
                            "def open(\n",
                            "    configuration: typing.Optional[Configuration] = None,\n",
                            "    iterator_timeout: {} = None,\n",
                            "    raw: typing.Literal[{}] = {},\n",
                            "    serial: typing.Optional[str] = None,\n",
                            "    usb_configuration: typing.Optional[UsbConfiguration] = None,\n",
                            ") -> {}{}:\n",
                            "    ...",
                        ),
                        iterator_timeout,
                        raw,
                        raw,
                        class_name,
                        class_suffix,
                    ).unwrap();
                }
            }
            writeln!(
                writer,
                concat!(
                    "\n",
                    "\n",
                    "def open(\n",
                    "    configuration: typing.Optional[Configuration] = None,\n",
                    "    iterator_timeout: typing.Optional[float] = None,\n",
                    "    raw: bool = False,\n",
                    "    serial: typing.Optional[str] = None,\n",
                    "    usb_configuration: typing.Optional[UsbConfiguration] = None,\n",
                    ") -> typing.Any:\n",
                    "    return device.Device.__new__(\n",
                    "        device.Device,\n",
                    "        raw,\n",
                    "        None\n",
                    "        if configuration is None\n",
                    "        else (configuration.type(), configuration.serialize()),\n",
                    "        serial,\n",
                    "        None if usb_configuration is None else usb_configuration.serialize(),\n",
                    "        iterator_timeout,\n",
                    "    )",
                ),
            ).unwrap();
        }
    };
}

fn macos_link_search_path() -> Option<String> {
    let output = cc::Build::new()
        .get_compiler()
        .to_command()
        .arg("--print-search-dirs")
        .output()
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("libraries: =") {
                let path = line.split('=').nth(1).unwrap();
                if !path.is_empty() {
                    return Some(format!("{}/lib/darwin", path));
                }
            }
        }
    }
    None
}

fn main() {
    generate!(prophesee_evk3_hd, prophesee_evk4);
    if std::env::var("TARGET").unwrap().contains("apple") {
        if let Some(path) = macos_link_search_path() {
            println!("cargo:rustc-link-lib=clang_rt.osx");
            println!("cargo:rustc-link-search={}", path);
        }
    }
}
