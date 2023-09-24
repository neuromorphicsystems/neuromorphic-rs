from __future__ import annotations

import dataclasses
import enum
import types
import typing

import numpy

from ... import serde
from ... import status
from .. import enums


@dataclasses.dataclass
class Biases:
    pr: serde.type.uint8 = 0x69
    fo_p: serde.type.uint8 = 0x4A
    fo_n: serde.type.uint8 = 0x00
    hpf: serde.type.uint8 = 0x00
    diff_on: serde.type.uint8 = 0x73
    diff: serde.type.uint8 = 0x50
    diff_off: serde.type.uint8 = 0x34
    refr: serde.type.uint8 = 0x44
    reqpuy: serde.type.uint8 = 0x94
    blk: serde.type.uint8 = 0x78

    def serialize(self) -> bytes:
        return serde.bincode.serialize(self, Biases)


@dataclasses.dataclass
class RateLimiter:
    reference_period_us: serde.type.uint16
    maximum_events_per_period: serde.type.uint32

    def serialize(self) -> bytes:
        return serde.bincode.serialize(self, RateLimiter)


@dataclasses.dataclass
class Configuration:
    biases: Biases = dataclasses.field(default_factory=Biases)
    x_mask: tuple[
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
    ] = (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
    y_mask: tuple[
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
        serde.type.uint64,
    ] = (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
    mask_intersection_only: bool = False
    rate_limiter: typing.Optional[RateLimiter] = None

    def serialize(self) -> bytes:
        return serde.bincode.serialize(self, Configuration)

    @staticmethod
    def type() -> str:
        return "prophesee_evk3_hd"


@dataclasses.dataclass
class UsbConfiguration:
    buffer_size: serde.type.uint64 = 131072
    ring_size: serde.type.uint64 = 4096
    transfer_queue_size: serde.type.uint64 = 32
    allow_dma: bool = False

    def serialize(self) -> bytes:
        return serde.bincode.serialize(self, UsbConfiguration)


@dataclasses.dataclass(frozen=True)
class Properties:
    width: serde.type.uint16 = 1280
    height: serde.type.uint16 = 720


class Device(typing.Protocol):
    def __enter__(self) -> "Device":
        ...

    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool:
        ...

    def __iter__(self) -> "Device":
        ...

    def __next__(self) -> tuple[status.StatusNonOptional, dict[str, numpy.ndarray[typing.Any, numpy.dtype[numpy.void]]]]:
        ...

    def clear_backlog(self, until: int):
        ...

    def name(self) -> typing.Literal[enums.Name.PROPHESEE_EVK3_HD]:
        ...

    def properties(self) -> Properties:
        ...

    def serial(self) -> str:
        ...

    def chip_firmware_configuration(self) -> Configuration:
        ...

    def speed(self) -> enums.Speed:
        ...

    def update_configuration(self, configuration: Configuration):
        ...


class DeviceOptional(typing.Protocol):
    def __enter__(self) -> "DeviceOptional":
        ...

    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool:
        ...

    def __iter__(self) -> "DeviceOptional":
        ...

    def __next__(self) -> tuple[status.Status, typing.Optional[dict[str, numpy.ndarray[typing.Any, numpy.dtype[numpy.void]]]]]:
        ...

    def clear_backlog(self, until: int):
        ...

    def name(self) -> typing.Literal[enums.Name.PROPHESEE_EVK3_HD]:
        ...

    def properties(self) -> Properties:
        ...

    def serial(self) -> str:
        ...

    def chip_firmware_configuration(self) -> Configuration:
        ...

    def speed(self) -> enums.Speed:
        ...

    def update_configuration(self, configuration: Configuration):
        ...


class DeviceRaw(typing.Protocol):
    def __enter__(self) -> "DeviceRaw":
        ...

    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool:
        ...

    def __iter__(self) -> "DeviceRaw":
        ...

    def __next__(self) -> tuple[status.StatusNonOptional, bytes]:
        ...

    def clear_backlog(self, until: int):
        ...

    def name(self) -> typing.Literal[enums.Name.PROPHESEE_EVK3_HD]:
        ...

    def properties(self) -> Properties:
        ...

    def serial(self) -> str:
        ...

    def chip_firmware_configuration(self) -> Configuration:
        ...

    def speed(self) -> enums.Speed:
        ...

    def update_configuration(self, configuration: Configuration):
        ...


class DeviceRawOptional(typing.Protocol):
    def __enter__(self) -> "DeviceRawOptional":
        ...

    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool:
        ...

    def __iter__(self) -> "DeviceRawOptional":
        ...

    def __next__(self) -> tuple[status.Status, typing.Optional[bytes]]:
        ...

    def clear_backlog(self, until: int):
        ...

    def name(self) -> typing.Literal[enums.Name.PROPHESEE_EVK3_HD]:
        ...

    def properties(self) -> Properties:
        ...

    def serial(self) -> str:
        ...

    def chip_firmware_configuration(self) -> Configuration:
        ...

    def speed(self) -> enums.Speed:
        ...

    def update_configuration(self, configuration: Configuration):
        ...
