import typing

from . import enums
from .. import serde
from .devices import prophesee_evk3_hd
from .devices import prophesee_evk4


Properties = typing.Union[
    prophesee_evk3_hd.Properties,
    prophesee_evk4.Properties,
]

Configuration = typing.Union[
    prophesee_evk3_hd.Configuration,
    prophesee_evk4.Configuration,
]

UsbConfiguration = typing.Union[
    prophesee_evk3_hd.UsbConfiguration,
    prophesee_evk4.UsbConfiguration,
]


def name_to_properties(name: enums.Name) -> Properties:
    if name == enums.Name.PROPHESEE_EVK3_HD:
        return prophesee_evk3_hd.Properties()
    if name == enums.Name.PROPHESEE_EVK4:
        return prophesee_evk4.Properties()
    raise Exception(f"unknown name {name}")


def deserialize_configuration(name: enums.Name, data: bytes) -> Configuration:
    if name == enums.Name.PROPHESEE_EVK3_HD:
        return serde.bincode.deserialize(data, prophesee_evk3_hd.Configuration)[0]
    if name == enums.Name.PROPHESEE_EVK4:
        return serde.bincode.deserialize(data, prophesee_evk4.Configuration)[0]
    raise Exception(f"unknown name {name}")
