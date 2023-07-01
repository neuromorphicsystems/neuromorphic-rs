import dataclasses
import typing


@dataclasses.dataclass
class RingStatus:
    system_time: float
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
    # ring is None if no data became available before iterator_timeout
    # ring may only be None if iterator_timeout is not None
    ring: typing.Optional[RingStatus]

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
