//! Media controls from MPRIS.

use gtk4::{glib::Propagation, prelude::*};
use relm4::prelude::*;

use futures::StreamExt;
use std::{sync::Arc, time::Duration};
use tokio_util::sync::CancellationToken;
use wayle_core::watch_all;
use wayle_media::{core::player::Player, types::PlaybackState};

use crate::services::media_service;

static PAUSE_ICON_NAME: &str = "media-playback-pause-symbolic";
static PLAY_ICON_NAME: &str = "media-playback-start-symbolic";

pub struct Media {
    players: Vec<Arc<Player>>,
    active_player: Option<Arc<Player>>,
    cancellation_token: Option<CancellationToken>,

    // Dynamic
    cover_art: Option<String>,
    title: String,
    artist: String,
    state: PlaybackState,
    position: Duration,
    length: Option<Duration>,

    // Constant
    can_go_previous: bool,
    can_control: bool,
    can_go_next: bool,
}

#[derive(Debug)]
pub enum MediaMsg {
    NextPlayer,
    PrevPlayer,

    PlayPause,
    Next,
    Prev,
    Position(Duration),
}

#[derive(Debug)]
pub enum MediaCmd {
    PlayerList(Vec<Arc<Player>>),

    TrackInfo(Option<String>, String, String, Option<Duration>),
    Playback(PlaybackState),
    Position(Duration),
}

#[relm4::component(async, pub)]
impl AsyncComponent for Media {
    type Init = ();
    type Input = MediaMsg;
    type Output = ();
    type CommandOutput = MediaCmd;

    view! {
        gtk::Box {
            add_css_class: "panel",
            set_orientation: gtk4::Orientation::Vertical,
            set_size_request: (630, 360),
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,

            gtk::CenterBox {
                set_size_request: (630, 60),
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,

                #[wrap(Some)]
                set_start_widget = &gtk::Button {
                    set_tooltip_text: Some("Previous Player"),
                    #[watch]
                    set_visible: model.players.len() > 1,

                    connect_clicked => MediaMsg::PrevPlayer,
                },
                #[wrap(Some)]
                set_center_widget = &gtk::Box {
                    set_spacing: 20,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,

                    gtk::Image {
                        set_valign: gtk::Align::Center,
                        // TODO:
                        #[watch]
                        set_icon_name: Some("todo"),
                    }
                },
                #[wrap(Some)]
                set_end_widget = &gtk::Button {
                    set_tooltip_text: Some("Next Player"),
                    #[watch]
                    set_visible: model.players.len() > 1,

                    connect_clicked => MediaMsg::NextPlayer,
                },
            },

            gtk::Separator,

            if model.active_player.is_some() {
                gtk::Grid {
                    set_margin_top: 20,
                    set_margin_start: 10,
                    set_margin_end: 10,
                    set_row_spacing: 10,
                    set_column_spacing: 10,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,

                    attach[0, 0, 1, 2] = &gtk::Image {
                        #[watch]
                        set_from_file: model.cover_art.as_deref(),
                        set_valign: gtk::Align::Center,
                        set_pixel_size: 120,
                    },
                    attach[1, 0, 1, 1] = &gtk::Label {
                        set_max_width_chars: 20,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_valign: gtk::Align::Center,
                        #[watch]
                        set_label: &model.title,
                        #[watch]
                        set_tooltip_text: Some(&model.title),
                    },
                    attach[1, 1, 1, 1] = &gtk::Label {
                        set_max_width_chars: 20,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_valign: gtk::Align::Center,
                        #[watch]
                        set_label: &model.artist,
                        #[watch]
                        set_tooltip_text: Some(&model.artist),
                    },

                    attach[0, 2, 2, 1] = &gtk::Scale {
                        set_width_request: 610,

                        #[watch]
                        set_value: model.position.as_secs_f64(),
                        #[watch]
                        set_range: (0.0, model.length.unwrap_or_default().as_secs_f64()),

                        connect_change_value[sender] => move |_, _, value| {
                            sender.input(MediaMsg::Position(Duration::from_secs_f64(value)));
                            Propagation::Stop
                        }
                    },

                    attach[0, 3, 2, 1] = &gtk::CenterBox {
                        #[wrap(Some)]
                        set_start_widget = &gtk::Label {
                            #[watch]
                            set_visible: model.length.is_some(),
                            #[watch]
                            set_label: &length_str(model.position),
                        },

                        #[wrap(Some)]
                        set_center_widget = &gtk::Box {
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,

                            gtk::Button {
                                set_height_request: 30,
                                set_icon_name: "media-skip-backward-symbolic",

                                #[watch]
                                set_visible: model.can_go_previous,

                                connect_clicked => MediaMsg::Prev,
                            },
                            gtk::Button {
                                set_height_request: 30,

                                #[watch]
                                set_icon_name: playback_icon_name(model.state),
                                #[watch]
                                set_visible: model.can_control,

                                connect_clicked => MediaMsg::PlayPause,
                            },
                            gtk::Button {
                                set_height_request: 30,
                                set_icon_name: "media-skip-forward-symbolic",

                                #[watch]
                                set_visible: model.can_go_next,

                                connect_clicked => MediaMsg::Next,
                            }
                        },

                        #[wrap(Some)]
                        set_end_widget = &gtk::Label {
                            #[watch]
                            set_visible: model.length.is_some(),
                            #[watch]
                            set_label: &length_str(model.length.unwrap_or_default()),
                        },
                    },
                }
            } else {
                gtk::Label {
                    set_size_request: (420, 200),
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_label: "No Players Found",
                }
            }
        },
    }

    async fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        Self::run_service(&sender).await;

