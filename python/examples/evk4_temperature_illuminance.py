import time

import neuromorphic_drivers as nd

nd.print_device_list()

with nd.open(nd.prophesee_evk4.Configuration(), raw=True) as device:
    print(device.serial(), device.properties())
    next = time.monotonic() + 1.0
    for status, packet in device:
        if time.monotonic() >= next:
            print(f"{device.temperature_celsius()}ºC")
            next += 1.0
