def install_udev_rules():
    import argparse
    import builtins
    import os
    import subprocess
    import sys

    parser = argparse.ArgumentParser(
        prog="neuromorphic-drivers-install-udev-rules",
        description="Install udev rules for Neuromorphic devices",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--output",
        "-o",
        default="/etc/udev/rules.d/65-neuromorphic-drivers.rules",
        help="Rules file path",
    )
    parser.add_argument(
        "--skip-system-check",
        "-s",
        action="store_true",
        help="Do not check the operating system type",
    )
    parser.add_argument(
        "--skip-root-check",
        "-r",
        action="store_true",
        help="Do not check for root privileges",
    )
    args = parser.parse_args()
    if not args.skip_system_check:
        if sys.platform == "win32":
            sys.stderr.write("udev rules are not compatible with Windows\n")
            sys.exit(1)
        if sys.platform == "darwin":
            sys.stderr.write("udev rules are not compatible with macOS\n")
            sys.exit(1)
    if not args.skip_root_check:
        if os.geteuid() != 0:
            sys.stderr.write(
                "this program requires root privileges (try running it with sudo)\n"
            )
            sys.exit(1)
    with builtins.open(args.output, "w") as output:
        output.write(
            "".join(
                (
                    'SUBSYSTEM=="usb", ATTRS{idVendor}=="152a",ATTRS{idProduct}=="84[0-1]?", MODE="0666"\n',
                    'SUBSYSTEM=="usb", ATTRS{idVendor}=="04b4",ATTRS{idProduct}=="00f[4-5]", MODE="0666"\n',
                )
            )
        )
    subprocess.check_call("udevadm control --reload-rules")
    subprocess.check_call("udevadm trigger")
