//! Status area

use chrono::{DateTime, Local};
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::prelude::*;

pub struct Status {
    battery_manager: battery::Manager,
    battery: battery::Battery,
    clock: DateTime<Local>,
    volume: f32,
    brightness: f32,
    coffee: bool,
    rotation: bool,
    osk: bool,
}

#[derive(Debug)]
pub enum StatusMsg {
    UpdateBattery,
    UpdateClock,
    SetVolume(f32),
    SetBrightness(f32),
    SetCoffee(bool),
    SetRotation(bool),
    SetOSK(bool),
}

#[relm4::component(pub)]
impl SimpleComponent for Status {
    type Init = (battery::Manager, battery::Battery);
    type Input = StatusMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                // TODO: Sliders
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Button {
                    set_tooltip_text: Some("Inhibit System Idle"),
                    set_icon_name: "coffee-symbolic",
                    connect_clicked => StatusMsg::SetCoffee(!&model.coffee),
                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Status {
            battery_manager: init.0,
            battery: init.1,
            clock: Local::now(),
            volume: 0.0,
            brightness: 0.0,
            coffee: false,
            rotation: false,
            osk: false,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            StatusMsg::UpdateBattery => {
                if let Err(err) = self.battery_manager.refresh(&mut self.battery) {
                    eprintln!("{err} when refreshing battery");
                }
            }
            StatusMsg::UpdateClock => self.clock = Local::now(),
            StatusMsg::SetVolume(val) => self.volume = val,
            StatusMsg::SetBrightness(val) => self.brightness = val,
            StatusMsg::SetCoffee(val) => {
                self.coffee = val;
            }
            StatusMsg::SetRotation(val) => self.rotation = val,
            StatusMsg::SetOSK(val) => self.osk = val,
        }
    }
}
