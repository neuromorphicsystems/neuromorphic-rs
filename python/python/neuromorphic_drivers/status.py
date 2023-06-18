import dataclasses
import typing


@dataclasses.dataclass
class PacketStatus:
    system_time: float
    read: int
    write_range: tuple[int, int]
    ring_length: int
    # current_t is None if the mode is raw
    current_t: typing.Optional[int]

    def backlog(self):
        return (
            self.write_range[0] + self.ring_length - 1 - self.read
        ) % self.ring_length


@dataclasses.dataclass
class Status:
    system_time: float
    # packet is None if no data became available before iterator_timeout
    # packet may only be None if iterator_timeout is not None
    packet: typing.Optional[PacketStatus]

    def delay(self) -> typing.Optional[float]:
        if self.packet is None:
            return None
        return self.system_time - self.packet.system_time


@dataclasses.dataclass
class StatusNonOptional:
    system_time: float
    packet: PacketStatus

    def delay(self) -> float:
        ...
