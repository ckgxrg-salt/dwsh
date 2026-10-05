//! Display time and calendar

use gtk4::prelude::*;
use relm4::prelude::*;

use chrono::{DateTime, Local};
use std::time::Duration;

pub struct Calendar {
    time: DateTime<Local>,
}

#[derive(Debug)]
pub enum CalendarCmd {
    UpdateTime(DateTime<Local>),
}

#[relm4::component(pub)]
impl Component for Calendar {
    type Init = ();
    type Input = ();
    type Output = ();
    type CommandOutput = CalendarCmd;

    view! {
        gtk::Box {
            add_css_class: "panel",
            set_size_request: (540, 360),

            gtk::Label {
                inline_css: "font-size: 96px;",
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                set_hexpand: true,
                set_margin_end: 25,

                #[watch]
                set_label: &model.time.format("%H").to_string(),
            },
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,

                gtk::Label {
                    inline_css: "font-size: 36px;",
                    set_halign: gtk::Align::Center,

                    #[watch]
                    set_label: &model.time.format("%a").to_string()
                },
                gtk::Label {
                    inline_css: "font-size: 24px;",
                    set_halign: gtk::Align::Center,

                    #[watch]
                    set_label: &model.time.format("%m/%d").to_string()
                },
            },
            gtk::Label {
                inline_css: "font-size: 96px;",
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                set_hexpand: true,
                set_margin_start: 25,

                #[watch]
                set_label: &model.time.format("%M").to_string()
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        watch_time(&sender);

        let model = Calendar { time: Local::now() };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            CalendarCmd::UpdateTime(now) => self.time = now,
        }
    }
}

fn watch_time(sender: &ComponentSender<Calendar>) {
    sender.command(|out, shutdown| {
        shutdown
            .register(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    let now = Local::now();
                    let _ = out.send(CalendarCmd::UpdateTime(now));
                }
            })
            .drop_on_shutdown()
    });
}
