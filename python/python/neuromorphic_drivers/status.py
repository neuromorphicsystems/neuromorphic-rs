from __future__ import annotations

import dataclasses
import typing


@dataclasses.dataclass
class RingStatus:
    system_time: float
    """
    Seconds since the UNIX epoch (1970-01-01 00:00:00 UTC) measured when the USB packet was received.
    """

    read_range: tuple[int, int]
    write_range: tuple[int, int]
    ring_length: int
    # current_t is None if the mode is raw
    current_t: typing.Optional[int]

    def backlog(self) -> int:
        return (
            self.write_range[0] + self.ring_length - self.read_range[1]
        ) % self.ring_length

    def raw_packets(self) -> int:
        return (
            self.read_range[1] + self.ring_length - self.read_range[0]
        ) % self.ring_length


@dataclasses.dataclass
class Status:
    system_time: float
    """
    Seconds since the UNIX epoch (1970-01-01 00:00:00 UTC) measured when the user consumed the packet.

    Status.ring.system_time is always smaller than status.system_time.
    The difference between Status.ring.system_time and Status.system_time is a measurement
    of software latency (status.system_time - status.ring.system_time).
    """

    ring: typing.Optional[RingStatus]
    """
    ring is None if no data became available before iterator_timeout
    ring may only be None if iterator_timeout is not None
    """

    def delay(self) -> typing.Optional[float]:
        if self.ring is None:
            return None
        return self.system_time - self.ring.system_time


@dataclasses.dataclass
class StatusNonOptional:
    system_time: float
    ring: RingStatus

    def delay(self) -> float:
        ...
