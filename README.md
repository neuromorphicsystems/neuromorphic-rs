neuromorphic_drivers is a library to interact with USB Neuromorphic devices. The drivers were written from scratch with portability and performance in mind.

- [Supported devices and features](#supported-devices-and-features)
- [Python](#python)
    - [Get started](#get-started)
    - [Examples](#examples)
    - [Performance considerations](#performance-considerations)
    - [(Advanced) Direct Memory Access](#advanced-direct-memory-access)

# Supported devices and features

| Name              | Type   | Resolution | Data types   | Mask | Synchronize | Rate limiter |
| ----------------- | ------ | ---------- | ------------ | ---- | ----------- | ------------ |
| Prophesee EVK4    | Camera | 1280 × 720 | DVS, trigger | ✓    | ✓           | ✓            |
| Prophesee EVK3 HD | Camera | 1280 × 720 | DVS, trigger | ✓    | -           | ✓            |

This table lists fratures supported by this library. Certain sensors support unlisted features or features marked as "no" that have yet to be added to neuromorphic_drivers.

| Name              | Links                                       |
| ----------------- | ------------------------------------------- |
| Prophesee EVK4    | https://www.prophesee.ai/event-camera-evk4/ |
| Prophesee EVK3 HD | https://www.prophesee.ai/event-based-evk-3/ |

# Python

## Get started

`python3 -m pip install neuromorphic_drivers`

```py
import neuromorphic_drivers as nd

nd.print_device_list()

with nd.open() as device:
    for status, packet in device:
        # packet = {"dvs_events": np.array([...])}
```

`status` contains

## Examples

See _python/tests_ for different usage examples. `python/tests/display.py` implements a live event viewer with exponential decays caculated by the GPU. It requires vispy and glfw (`python3 -m pip install vispy glfw`).

## Performance considerations

Recent event-based cameras, such as the Prophesee EVK4, can generate more data than can be processed in real-time under certain circumstances. While the data can always be moved to the computer's memory in real-time, the simplest algorithms (including converting the raw USB bytes to a `{t, x, y, polarity}` event representation) struggle to keep up during data rate peaks. This library uses seperate threads for reading (USB to memory) and processing (memory to memory or disk) with a circular buffer (ring) at the interface. Short data bursts are seemlessly absorbed by the ring and are typically not an issue, even though they brifely cause a spike in latency. However, persistent high data rates cause the ring to slowly fill and cause the program to eventually crash. Depending on the use-case, one of the following work arounds can be applied:

-   reduce the camera's sensitivity (usually by tuning `diff_off` and `diff_on`)
-   enable the event rate limiter if the sensor supports it (the limiter randomly drops events on the sensor side, reducing bandwidth issues but significantly degrading the quality of transient bursts)
-   reduce the camera's spatial resolution by masking rows and columns
-   change the environment if possible (avoid flickering lights, reduce the optical flow, remove background clutter, keep large and fast objects out of the field of view)
-   call `device.clear_backlog(until=0)` whenever the `backlog` becomes too large (the maximum backlog is the size of the ring minus the size of the transfer queue)
-   use `nd.open(raw=true)` to skip the parser and directly access the USB bytes, typically to save them to a file

## (Advanced) Direct Memory Access

This library relies on libusb for all USB communications. libusb supports Direct Memory Access (Linux only for now, though other platforms may be added in the future), which allows the USB controller to directly write packets to memory without requiring CPU (and OS kernel) intervention. While this can increase performance and reduce CPU usage, DMA comes without caveats that we disabled it by default. Users when knoweldge of their USB controller and its limitations may want to enable it to increase performance on embedded systems.

USB drivers have a limited number of DMA file objects (128 / 256 / 512 / 1024). This is typically not enough to accomodate event bursts (we use 4096 buffers by default for the EVK4, with 131072 bytes per buffer). The code fall backs to non-DMA buffers if all the DMA buffers are used (for instance the first 128 buffers would be DMA and the rest would be non-DMA) and may result in variable performance over time.

Using all the available DMA buffers can cause other USB transfers to fail (including control transfers to configure the sensor).

Using DMA thus requires one of the following workarounds:

-   Use a small number of DMA buffers and copy packets to a larger ring before processing (this somewhat defeats the purpose of DMA and increases memory copies).
-   Use larger buffers (around 1 MB) to inccrease the ring size without increasing the number of buffers, at the cost of increased latency.
-   Ensure that processing can always keep up with the data rate (sparse scenes / low-sensitivity biases / event-rate limiter / simple processing). Note that simply expanding vectorized EVT3 events to 13-bytes DVS events is not real-time during data peaks.

```sh
cargo publish
```

```sh
cd python
python3 -m venv .venv
source .venv/bin/activate
pip install maturin
maturin develop
```

```sh
cd drivers
cargo test --release read -- --nocapture
```

/etc/udev/rules.d/65-neuromorphic-drivers.rules

```txt
SUBSYSTEM=="usb", ATTRS{idVendor}=="152a",ATTRS{idProduct}=="84[0-1]?", MODE="0666"
SUBSYSTEM=="usb", ATTRS{idVendor}=="04b4",ATTRS{idProduct}=="00f[4-5]", MODE="0666"
```

sudo udevadm control --reload-rules
sudo udevadm trigger
