use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::prelude::*;
use std::fmt::Display;

pub struct LogoutApp {
    text: String,
    focused: LogoutAction,
}

#[derive(PartialEq, Clone, Debug)]
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
    // IcedEvent(Event),
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

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_align: gtk::Align::Center,

                gtk::Box {
                    set_align: gtk::Align::Center,

                    gtk::Button {
                        set_label: "Poweroff",
                        connect_clicked => Message::SelectAction(LogoutAction::Poweroff),
                    },
                    gtk::Button {
                        set_label: "Reboot",
                        connect_clicked => Message::SelectAction(LogoutAction::Reboot),
                    }
                },

                gtk::Label {
                    #[watch]
                    set_label: &model.text,
                },

                gtk::Box {
                    set_align: gtk::Align::Center,

                    gtk::Button {
                        set_label: "Suspend",
                        connect_clicked => Message::SelectAction(LogoutAction::Suspend),
                    },
                    gtk::Button {
                        set_label: "Logout",
                        connect_clicked => Message::SelectAction(LogoutAction::Logout),
                    },
                    gtk::Button {
                        set_label: "Lock",
                        connect_clicked => Message::SelectAction(LogoutAction::Lock),
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
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
                if action != LogoutAction::None && self.focused == action {
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
    panic!("Exited");
}
