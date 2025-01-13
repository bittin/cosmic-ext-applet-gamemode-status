// SPDX-License-Identifier: GPL-3.0-only

mod app;
mod dbus;
mod localization;

use app::GameModeStatus;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<GameModeStatus>(())
}
