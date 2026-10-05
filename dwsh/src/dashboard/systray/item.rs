//! A single tray item

use gtk4::prelude::*;
use relm4::prelude::*;

use futures::StreamExt;
use wayle_systray::{
    adapters::gtk4::Adapter,
    core::item::TrayItem,
    types::{Coordinates, menu::MenuItem},
};

use std::sync::Arc;

pub struct SystrayItem {
    item: Arc<TrayItem>,
    icon_name: Option<String>,
    menu_root: Option<MenuItem>,
    button: Option<gtk::Button>,
    popover: Option<gtk::PopoverMenu>,
}

#[derive(Debug)]
pub enum SystrayItemMsg {
    LeftClick,
    RightClick,
}

#[derive(Debug)]
pub enum SystrayItemCmd {
    UpdateMenu(Option<MenuItem>),
    UpdateIcon(Option<String>),
}

#[relm4::factory(pub)]
impl FactoryComponent for SystrayItem {
    type Init = Arc<TrayItem>;
    type Input = SystrayItemMsg;
    type Output = ();
    type CommandOutput = SystrayItemCmd;
    type ParentWidget = gtk::Box;

    view! {
        gtk::Button {
            set_size_request: (40, 40),
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,

            #[watch]
            set_icon_name: self.icon_name.as_deref().unwrap_or("missing-icon-name"),

            connect_clicked => SystrayItemMsg::LeftClick,
        },
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            item: init,
            icon_name: None,
            menu_root: None,
            button: None,
            popover: None,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let right_click = gtk::GestureClick::builder().button(3).build();
        right_click.connect_released({
            let sender = sender.clone();
            move |gesture, _, _, _| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                sender.input(SystrayItemMsg::RightClick);
            }
        });
        root.add_controller(right_click);

        self.button = Some(root.clone());

        self.watch_icon(&sender);
        self.watch_menu(&sender);

        let widgets = view_output!();
        widgets
    }

    // TODO: `tracing`
    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            SystrayItemMsg::LeftClick => {
                let item = self.item.clone();
                relm4::spawn_local(async move {
                    let _ = item.activate(Coordinates::new(0, 0)).await;
                });
            }
            SystrayItemMsg::RightClick => self.toggle_menu(&sender),
        }
    }

    fn update_cmd(&mut self, message: Self::CommandOutput, _sender: FactorySender<Self>) {
        match message {
            SystrayItemCmd::UpdateIcon(value) => self.icon_name = value,
            SystrayItemCmd::UpdateMenu(value) => self.menu_root = value,
        }
    }
}

impl SystrayItem {
    fn toggle_menu(&mut self, _sender: &FactorySender<SystrayItem>) {
        if let Some(popover) = self.popover.as_ref()
            && popover.is_visible()
        {
            popover.popdown();
            return;
        }

        let popover = Adapter::build_popover(&self.item);

        if let Some(button) = self.button.as_ref() {
            popover.set_parent(button);
        }

        popover.popup();
        self.popover = Some(popover);
    }

    fn watch_icon(&self, sender: &FactorySender<SystrayItem>) {
        let mut stream = self.item.icon_name.watch();

        sender.command(|out, shutdown| {
            shutdown
                .register(async move {
                    while let Some(value) = stream.next().await {
                        let _ = out.send(SystrayItemCmd::UpdateIcon(value));
                    }
                })
                .drop_on_shutdown()
        });
    }

    fn watch_menu(&self, sender: &FactorySender<SystrayItem>) {
        let mut stream = self.item.menu.watch();

        sender.command(|out, shutdown| {
            shutdown
                .register(async move {
                    while let Some(value) = stream.next().await {
                        let _ = out.send(SystrayItemCmd::UpdateMenu(value));
                    }
                })
                .drop_on_shutdown()
        });
    }
}
