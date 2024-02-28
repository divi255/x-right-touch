# x-right-touch

Simulates right button click on X11 touch screens with long touch gestures.

Initially created for my Panasonic Toughbook CF-19, theoretically can work with
any touch screen. Useful especially for non-Wacom screens which does not
provide gestures on Linux out-of-the-box.

## How to build

[Install Rust](https://www.rust-lang.org/tools/install)

Install required packages (for Debian/Ubuntu):

```
sudo apt-get -y install build-essential pkg-config \
    libudev-dev libx11-dev libxi-dev libxtst-dev libinput-dev
```

Build and install

```
make all
sudo make install
```

## Notes

* Requires root permissions to work with *libinput*, so suid attribute is set
  during installing.

* Has not been tested on Wayland.
