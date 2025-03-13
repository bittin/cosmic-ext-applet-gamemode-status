// SPDX-License-Identifier: GPL-3.0-only

mod applet;
mod dbus;
mod localization;

use applet::GameModeStatus;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<GameModeStatus>(())
}
