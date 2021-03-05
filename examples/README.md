[1]: https://github.com/Sawchord/usbip-device

# Examples

This is a collection of examples to showcase the features of this crate.

## Setup

The examples use the [`usbip-device`][1] to simulate a USB device.
USBIP only works on linux for the moment.

On Ubuntu, install USBIP via:

```bash
sudo apt-get install linux-tools-generic
```

## Running

First, start the example:

```bash
cargo run --example <example_name>
```

then in another console run:

```bash
usbip attach -r "localhost" -b "1-1"
```

### Note
It might be necessary to run the latter command as root,
depending on your setup.