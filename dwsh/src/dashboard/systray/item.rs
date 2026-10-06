//! A single tray item

use gtk4::gdk::{MemoryFormat, MemoryTexture};
use gtk4::glib::Bytes;
use gtk4::prelude::*;
use relm4::prelude::*;

use futures::StreamExt;
use wayle_systray::{
    adapters::gtk4::Adapter,
    core::item::TrayItem,
    types::{Coordinates, item::IconPixmap, menu::MenuItem},
};

use std::sync::Arc;

// Side length of a tray item button.
const TRAY_ITEM_SIZE: i32 = 100;

pub struct SystrayItem {
    item: Arc<TrayItem>,
    icon_name: Option<String>,
    icon_pixmap: Option<IconPixmap>,
    icon_theme_path: Option<String>,
    menu_root: Option<MenuItem>,
    button: Option<gtk::Button>,
    image: Option<gtk::Image>,
    popover: Option<gtk::PopoverMenu>,
}

#[derive(Debug)]
pub enum SystrayItemMsg {
    LeftClick,
    RightClick,
}

#[derive(Debug)]
pub enum SystrayItemCmd {
    Menu(Option<MenuItem>),
    Icon(Option<String>),
    IconPixmap(Vec<IconPixmap>),
    IconThemePath(Option<String>),
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
            set_size_request: (TRAY_ITEM_SIZE, TRAY_ITEM_SIZE),
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,

            connect_clicked => SystrayItemMsg::LeftClick,

            #[name(image)]
            gtk::Image {
                set_icon_name: Some("missing-icon-name"),
                set_pixel_size: 32,
            }
        },
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            item: init,
            icon_name: None,
            icon_pixmap: None,
            icon_theme_path: None,
            menu_root: None,
            button: None,
            image: None,
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
        self.watch_icon_pixmap(&sender);
        self.watch_icon_theme_path(&sender);
        self.watch_menu(&sender);

        let widgets = view_output!();
        self.image = Some(widgets.image.clone());
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
            SystrayItemCmd::Icon(value) => {
                self.icon_name = value;
                self.refresh_icon();
            }
            SystrayItemCmd::IconPixmap(value) => {
                self.icon_pixmap = value.into_iter().max_by_key(|p| p.width * p.height);
                self.refresh_icon();
            }
            SystrayItemCmd::IconThemePath(value) => {
                self.icon_theme_path = value;
                self.refresh_icon();
            }
            SystrayItemCmd::Menu(value) => self.menu_root = value,
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
                        let _ = out.send(SystrayItemCmd::Icon(value));
                    }
                })
                .drop_on_shutdown()
        });
    }

    fn watch_icon_pixmap(&self, sender: &FactorySender<SystrayItem>) {
        let mut stream = self.item.icon_pixmap.watch();

        sender.command(|out, shutdown| {
            shutdown
                .register(async move {
                    while let Some(value) = stream.next().await {
                        let _ = out.send(SystrayItemCmd::IconPixmap(value));
                    }
                })
                .drop_on_shutdown()
        });
    }

    fn watch_icon_theme_path(&self, sender: &FactorySender<SystrayItem>) {
        let mut stream = self.item.icon_theme_path.watch();

        sender.command(|out, shutdown| {
            shutdown
                .register(async move {
                    while let Some(value) = stream.next().await {
                        let _ = out.send(SystrayItemCmd::IconThemePath(value));
                    }
                })
                .drop_on_shutdown()
        });
    }

    /// Tray item's icon can come from various sources.
    fn refresh_icon(&self) {
        let Some(image) = self.image.as_ref() else {
            return;
        };

        if let Some(theme_path) = self.icon_theme_path.as_deref().filter(|p| !p.is_empty())
            && let Some(display) = gtk4::gdk::Display::default()
        {
            gtk4::IconTheme::for_display(&display).add_search_path(theme_path);
        }

        if let Some(name) = self.icon_name.as_deref().filter(|n| !n.is_empty()) {
            if std::path::Path::new(name).exists() {
                let file = gtk4::gio::File::for_path(name);
                image.set_from_gicon(&gtk4::gio::FileIcon::new(&file));
            } else {
                image.set_from_gicon(&gtk4::gio::ThemedIcon::new(name));
            }
        } else if let Some(pixmap) = &self.icon_pixmap {
            let bytes = Bytes::from(&pixmap.data);
            let texture = MemoryTexture::new(
                pixmap.width,
                pixmap.height,
                MemoryFormat::A8r8g8b8,
                &bytes,
                (pixmap.width * 4) as usize,
            );
            image.set_from_gicon(&texture);
        } else {
            image.set_icon_name(Some("missing-icon-name"));
        }
    }

    fn watch_menu(&self, sender: &FactorySender<SystrayItem>) {
        let mut stream = self.item.menu.watch();

        sender.command(|out, shutdown| {
            shutdown
                .register(async move {
                    while let Some(value) = stream.next().await {
                        let _ = out.send(SystrayItemCmd::Menu(value));
                    }
                })
                .drop_on_shutdown()
        });
    }
}
