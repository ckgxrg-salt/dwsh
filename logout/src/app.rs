use gtk4::gdk::Key;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::prelude::*;
use std::fmt::Display;
use std::process::Command;

pub struct DwshLogout {
    text: String,
    focused: LogoutAction,
}

#[derive(PartialEq, Debug)]
pub enum LogoutAction {
    None,
    Poweroff,
    Reboot,
    Suspend,
    Logout,
    Lock,
}

impl Display for LogoutAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::None => write!(f, "Daywatch"),
            Self::Poweroff => write!(f, "Power off"),
            Self::Reboot => write!(f, "Reboot"),
            Self::Suspend => write!(f, "Suspend"),
            Self::Logout => write!(f, "Log out"),
            Self::Lock => write!(f, "Lock screen"),
        }
    }
}

#[derive(Debug)]
pub enum Message {
    SelectAction(LogoutAction),
}

#[relm4::component(pub)]
impl SimpleComponent for DwshLogout {
    type Init = ();
    type Input = Message;
    type Output = ();

    view! {
        gtk::Window {
            init_layer_shell: (),
            set_layer: Layer::Overlay,
            set_anchor: (Edge::Left, true),
            set_anchor: (Edge::Right, true),
            set_anchor: (Edge::Top, true),
            set_anchor: (Edge::Bottom, true),
            set_keyboard_mode: KeyboardMode::Exclusive,
            set_title: Some("dwsh-logout"),

            add_controller = gtk::GestureClick {
                connect_released[sender] => move |_, _, _, _| sender.input(Message::SelectAction(LogoutAction::None)),
            },
            add_controller = gtk::EventControllerKey {
                connect_key_released[sender] => move |_, key, _, _| if let Some(action) = identify_key(key) {
                    sender.input(Message::SelectAction(action));
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_align: gtk::Align::Center,
                set_spacing: 50,

                gtk::Box {
                    set_align: gtk::Align::Center,
                    set_height_request: 300,
                    set_spacing: 50,

                    gtk::Button {
                        set_tooltip_text: Some("Power Off"),
                        set_icon_name: "system-shutdown-symbolic",
                        set_size_request: (300, 350),
                        #[watch]
                        set_class_active: ("focused", model.focused == LogoutAction::Poweroff),
                        connect_clicked => Message::SelectAction(LogoutAction::Poweroff),
                    },
                    gtk::Button {
                        set_label: "Reboot",
                        set_icon_name: "system-reboot-symbolic",
                        set_size_request: (300, 350),
                        #[watch]
                        set_class_active: ("focused", model.focused == LogoutAction::Reboot),
                        connect_clicked => Message::SelectAction(LogoutAction::Reboot),
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,

                    gtk::Label {
                        #[watch]
                        set_label: &model.text,
                    },
                    gtk::Separator,
                },

                gtk::Box {
                    set_align: gtk::Align::Center,
                    set_height_request: 300,
                    set_spacing: 30,

                    gtk::Button {
                        set_label: "Suspend",
                        set_icon_name: "system-suspend-symbolic",
                        set_size_request: (300, 350),
                        #[watch]
                        set_class_active: ("focused", model.focused == LogoutAction::Suspend),
                        connect_clicked => Message::SelectAction(LogoutAction::Suspend),
                    },
                    gtk::Button {
                        set_label: "Logout",
                        set_icon_name: "system-log-out-symbolic",
                        set_size_request: (300, 350),
                        #[watch]
                        set_class_active: ("focused", model.focused == LogoutAction::Logout),
                        connect_clicked => Message::SelectAction(LogoutAction::Logout),
                    },
                    gtk::Button {
                        set_label: "Lock",
                        set_icon_name: "system-lock-screen-symbolic",
                        set_size_request: (300, 350),
                        #[watch]
                        set_class_active: ("focused", model.focused == LogoutAction::Lock),
                        connect_clicked => Message::SelectAction(LogoutAction::Lock),
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = DwshLogout {
            text: LogoutAction::None.to_string(),
            focused: LogoutAction::None,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Message::SelectAction(action) => {
                if self.focused == action {
                    execute(&action);
                } else {
                    self.text = action.to_string();
                    self.focused = action;
                }
            }
        }
    }
}

fn identify_key(key: Key) -> Option<LogoutAction> {
    match key {
        Key::Escape => Some(LogoutAction::None),
        Key::s => Some(LogoutAction::Poweroff),
        Key::r => Some(LogoutAction::Reboot),
        Key::l => Some(LogoutAction::Lock),
        Key::e => Some(LogoutAction::Logout),
        Key::u => Some(LogoutAction::Suspend),
        _ => None,
    }
}

// Executes the given logout action.
fn execute(action: &LogoutAction) {
    match *action {
        LogoutAction::Poweroff => {
            let _ = Command::new("hyprctl")
                .arg("dispatch exec systemctl poweroff")
                .spawn();
        }
        LogoutAction::Reboot => {
            let _ = Command::new("hyprctl")
                .arg("dispatch exec systemctl reboot")
                .spawn();
        }
        LogoutAction::Logout => {
            let _ = Command::new("hyprctl").arg("dispatch exit").spawn();
        }
        LogoutAction::Lock => {
            let _ = Command::new("hyprctl")
                .arg("dispatch exec hyprlock --immediate")
                .spawn();
        }
        LogoutAction::Suspend => {
            let _ = Command::new("hyprctl")
                .arg("dispatch exec systemctl suspend")
                .spawn();
        }
        LogoutAction::None => (),
    }
    relm4::main_application().quit();
}
