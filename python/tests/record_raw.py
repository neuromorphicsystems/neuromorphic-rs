import datetime
import pathlib

import neuromorphic_drivers as nd

with nd.open(raw=True) as device:
    print(device.properties())
    filename = (
        pathlib.Path(__file__).resolve().parent
        / f"{datetime.datetime.now(tz=datetime.timezone.utc).isoformat().replace('+00:00', 'Z').replace(':', '-')}.raw"
    )
    print(f"writing to {filename}")
    total = 0
    with open(filename, "wb") as output:
        for status, packet in device:
            output.write(packet)
            output.flush()
            total += len(packet)
            print(
                f"{total / 1e6:.1f} MB (backlog: {status.ring.backlog()}, raw: {status.ring.raw_packets()})"
            )
