from __future__ import annotations

import builtins
import dataclasses
import pathlib
import struct
import sys
import typing
import zlib

import numpy

from . import serde as serde
from .generated.devices_types import *
from .neuromorphic_drivers import list_devices as extension_list_devices


@dataclasses.dataclass
class ListedDevice:
    name: Name
    speed: Speed
    serial: typing.Optional[str]
    error: typing.Optional[str]


def list_devices() -> list[ListedDevice]:
    return [
        ListedDevice(
            name=Name(name),
            speed=Speed(speed),
            serial=serial,
            error=error,
        )
        for (name, speed, serial, error) in extension_list_devices()
    ]


def print_device_list(color: bool = True):
    table = [["Name", "Speed", "Serial", "Error"]]
    for device in list_devices():
        table.append(
            [
                device.name.value,
                device.speed.value,
                "-" if device.serial is None else device.serial,
                "-" if device.error is None else device.error,
            ]
        )
    widths = [
        max(len(row[column]) for row in table) for column in range(0, len(table[0]))
    ]
    title = lambda string: f"\033[36;1m{string}\033[0m" if color else string
    sys.stdout.write(
        "".join(
            (
                "┌",
                "┬".join("─" * (width + 2) for width in widths),
                "┐\n",
                "".join(
                    (
                        "│",
                        "│".join(
                            title(f" {column:<{width}} ")
                            for width, column in zip(widths, table[0])
                        ),
                        "│",
                    )
                ),
                *(
                    (
                        "\n├",
                        "┼".join("─" * (width + 2) for width in widths),
                        "┤\n",
                        "\n".join(
                            "".join(
                                (
                                    "│",
                                    "│".join(
                                        f" {column:<{width}} "
                                        for width, column in zip(widths, row)
                                    ),
                                    "│",
                                )
                            )
                            for row in table[1:]
                        ),
                    )
                    if len(table) > 1
                    else ()
                ),
                "\n└",
                "┴".join("─" * (width + 2) for width in widths),
                "┘\n",
            )
        )
    )


class Mask:
    def __init__(self, width: serde.type.uint16, height: serde.type.uint16, fill: bool):
        self.x_unpacked = numpy.full(width, fill_value=fill, dtype=bool)
        self.y_unpacked = numpy.full(height, fill_value=fill, dtype=bool)

    def clear_x_range(
        self,
        start: serde.type.uint16,
        stop: serde.type.uint16,
        step: serde.type.uint16,
    ):
        for index in range(start, stop, step):
            self.x_unpacked[index] = False

    def fill_x_range(
        self,
        start: serde.type.uint16,
        stop: serde.type.uint16,
        step: serde.type.uint16,
    ):
        for index in range(start, stop, step):
            self.x_unpacked[index] = True

    def clear_y_range(
        self,
        start: serde.type.uint16,
        stop: serde.type.uint16,
        step: serde.type.uint16,
    ):
        for index in range(start, stop, step):
            self.y_unpacked[index] = False

    def fill_y_range(
        self,
        start: serde.type.uint16,
        stop: serde.type.uint16,
        step: serde.type.uint16,
    ):
        for index in range(start, stop, step):
            self.x_unpacked[index] = True

    def clear_rectangle(
        self,
        x: serde.type.uint16,
        y: serde.type.uint16,
        width: serde.type.uint16,
        height: serde.type.uint16,
    ):
        self.clear_x_range(start=x, stop=x + width, step=1)
        self.clear_y_range(start=y, stop=y + height, step=1)

    def x(self) -> tuple[serde.type.uint64, ...]:
        uint8 = numpy.packbits(self.x_unpacked, bitorder="little")
        if len(uint8) % 8 != 0:
            uint8 = numpy.append(
                uint8, numpy.zeros(8 - len(uint8) % 8, dtype=numpy.uint8)
            )
        return tuple(uint8.view(dtype="=u8").tolist())

    def y(self) -> tuple[serde.type.uint64, ...]:
        uint8 = numpy.packbits(self.y_unpacked, bitorder="little")
        if len(uint8) % 8 != 0:
            uint8 = numpy.append(
                uint8, numpy.zeros(8 - len(uint8) % 8, dtype=numpy.uint8)
            )
        return tuple(uint8.view(dtype="=u8").tolist())

    def pixels(self) -> numpy.ndarray[typing.Any, numpy.dtype[numpy.bool_]]:
        result = numpy.full(
            (len(self.y_unpacked), len(self.x_unpacked)), fill_value=False, dtype=bool
        )
        result[:, self.x_unpacked] = True
        result[numpy.flip(self.y_unpacked), :] = True
        return numpy.bitwise_not(result)

    def write_pixels_to_png_file(self, path: typing.Union[str, bytes, pathlib.Path]):
        def pack(tag: bytes, data: bytes):
            content = tag + data
            return b"".join(
                (
                    struct.pack(">I", len(data)),
                    content,
                    struct.pack(">I", 0xFFFFFFFF & zlib.crc32(content)),
                )
            )

        pixels = self.pixels()
        with builtins.open(path, "wb") as file:
            file.write(
                b"".join(
                    [
                        b"\x89PNG\r\n\x1a\n",
                        pack(
                            b"IHDR",
                            struct.pack(
                                ">IIBBBBB",
                                pixels.shape[1],
                                pixels.shape[0],
                                1,
                                0,
                                0,
                                0,
                                0,
                            ),
                        ),
                        pack(
                            b"IDAT",
                            zlib.compress(
                                numpy.hstack(
                                    (
                                        numpy.zeros(
                                            (pixels.shape[0], 1), dtype=numpy.uint8
                                        ),
                                        numpy.packbits(pixels).reshape(
                                            (pixels.shape[0], pixels.shape[1] // 8)
                                        ),
                                    )
                                ).tobytes(),
                                9,
                            ),
                        ),
                        pack(b"IEND", b""),
                    ]
                )
            )
