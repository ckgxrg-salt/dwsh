use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::prelude::*;

pub struct Dwsh {}

#[derive(Debug)]
pub enum Message {
    TextInput(String),
}

#[relm4::component(pub)]
impl SimpleComponent for Dwsh {
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
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Dwsh {};
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}
