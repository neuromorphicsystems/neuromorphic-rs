import neuromorphic_drivers as nd
import numpy as np

nd.print_device_list()

devices: list[nd.GenericDevice] = []
for listed_device in nd.list_devices():
    device = nd.open(serial=listed_device.serial)
    devices.append(device)
    print(listed_device.serial, devices[0].serial(), devices[0].properties())

backlogs = np.array([0 for _ in devices])
while True:
    index = np.argmax(backlogs)
    status, packet = devices[index].__next__()
    backlog = status.ring.backlog()
    print(f"{index}: {round(status.delay() * 1e6)} µs, backlog: {backlog}")
    backlogs[:] += 1
    backlogs[index] = backlog
