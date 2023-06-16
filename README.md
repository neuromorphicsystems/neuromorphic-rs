# neuromorphic_drivers

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

Notes on DMA:

-   Only available on Linux
-   The code allocates as many DMA buffers as possible (up to the requested number).
-   USB drivers usually have a limited number of DMA file objects (128 / 256 / 512 / 1024). More buffers are usually required to accomodate event bursts (we use 4096 buffers by default for the EVK4). The code fall backs to non-DMA buffers if all the DMA buffers are used (for instance the first 128 buffers would be DMA and the rest would be non-DMA).
-   Using all the available DMA buffers can cause other USB transfers to fail (including control transfers to configure the sensor).
-   Using DMA usually require one of the following workarounds:
    -   Copy DMA buffers to a larger ring before processing (this somewhat defeats the purpose of DMA in the first place)
    -   Use larger buffers (around 1 MB) to inccrease the ring size without increasing the number of buffers, at the cost of increased latency
    -   Ensure that processing can always keep up with the data rate (sparse scenes / low-sensitivity biases / event-rate limiter / simple processing). Note that simply expanding vectorized EVT3 events to 13-bytes DVS events is not real-time during peak bursts.

sudo udevadm control --reload-rules
sudo udevadm trigger
