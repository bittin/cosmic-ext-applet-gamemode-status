// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use cosmic::app::{Command, Core};
use cosmic::cosmic_theme::Layer;
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::wayland::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::{subscription, Alignment, Length, Subscription};
use cosmic::iced_style::application;
use cosmic::widget::{layer_container, Column, Grid, JustifyContent, Text};
use cosmic::{Application, Element, Theme};

use crate::dbus::GameModeProxy;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, UpdateKind};
use zbus::Connection;

use crate::fl;

#[derive(Default)]
pub struct GameModeStatus {
    core: Core,
    sys: System,
    games: HashMap<i32, String>,
    popup: Option<Id>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    GameListUpdate,
    GameListSet(Vec<i32>),
}

impl Application for GameModeStatus {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "dev.DBrox.CosmicGameModeStatus";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let sys = System::new_with_specifics(
            RefreshKind::new()
                .with_processes(ProcessRefreshKind::new().with_exe(UpdateKind::OnlyIfNotSet)),
        );
        let app = GameModeStatus {
            core,
            sys,
            ..Default::default()
        };

        (app, Self::game_list_command())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<Self::Message> {
        self.core
            .applet
            .icon_button("applications-games-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        self.core
            .applet
            .popup_container(
                Column::new()
                    .align_items(Alignment::Center)
                    .push(
                        Text::new(if self.games.is_empty() {
                            fl!("gamemode-off")
                        } else {
                            fl!("gamemode-on")
                        })
                        .horizontal_alignment(Horizontal::Center),
                    )
                    .push(
                        layer_container(if self.games.is_empty() {
                            Text::new(fl!("no-active-clients"))
                                .horizontal_alignment(Horizontal::Center)
                                .into()
                        } else {
                            self.game_grid()
                        })
                        .layer(Layer::Primary)
                        .padding(10),
                    )
                    .padding(10)
                    .spacing(5),
            )
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let popup_settings =
                        self.core
                            .applet
                            .get_popup_settings(Id::MAIN, new_id, None, None, None);
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::GameListUpdate => (),
            Message::GameListSet(list) => {
                self.games = HashMap::new();
                self.sys.refresh_processes(ProcessesToUpdate::Some(
                    &list
                        .iter()
                        .map(|pid| Pid::from(*pid as usize))
                        .collect::<Vec<_>>(),
                ));
                for pid in &list {
                    if let Some(process) = self.sys.process(Pid::from(*pid as usize)) {
                        let exe = process
                            .exe()
                            .expect("failed to get path")
                            .file_name()
                            .expect("failed to get filename")
                            .to_str()
                            .unwrap()
                            .to_string();
                        self.games.insert(*pid, exe);
                    }
                }
                return Command::none();
            }
        }
        Self::game_list_command()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        struct GameMode;
        let registered = subscription::channel(
            std::any::TypeId::of::<GameMode>(),
            100,
            move |mut output| async move {
                let conn = Connection::session()
                    .await
                    .expect("Failled to start dbus session");
                let proxy = GameModeProxy::new(&conn)
                    .await
                    .expect("Failed to get proxy");
                let mut registered = proxy
                    .receive_game_registered()
                    .await
                    .expect("Failed to get GameRegistered signal");

                while let Some(_) = registered.next().await {
                    _ = output.send(Message::GameListUpdate).await;
                }
                panic!("Stream ended unexpectedly");
            },
        );
        let unregistered = subscription::channel(
            std::any::TypeId::of::<GameMode>(),
            100,
            move |mut output| async move {
                let conn = Connection::session()
                    .await
                    .expect("Failled to start dbus session");
                let proxy = GameModeProxy::new(&conn)
                    .await
                    .expect("Failed to get proxy");
                let mut unregistered = proxy
                    .receive_game_unregistered()
                    .await
                    .expect("Failed to get GameRegistered signal");

                while let Some(_) = unregistered.next().await {
                    _ = output.send(Message::GameListUpdate).await;
                }
                panic!("Stream ended unexpectedly");
            },
        );

        Subscription::batch(vec![registered, unregistered])
    }

    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
    }
}

impl GameModeStatus {
    fn game_grid(&self) -> Element<Message> {
        let mut grid = Grid::<Message>::new()
            .push(Text::new("PID"))
            .push(Text::new(fl!("name")));

        for (pid, name) in &self.games {
            grid = grid
                .insert_row()
                .push(Text::new(format!("{}", pid)))
                .push(Text::new(name));
        }

        grid.column_alignment(Alignment::Center)
            .row_alignment(Alignment::Center)
            .height(Length::Shrink)
            .width(Length::Shrink)
            .column_spacing(20)
            .justify_content(JustifyContent::SpaceEvenly)
            .into()
    }

    fn game_list_command() -> Command<Message> {
        Command::perform(
            async {
                let conn = Connection::session()
                    .await
                    .expect("Failed to start dbus session");
                let proxy = GameModeProxy::new(&conn)
                    .await
                    .expect("Failed to get proxy");
                let list = proxy.list_games().await.expect("Failed to get list");
                list.iter().map(|g| g.0).collect::<Vec<_>>()
            },
            |res| cosmic::app::Message::App(Message::GameListSet(res)),
        )
    }
}
