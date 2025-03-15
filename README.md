# GameMode Status Cosmic Applet



![screenshot of the applet](./res/screenshot1.png) ![screenshot of the applet](./res/screenshot2.png)

## Dependencies

- gamemode
- libsystemd0
- libxkbcommon-dev

Or equivalent packages in non-debian based distros.

## Install

Clone the repo and run the commands corresponding to your distro:

```sh
git clone https://github.com/D-Brox/cosmic-ext-applet-gamemode-status
cd cosmic-ext-applet-gamemode-status

# Debian based distros
just build-deb
sudo just install-deb

# RPM based distros
just build-rpm
sudo just install-rpm

# For other distros:
just build-release
# Global install (root)
sudo just install
# or local install (user)
just install-local
```

## Contributing

Contributions are welcome

To build and install the debug build

```sh
just build-debug && sudo just debug=1 install
```

## Special Thanks

- [gicmo](https://github.com/gicmo) for their [GNOME Shell system monitor extension](https://github.com/gicmo/gamemode-extension), the inspiration for this applet
- [edfloreshz](https://github.com/edfloreshz) for the [template for COSMIC applets](https://github.com/edfloreshz/cosmic-applet-template), which taught me the logic behind an applet
