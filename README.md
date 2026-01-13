# Sermo - A Cross-Platform Serial monitor

> [!WARNING]
> When developing against physical USB microcontrollers, some operating systems may require device drivers or udev rules for the OS to expose the device as a serial port.

## Development

Some USB microcontrollers require OS drivers or udev rules to be exposed as serial ports. If a device does not appear in your system or in the browser's Web Serial chooser, install the vendor driver (Windows/macOS) or add an appropriate udev rule on Linux, then reconnect the device.
```
packages/
├─ core/   # Core logic shared across platforms
├─ ui/     # Shared UI components
├─ desktop/
├─ mobile/
└─ web/
```

Each platform crate holds its platform-specific entrypoint and assets. To serve the web app locally:

```bash
cd packages/web
dx serve
```
