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
    pr: serde.type.uint8 = 0x7C
    fo: serde.type.uint8 = 0x53
    hpf: serde.type.uint8 = 0x00
    diff_on: serde.type.uint8 = 0x66
    diff: serde.type.uint8 = 0x4D
    diff_off: serde.type.uint8 = 0x49
    inv: serde.type.uint8 = 0x5B
    refr: serde.type.uint8 = 0x14
    reqpuy: serde.type.uint8 = 0x8C
    reqpux: serde.type.uint8 = 0x7C
    sendreqpdy: serde.type.uint8 = 0x94
    unknown_1: serde.type.uint8 = 0x74
    unknown_2: serde.type.uint8 = 0x51

    def serialize(self) -> bytes:
        return serde.bincode.serialize(self, Biases)


class Clock(enum.Enum):
    INTERNAL = 0
    INTERNAL_WITH_OUTPUT_ENABLED = 1
    EXTERNAL = 2

    def serialize(self) -> bytes:
        return serde.bincode.serialize(self, Clock)


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
    pixel_mask: tuple[
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
        serde.type.uint64,
    ] = (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
    mask_intersection_only: bool = False
    enable_external_trigger: bool = True
    clock: Clock = Clock.INTERNAL
    rate_limiter: typing.Optional[RateLimiter] = None
    enable_output: bool = True

    def serialize(self) -> bytes:
        return serde.bincode.serialize(self, Configuration)

    @staticmethod
    def type() -> str:
        return "prophesee_evk4"


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

    def name(self) -> typing.Literal[enums.Name.PROPHESEE_EVK4]:
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

    def name(self) -> typing.Literal[enums.Name.PROPHESEE_EVK4]:
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

    def name(self) -> typing.Literal[enums.Name.PROPHESEE_EVK4]:
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

    def name(self) -> typing.Literal[enums.Name.PROPHESEE_EVK4]:
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
