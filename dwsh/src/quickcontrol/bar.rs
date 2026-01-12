//! Top bar for the quickcontrol interface

use chrono::{DateTime, Local};
use iced::widget::{row, text};
use iced::{Subscription, Task};
use std::time::{Duration, Instant};

struct Bar {
    battery_manager: battery::Manager,
    battery: battery::Battery,
    clock: DateTime<Local>,
}

#[derive(Clone)]
enum Message {
    UpdateClock(Instant),
    UpdateBattery(Instant),
}

impl Bar {
    fn new() -> Self {
        let manager = battery::Manager::new().expect("unable to create battery manager");
        let battery = manager
            .batteries()
            .expect("unable to get batteries")
            .next()
            .expect("unable to get any battery")
            .expect("failed to access battery");
        Self {
            battery_manager: manager,
            battery,
            clock: Local::now(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_secs(1)).map(Message::UpdateClock),
            iced::time::every(Duration::from_secs(60)).map(Message::UpdateBattery),
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateBattery(_) => {
                self.battery_manager.refresh(&mut self.battery);
            }
            Message::UpdateClock(_) => {
                self.clock = Local::now();
            }
        };
        Task::none()
    }

    fn view(&self) -> iced::Element<Message> {
        row![
            text(format!("{}", self.clock.format("%d"))),
            text(format!(
                "{}%",
                self.battery
                    .energy()
                    .get::<battery::units::energy::watt_hour>()
            ))
        ]
        .into()
    }
}
