//! Displays a quote from `fortune`.

use gtk4::prelude::*;
use relm4::prelude::*;

use std::collections::VecDeque;
use tokio::process::Command;

// TODO: Write a config manager
const QUOTE_MAX_WIDTH: usize = 75;
const QUOTE_MAX_LINES: usize = 5;

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
            gtk::Button {
                set_width_request: 50,

                set_tooltip_text: Some("Refresh quote"),
                set_icon_name: "messenger-indicator-symbolic",
                connect_clicked => QuoteMsg::RefreshQuote
            },
            gtk::Label {
                set_size_request: (1210, 200),
                inline_css: "font-size: 24px;",

                set_wrap: true,

                #[watch]
                set_label: &format_quote(&model.text),
                #[watch]
                set_tooltip_text: Some(&model.text)
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
            String::from_utf8(o.stdout).unwrap_or(String::from("Error parsing quote from fortune"))
        })
        .unwrap_or(String::from("Error fetching quote from fortune"))
}

// TODO: awful
fn format_quote(s: &str) -> String {
    let mut lines: VecDeque<String> = s.split('\n').map(String::from).collect();
    let mut result: Vec<String> = Vec::new();

    while let Some(current_line) = lines.pop_front() {
        if result.len() >= QUOTE_MAX_LINES {
            lines.push_front(current_line);
            break;
        }

        if current_line.chars().count() < QUOTE_MAX_WIDTH {
            result.push(current_line);
        } else {
            let target_char_idx = QUOTE_MAX_WIDTH - 1;
            let mut split_byte_idx = current_line.len();
            let mut last_space_byte_idx = None;

            for (char_idx, (byte_idx, c)) in current_line.char_indices().enumerate() {
                if char_idx == target_char_idx {
                    split_byte_idx = byte_idx;
                    if c == ' ' {
                        last_space_byte_idx = Some(byte_idx);
                    }
                    break;
                }
                if c == ' ' {
                    last_space_byte_idx = Some(byte_idx);
                }
            }

            let split_idx = last_space_byte_idx.unwrap_or(split_byte_idx);

            let keep_part = current_line[..split_idx].to_string();
            let remainder = current_line[split_idx..].trim_start();

            result.push(keep_part);

            if !remainder.is_empty() {
                lines.push_front(remainder.to_string());
            }
        }
    }

    if !lines.is_empty() && result.len() == QUOTE_MAX_LINES {
        result.push("......".to_string());
    }

    result.join("\n")
}
