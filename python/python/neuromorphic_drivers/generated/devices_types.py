from __future__ import annotations

import types
import typing

import numpy

from .. import device
from .. import status
from .devices import prophesee_evk3_hd as prophesee_evk3_hd
from .devices import prophesee_evk4 as prophesee_evk4
from .enums import *
from .unions import *



class GenericDevice(typing.Protocol):
    def __enter__(self) -> "GenericDevice":
        ...

    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool:
        ...

    def __iter__(self) -> "GenericDevice":
        ...

    def __next__(self) -> tuple[status.StatusNonOptional, dict[str, numpy.ndarray[typing.Any, numpy.dtype[numpy.void]]]]:
        ...

    def clear_backlog(self, until: int):
        ...

    def name(self) -> Name:
        ...

    def properties(self) -> Properties:
        ...

    def serial(self) -> str:
        ...

    def chip_firmware_configuration(self) -> Configuration:
        ...

    def speed(self) -> Speed:
        ...

    def update_configuration(self, configuration: Configuration):
        ...


class GenericDeviceOptional(typing.Protocol):
    def __enter__(self) -> "GenericDeviceOptional":
        ...

    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool:
        ...

    def __iter__(self) -> "GenericDeviceOptional":
        ...

    def __next__(self) -> tuple[status.Status, typing.Optional[dict[str, numpy.ndarray[typing.Any, numpy.dtype[numpy.void]]]]]:
        ...

    def clear_backlog(self, until: int):
        ...

    def name(self) -> Name:
        ...

    def properties(self) -> Properties:
        ...

    def serial(self) -> str:
        ...

    def chip_firmware_configuration(self) -> Configuration:
        ...

    def speed(self) -> Speed:
        ...

    def update_configuration(self, configuration: Configuration):
        ...


class GenericDeviceRaw(typing.Protocol):
    def __enter__(self) -> "GenericDeviceRaw":
        ...

    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool:
        ...

    def __iter__(self) -> "GenericDeviceRaw":
        ...

    def __next__(self) -> tuple[status.StatusNonOptional, bytes]:
        ...

    def clear_backlog(self, until: int):
        ...

    def name(self) -> Name:
        ...

    def properties(self) -> Properties:
        ...

    def serial(self) -> str:
        ...

    def chip_firmware_configuration(self) -> Configuration:
        ...

    def speed(self) -> Speed:
        ...

    def update_configuration(self, configuration: Configuration):
        ...


class GenericDeviceRawOptional(typing.Protocol):
    def __enter__(self) -> "GenericDeviceRawOptional":
        ...

    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool:
        ...

    def __iter__(self) -> "GenericDeviceRawOptional":
        ...

    def __next__(self) -> tuple[status.Status, typing.Optional[bytes]]:
        ...

    def clear_backlog(self, until: int):
        ...

    def name(self) -> Name:
        ...

    def properties(self) -> Properties:
        ...

    def serial(self) -> str:
        ...

    def chip_firmware_configuration(self) -> Configuration:
        ...

    def speed(self) -> Speed:
        ...

    def update_configuration(self, configuration: Configuration):
        ...


@typing.overload
def open(
    configuration: prophesee_evk3_hd.Configuration,
    iterator_timeout: typing.Literal[None] = None,
    raw: typing.Literal[False] = False,
    serial: typing.Optional[str] = None,
    usb_configuration: typing.Optional[UsbConfiguration] = None,
    iterator_maximum_raw_packets: int = 64,
) -> prophesee_evk3_hd.Device:
    ...


@typing.overload
def open(
    configuration: prophesee_evk3_hd.Configuration,
    iterator_timeout: typing.Optional[float] = None,
    raw: typing.Literal[False] = False,
    serial: typing.Optional[str] = None,
    usb_configuration: typing.Optional[UsbConfiguration] = None,
    iterator_maximum_raw_packets: int = 64,
) -> prophesee_evk3_hd.DeviceOptional:
    ...


@typing.overload
def open(
    configuration: prophesee_evk3_hd.Configuration,
    iterator_timeout: typing.Literal[None] = None,
    raw: typing.Literal[True] = True,
    serial: typing.Optional[str] = None,
    usb_configuration: typing.Optional[UsbConfiguration] = None,
    iterator_maximum_raw_packets: int = 64,
) -> prophesee_evk3_hd.DeviceRaw:
    ...


