//! Displays a quote from `fortune`.

use gtk4::prelude::*;
use relm4::prelude::*;

use tokio::process::Command;

// TODO: Write a config manager
const QUOTE_MAX_WIDTH_CHARS: i32 = 100;

pub struct Quote {
    text: String,
}

#[derive(Debug)]
pub enum QuoteMsg {
    RefreshQuote,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for Quote {
    type Init = ();
    type Input = QuoteMsg;
    type Output = ();

    view! {
        gtk::Box {
            add_css_class: "panel",
            set_size_request: (1210, 200),
            set_spacing: 10,

            gtk::Button {
                set_width_request: 50,
                set_valign: gtk::Align::Center,

                set_tooltip_text: Some("Refresh quote"),
                set_icon_name: "messenger-indicator-symbolic",
                connect_clicked => QuoteMsg::RefreshQuote
            },

            gtk::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_vscrollbar_policy: gtk::PolicyType::Automatic,

                gtk::Label {
                    inline_css: "font-size: 24px;",
                    set_wrap: true,
                    set_wrap_mode: gtk::pango::WrapMode::Word,
                    set_max_width_chars: QUOTE_MAX_WIDTH_CHARS,
                    set_xalign: 0.5,
                    set_valign: gtk::Align::Center,
                    set_hexpand: true,

                    #[watch]
                    set_label: &model.text,
                    #[watch]
                    set_tooltip_text: Some(&model.text)
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            text: get_fortune().await,
        };
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        match message {
            QuoteMsg::RefreshQuote => self.text = get_fortune().await,
        }
    }
}

async fn get_fortune() -> String {
    Command::new("fortune")
        .output()
        .await
        .map(|o| {
            String::from_utf8(o.stdout)
                .map(|s| s.trim_end().to_string())
                .unwrap_or(String::from("Error parsing quote from fortune"))
        })
        .unwrap_or(String::from("Error fetching quote from fortune"))
}
