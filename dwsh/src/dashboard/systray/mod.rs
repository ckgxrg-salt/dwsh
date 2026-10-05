//! The system tray

mod item;

use gtk4::prelude::*;
use relm4::prelude::*;

use futures::StreamExt;
use std::sync::Arc;
use wayle_systray::core::item::TrayItem;

use crate::services::tray_service;
use item::SystrayItem;

pub struct Systray {
    items: FactoryVecDeque<SystrayItem>,
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
            set_spacing: 10,

            #[local_ref]
            items_widget -> gtk::Box {
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                set_spacing: 10,
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        watch_tray(&sender).await;

        let items = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        let model = Self { items };

        let items_widget = model.items.widget();
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
                let mut guard = self.items.guard();
                guard.clear();
                value.iter().for_each(|item| {
                    guard.push_back(item.clone());
                });
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