@typing.overload
def open(
    configuration: prophesee_evk3_hd.Configuration,
    iterator_timeout: typing.Optional[float] = None,
    raw: typing.Literal[True] = True,
    serial: typing.Optional[str] = None,
    usb_configuration: typing.Optional[UsbConfiguration] = None,
    iterator_maximum_raw_packets: int = 64,
) -> prophesee_evk3_hd.DeviceRawOptional:
    ...


@typing.overload
def open(
    configuration: prophesee_evk4.Configuration,
    iterator_timeout: typing.Literal[None] = None,
    raw: typing.Literal[False] = False,
    serial: typing.Optional[str] = None,
    usb_configuration: typing.Optional[UsbConfiguration] = None,
    iterator_maximum_raw_packets: int = 64,
) -> prophesee_evk4.Device:
    ...


@typing.overload
def open(
    configuration: prophesee_evk4.Configuration,
    iterator_timeout: typing.Optional[float] = None,
    raw: typing.Literal[False] = False,
    serial: typing.Optional[str] = None,
    usb_configuration: typing.Optional[UsbConfiguration] = None,
    iterator_maximum_raw_packets: int = 64,
) -> prophesee_evk4.DeviceOptional:
    ...


@typing.overload
def open(
    configuration: prophesee_evk4.Configuration,
    iterator_timeout: typing.Literal[None] = None,
    raw: typing.Literal[True] = True,
    serial: typing.Optional[str] = None,
    usb_configuration: typing.Optional[UsbConfiguration] = None,
    iterator_maximum_raw_packets: int = 64,
) -> prophesee_evk4.DeviceRaw:
    ...


@typing.overload
def open(
    configuration: prophesee_evk4.Configuration,
    iterator_timeout: typing.Optional[float] = None,
    raw: typing.Literal[True] = True,
    serial: typing.Optional[str] = None,
    usb_configuration: typing.Optional[UsbConfiguration] = None,
    iterator_maximum_raw_packets: int = 64,
) -> prophesee_evk4.DeviceRawOptional:
    ...


@typing.overload
def open(
    configuration: typing.Optional[Configuration] = None,
    iterator_timeout: typing.Literal[None] = None,
    raw: typing.Literal[False] = False,
    serial: typing.Optional[str] = None,
    usb_configuration: typing.Optional[UsbConfiguration] = None,
    iterator_maximum_raw_packets: int = 64,
) -> GenericDevice:
    ...


@typing.overload
def open(
    configuration: typing.Optional[Configuration] = None,
    iterator_timeout: typing.Optional[float] = None,
    raw: typing.Literal[False] = False,
    serial: typing.Optional[str] = None,
    usb_configuration: typing.Optional[UsbConfiguration] = None,
    iterator_maximum_raw_packets: int = 64,
) -> GenericDeviceOptional:
    ...


@typing.overload
def open(
    configuration: typing.Optional[Configuration] = None,
    iterator_timeout: typing.Literal[None] = None,
    raw: typing.Literal[True] = True,
    serial: typing.Optional[str] = None,
    usb_configuration: typing.Optional[UsbConfiguration] = None,
    iterator_maximum_raw_packets: int = 64,
) -> GenericDeviceRaw:
    ...


@typing.overload
def open(
    configuration: typing.Optional[Configuration] = None,
    iterator_timeout: typing.Optional[float] = None,
    raw: typing.Literal[True] = True,
    serial: typing.Optional[str] = None,
    usb_configuration: typing.Optional[UsbConfiguration] = None,
    iterator_maximum_raw_packets: int = 64,
) -> GenericDeviceRawOptional:
    ...


def open(
    configuration: typing.Optional[Configuration] = None,
    iterator_timeout: typing.Optional[float] = None,
    raw: bool = False,
    serial: typing.Optional[str] = None,
    usb_configuration: typing.Optional[UsbConfiguration] = None,
    iterator_maximum_raw_packets: int = 64,
) -> typing.Any:
    return device.Device.__new__(
        device.Device,
        raw,
        iterator_maximum_raw_packets,
        None
        if configuration is None
        else (configuration.type(), configuration.serialize()),
        serial,
        None if usb_configuration is None else usb_configuration.serialize(),
        iterator_timeout,
    )
