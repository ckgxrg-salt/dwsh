//! Some toggles for services

use gtk4::{ApplicationInhibitFlags, prelude::*};
use relm4::prelude::*;

use tokio::process::Command;

pub struct Switches {
    inhibit: bool,
    inhibit_cookie: u32,
    mpd: bool,
    cava: bool,
}

#[derive(Debug)]
pub enum SwitchesMsg {
    Inhibit,
    Mpd,
    Cava,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for Switches {
    type Init = ();
    type Input = SwitchesMsg;
    type Output = ();

    view! {
        gtk::Grid {
            add_css_class: "switches",
            add_css_class: "panel",

            attach[0, 0, 1, 1] = &gtk::Button {
                set_size_request: (150, 150),

                #[watch]
                set_class_active: ("active", model.mpd),
                #[watch]
                set_tooltip_text: if model.mpd { Some("MPD is running") } else { Some("MPD is not running") },
                set_icon_name: "playlist-symbolic",
                connect_clicked => SwitchesMsg::Mpd,
            },
            attach[1, 0, 1, 1] = &gtk::Button {
                set_size_request: (150, 150),

                #[watch]
                set_class_active: ("active", model.inhibit),
                #[watch]
                set_tooltip_text: if model.inhibit { Some("Inhibited system idle") } else { Some("This button says Zzz, pretending to sleep") },
                set_icon_name: if model.inhibit { "caffeine-cup-full-symbolic" } else { "caffeine-cup-empty-symbolic" },
                connect_clicked => SwitchesMsg::Inhibit,
            },
            attach[0, 1, 1, 1] = &gtk::Button {
                set_size_request: (150, 150),

                #[watch]
                set_class_active: ("active", model.cava),
                #[watch]
                set_tooltip_text: if model.cava { Some("CAVA is running") } else { Some("CAVA is not running") },
                set_icon_name: "histogram-symbolic",
                connect_clicked => SwitchesMsg::Cava,
            },
        },
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            inhibit: false,
            inhibit_cookie: 0,
            mpd: get_state("mpd.service").await,
            cava: get_state("cava.service").await,
        };
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        match message {
            SwitchesMsg::Inhibit => {
                if self.inhibit {
                    relm4::main_application().uninhibit(self.inhibit_cookie);
                } else {
                    self.inhibit_cookie = relm4::main_application().inhibit(
                        None::<&gtk::Window>,
                        ApplicationInhibitFlags::IDLE,
                        Some("Inhibited by dwsh"),
                    );
                }
                self.inhibit = !self.inhibit;
            }
            SwitchesMsg::Mpd => {
                let current = get_state("mpd.service").await;
                if current {
                    let _ = Command::new("systemctl")
                        .args([
                            "--user",
                            "stop",
                            "mpd-mpris.service",
                            "mpd-notification.service",
                            "mpd.service",
                        ])
                        .status()
                        .await;
                } else {
                    let _ = Command::new("systemctl")
                        .args([
                            "--user",
                            "start",
                            "mpd.service",
                            "mpd-notification.service",
                            "mpd-mpris.service",
                        ])
                        .status()
                        .await;
                }
                self.mpd = get_state("mpd.service").await;
            }
            SwitchesMsg::Cava => {
                let current = get_state("cava.service").await;
                if current {
                    let _ = Command::new("systemctl")
                        .args(["--user", "stop", "cava.service"])
                        .status()
                        .await;
                } else {
                    let _ = Command::new("systemctl")
                        .args(["--user", "start", "cava.service"])
                        .status()
                        .await;
                }
                self.cava = get_state("cava.service").await;
            }
        }
    }
}

/// Ugly wrapper for `systemctl is-active`
async fn get_state(service: &str) -> bool {
    Command::new("systemctl")
        .args(["--user", "is-active", service])
        .stdout(std::process::Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}