        let model = Self {
            players: Vec::new(),
            active_player: None,
            cancellation_token: None,

            cover_art: None,
            artist: String::default(),
            title: String::default(),
            state: PlaybackState::Stopped,
            position: Duration::default(),
            length: None,

            can_go_previous: false,
            can_control: false,
            can_go_next: false,
        };
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        if let Some(player) = &self.active_player {
            match message {
                MediaMsg::NextPlayer => {
                    if let Some(current_index) = self.players.iter().position(|p| *p == *player) {
                        let next_index = if current_index == self.players.len() - 1 {
                            0
                        } else {
                            current_index + 1
                        };
                        let next = self.players.get(next_index).cloned();
                        self.set_active_player(next, &sender);
                    }
                }
                MediaMsg::PrevPlayer => {
                    if let Some(current_index) = self.players.iter().position(|p| *p == *player) {
                        let next_index = if current_index == 0 {
                            self.players.len() - 1
                        } else {
                            current_index - 1
                        };
                        let next = self.players.get(next_index).cloned();
                        self.set_active_player(next, &sender);
                    }
                }
                MediaMsg::PlayPause => {
                    let _ = player.play_pause().await;
                }
                MediaMsg::Prev => {
                    let _ = player.previous().await;
                }
                MediaMsg::Next => {
                    let _ = player.next().await;
                }
                MediaMsg::Position(pos) => {
                    // [`player.set_position`] seems broken here, use seek for now.
                    // TODO: properly handle overflow
                    let new: i64 = pos.as_micros().try_into().unwrap();
                    let old: i64 = self.position.as_micros().try_into().unwrap();
                    let _ = player.seek(new - old).await;
                    self.position = pos;
                }
            }
        }
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            MediaCmd::PlayerList(list) => {
                if list.is_empty() {
                    self.set_active_player(None, &sender);
                } else {
                    if self.players.is_empty() {
                        self.set_active_player(list.first().cloned(), &sender);
                    }
                }
                self.players = list;
            }
            MediaCmd::TrackInfo(cover_art, title, artist, length) => {
                self.cover_art = cover_art;
                self.title = title;
                self.artist = artist;
                self.length = length;
                self.position = Duration::default();
            }
            MediaCmd::Playback(state) => {
                self.state = state;
            }
            MediaCmd::Position(position) => {
                self.position = position;
            }
        }
    }
}

impl Media {
    async fn run_service(sender: &AsyncComponentSender<Media>) {
        let service = media_service().await;

        let mut player_list = service.player_list.watch();
        sender.command(|out, shutdown| {
            shutdown
                .register(async move {
                    while let Some(list) = player_list.next().await {
                        let _ = out.send(MediaCmd::PlayerList(list));
                    }
                })
                .drop_on_shutdown()
        });
    }

    fn set_active_player(
        &mut self,
        player: Option<Arc<Player>>,
        sender: &AsyncComponentSender<Media>,
    ) {
        if let Some(token) = &self.cancellation_token {
            token.cancel();
        }
        if let Some(player) = &player {
            self.can_go_previous = player.can_go_previous.get();
            self.can_control = player.can_control.get();
            self.can_go_next = player.can_go_next.get();
        }
        self.active_player = player;
        self.run_active_player_service(sender);
    }

    /// Monitors changes of active player async.
    /// Does nothing if `self.active_player` is [`None`].
    fn run_active_player_service(&mut self, sender: &AsyncComponentSender<Media>) {
        if let Some(player) = &self.active_player {
            let token = CancellationToken::new();

            let _token = token.clone();
            let mut track_info = watch_all!(player.metadata, cover_art, title, artist, length);
            sender.command(|out, shutdown| async move {
                tokio::select! {
                    _ = _token.cancelled() => (),
                    _ = shutdown.register(async move {
                            while let Some(value) = track_info.next().await {
                                let mut cover_art = value.cover_art.get();
                                // Fallback, try to use `art_url` to work out a cover art.
                                if cover_art.is_none() {
                                    let art_url = value.art_url.get();
                                    cover_art = art_url.map(|url| url.strip_prefix("file://").unwrap_or_default().to_string());
                                }

                                let _ = out.send(MediaCmd::TrackInfo(cover_art, value.title.get(), value.artist.get(), value.length.get()));
                            }
                        })
                        .drop_on_shutdown() => (),
                }
            });

            let _token = token.clone();
            let mut playback = player.playback_state.watch();
            sender.command(|out, shutdown| async move {
                tokio::select! {
                    _ = _token.cancelled() => (),
                    _ = shutdown.register(async move {
                            while let Some(value) = playback.next().await {
                                let _ = out.send(MediaCmd::Playback(value));
                            }
                        })
                        .drop_on_shutdown() => (),
                }
            });

            let _token = token.clone();
            let mut position = player.position.watch();
            sender.command(|out, shutdown| async move {
                tokio::select! {
                    _ = _token.cancelled() => (),
                    _ = shutdown.register(async move {
                            while let Some(value) = position.next().await {
                                let _ = out.send(MediaCmd::Position(value));
                            }
                        })
                        .drop_on_shutdown() => (),
                }
            });

            self.cancellation_token = Some(token);
        }
    }
}

fn length_str(duration: Duration) -> String {
    format!(
        "{:02.0}:{:02.0}",
        duration.as_secs_f32() / 60.0,
        duration.as_secs_f32() % 60.0
    )
}

fn playback_icon_name(state: PlaybackState) -> &'static str {
    if state == PlaybackState::Playing {
        PAUSE_ICON_NAME
    } else {
        PLAY_ICON_NAME
    }
}
