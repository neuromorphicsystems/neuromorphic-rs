import pathlib
import struct
import typing
import zlib

import numpy

from . import serde


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
        with open(path, "wb") as file:
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
