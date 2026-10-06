//! The system tray

mod item;

use gtk4::prelude::*;
use relm4::prelude::*;

use futures::StreamExt;
use std::collections::HashSet;
use std::sync::Arc;
use wayle_systray::core::item::TrayItem;

use crate::services::tray_service;
use item::SystrayItem;

pub struct Systray {
    upper_items: FactoryVecDeque<SystrayItem>,
    lower_items: FactoryVecDeque<SystrayItem>,
    lower_widget: gtk::Box,
    separator: gtk::Separator,
}

#[derive(Debug)]
pub enum TrayCmd {
    UpdateItems(Vec<Arc<TrayItem>>),
}

#[relm4::component(async, pub)]
impl AsyncComponent for Systray {
    type Init = ();
    type Input = ();
    type Output = ();
    type CommandOutput = TrayCmd;

    view! {
        gtk::Box {
            add_css_class: "tray",
            add_css_class: "panel",
            set_size_request: (630, 360),
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            gtk::Box {
                set_vexpand: true,
            },

            #[local_ref]
            upper_widget -> gtk::Box {
                set_halign: gtk::Align::Start,
                set_valign: gtk::Align::Center,
                set_margin_start: 35,
                set_margin_end: 35,
                set_spacing: 10,
            },

            #[local_ref]
            separator -> gtk::Separator {
                set_visible: false,
            },

            #[local_ref]
            lower_widget -> gtk::Box {
                set_halign: gtk::Align::Start,
                set_valign: gtk::Align::Center,
                set_margin_start: 35,
                set_margin_end: 35,
                set_spacing: 10,
                set_visible: false,
            },

            gtk::Box {
                set_vexpand: true,
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        watch_tray(&sender).await;

        let upper_items: FactoryVecDeque<SystrayItem> = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();
        let lower_items: FactoryVecDeque<SystrayItem> = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        let lower_widget = lower_items.widget().clone();
        let separator = gtk::Separator::default();

        let model = Self {
            upper_items,
            lower_items,
            lower_widget,
            separator,
        };

        let upper_widget = model.upper_items.widget();
        let lower_widget = model.lower_items.widget();
        let separator = &model.separator;
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            TrayCmd::UpdateItems(value) => {
                // Some applications register under different names.
                // Deduplicate them by id manually.
                let mut seen = HashSet::new();
                let value: Vec<_> = value
                    .into_iter()
                    .filter(|item| seen.insert(item.id.get()))
                    .collect();

                {
                    let mut upper = self.upper_items.guard();
                    upper.clear();
                    for item in value.iter().take(5) {
                        upper.push_back(item.clone());
                    }
                }

                {
                    let mut lower = self.lower_items.guard();
                    lower.clear();
                    for item in value.iter().skip(5) {
                        lower.push_back(item.clone());
                    }
                }

                let show_lower = value.len() > 5;
                self.lower_widget.set_visible(show_lower);
                self.separator.set_visible(show_lower);
            }
        }
    }
}

async fn watch_tray(sender: &AsyncComponentSender<Systray>) {
    let service = tray_service().await;
    let mut stream = service.items.watch();

    sender.command(|out, shutdown| {
        shutdown
            .register(async move {
                while let Some(value) = stream.next().await {
                    let _ = out.send(TrayCmd::UpdateItems(value));
                }
            })
            .drop_on_shutdown()
    });
}
