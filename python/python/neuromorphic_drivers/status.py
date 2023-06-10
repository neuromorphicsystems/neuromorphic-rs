import dataclasses
import typing


@dataclasses.dataclass
class PacketStatus:
    system_time: float
    read: int
    write_range: tuple[int, int]
    ring_length: int

    def backlog(self):
        return (self.write_range[0] + self.ring_length - 1 - self.read) % self.ring_length


@dataclasses.dataclass
class Status:
    system_time: float
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
