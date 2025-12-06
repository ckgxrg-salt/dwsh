use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::prelude::*;
use std::fmt::Display;

pub struct LogoutApp {
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
            Self::Lock => write!(f, "Lock Screen"),
        }
    }
}

#[derive(Debug)]
pub enum Message {
    SelectAction(LogoutAction),
}

#[relm4::component(pub)]
impl SimpleComponent for LogoutApp {
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
            set_title: Some("dwsh-logout"),

            add_controller = gtk::GestureClick {
                connect_released[sender] => move |_, _, _, _| sender.input(Message::SelectAction(LogoutAction::None)),
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
                        set_icon_name: "system-shutdown",
                        set_size_request: (300, 250),
                        #[watch]
                        set_margin_top: if model.focused == LogoutAction::Poweroff {0} else {50},
                        connect_clicked => Message::SelectAction(LogoutAction::Poweroff),
                    },
                    gtk::Button {
                        set_label: "Reboot",
                        set_icon_name: "system-reboot",
                        set_size_request: (300, 250),
                        #[watch]
                        set_margin_top: if model.focused == LogoutAction::Reboot {0} else {50},
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
                        set_icon_name: "system-suspend",
                        set_size_request: (300, 250),
                        #[watch]
                        set_margin_top: if model.focused == LogoutAction::Suspend {0} else {50},
                        connect_clicked => Message::SelectAction(LogoutAction::Suspend),
                    },
                    gtk::Button {
                        set_label: "Logout",
                        set_icon_name: "system-log-out",
                        set_size_request: (300, 250),
                        #[watch]
                        set_margin_top: if model.focused == LogoutAction::Logout {0} else {50},
                        connect_clicked => Message::SelectAction(LogoutAction::Logout),
                    },
                    gtk::Button {
                        set_label: "Lock",
                        set_icon_name: "system-lock-screen",
                        set_size_request: (300, 250),
                        #[watch]
                        set_margin_top: if model.focused == LogoutAction::Lock {0} else {50},
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
        let model = LogoutApp {
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

// Executes the given logout action.
fn execute(action: &LogoutAction) {
    println!("{action}");
    relm4::main_application().quit();
}
