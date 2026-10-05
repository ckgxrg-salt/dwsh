//! Display battery level, cpu and memory usage.

use gtk4::prelude::*;
use relm4::prelude::*;

use futures::stream::StreamExt;
use wayle_battery::types::{DeviceState, WarningLevel};
use wayle_core::watch_all;

use crate::services::{battery_service, sysinfo_service};

#[derive(Debug, PartialEq, Eq)]
pub enum BatteryState {
    Charging,
    Normal,
    Low,
    Critical,
}

static BATTERY_DEFAULT_ICON: &str = "battery-symbolic";

pub struct Stats {
    battery: f64,
    battery_state: BatteryState,
    battery_icon: String,
    cpu: f64,
    mem: f64,
}

#[derive(Debug)]
pub enum StatsCmd {
    UpdateBattery(f64, BatteryState, String),
    UpdateCpuMem(f64, f64),
}

#[relm4::component(async, pub)]
impl AsyncComponent for Stats {
    type Init = ();
    type Input = ();
    type CommandOutput = StatsCmd;
    type Output = ();

    view! {
        gtk::Box {
            add_css_class: "panel",
            set_orientation: gtk::Orientation::Vertical,
            set_size_request: (360, 360),

            gtk::Box {
                set_vexpand: true,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_halign: gtk::Align::Center,
                set_margin_top: 10,
                set_margin_bottom: 10,

                gtk::Image {
                    set_valign: gtk::Align::Center,

                    #[watch]
                    set_icon_name: Some(&model.battery_icon),
                },
                gtk::ProgressBar {
                    add_css_class: "battery-bar",
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Center,
                    #[watch]
                    set_class_active: ("low", model.battery_state == BatteryState::Low),
                    #[watch]
                    set_class_active: ("crit", model.battery_state == BatteryState::Critical),
                    #[watch]
                    set_class_active: ("charging", model.battery_state == BatteryState::Charging),

                    #[watch]
                    set_fraction: model.battery,
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_halign: gtk::Align::Center,
                set_margin_top: 10,
                set_margin_bottom: 10,

                gtk::Image {
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("cpu-symbolic"),
                },
                gtk::ProgressBar {
                    add_css_class: "cpu-bar",
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Center,

                    #[watch]
                    set_fraction: model.cpu
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_halign: gtk::Align::Center,
                set_margin_top: 10,
                set_margin_bottom: 10,

                gtk::Image {
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("drive-virtual-symbolic"),
                },
                gtk::ProgressBar {
                    add_css_class: "memory-bar",
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Center,

                    #[watch]
                    set_fraction: model.mem
                },
            },

            gtk::Box {
                set_vexpand: true,
            },
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        watch_battery(&sender).await;
        watch_sysinfo(&sender);

        let model = Stats {
            battery: 0.0,
            battery_state: BatteryState::Normal,
            battery_icon: BATTERY_DEFAULT_ICON.to_string(),
            cpu: 0.0,
            mem: 0.0,
        };
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            StatsCmd::UpdateBattery(percentage, state, icon) => {
                self.battery = percentage;
                self.battery_state = state;
                self.battery_icon = icon;
            }
            StatsCmd::UpdateCpuMem(cpu, mem) => {
                self.cpu = cpu;
                self.mem = mem;
            }
        }
    }
}

async fn watch_battery(sender: &AsyncComponentSender<Stats>) {
    let service = battery_service().await;
    let mut stream = watch_all!(service.device, percentage, state, warning_level, icon_name);
    // let is_present = service.device.is_present.clone();

    sender.command(|out, shutdown| {
        shutdown
            .register(async move {
                while let Some(value) = stream.next().await {
                    let state = if value.state.get() == DeviceState::Charging
                        || value.state.get() == DeviceState::FullyCharged
                    {
                        BatteryState::Charging
                    } else if value.warning_level.get() == WarningLevel::Low {
                        BatteryState::Low
                    } else if value.warning_level.get() == WarningLevel::Critical {
                        BatteryState::Critical
                    } else {
                        BatteryState::Normal
                    };

                    let _ = out.send(StatsCmd::UpdateBattery(
                        value.percentage.get() / 100.0,
                        state,
                        value.icon_name.get(),
                    ));
                }
            })
            .drop_on_shutdown()
    });
}

fn watch_sysinfo(sender: &AsyncComponentSender<Stats>) {
    let service = sysinfo_service();
    let mut stream = watch_all!(service, cpu, memory);

    sender.command(|out, shutdown| {
        shutdown
            .register(async move {
                while let Some(value) = stream.next().await {
                    let cpu_val = (value.cpu.get().usage_percent / 100.0) as f64;
                    let mem_val = (value.memory.get().usage_percent / 100.0) as f64;
                    let _ = out.send(StatsCmd::UpdateCpuMem(cpu_val, mem_val));
                }
            })
            .drop_on_shutdown()
    });
}
