use iced::widget::{button, column, container, row, text};
use iced::{time, Border, Color, Element, Length, Shadow, Subscription, Task, Theme};

#[cfg(feature = "battery")]
use monitor_app::get_battery_info;
#[cfg(feature = "disk")]
use monitor_app::get_disk_usage;
#[cfg(feature = "network")]
use monitor_app::network_deltas;

#[cfg(feature = "disk")]
use sysinfo::Disks;
#[cfg(feature = "network")]
use sysinfo::Networks;
use sysinfo::System;

use std::time::Duration;
use tray_icon::{
    menu::{Menu, MenuItem},
    TrayIconBuilder,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    System,
    Network,
    Power,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    TabSelected(Tab),
}

struct State {
    cpu: f32,
    used_mem_mb: u64,
    total_mem_mb: u64,
    current_tab: Tab,
    #[cfg(feature = "network")]
    networks: Networks,
    #[cfg(feature = "network")]
    down_mbps: f32,
    #[cfg(feature = "network")]
    up_mbps: f32,
    #[cfg(feature = "battery")]
    battery_percent: f32,
    #[cfg(feature = "battery")]
    battery_charging: bool,
    #[cfg(feature = "disk")]
    disk_percent: f32,
    #[cfg(feature = "disk")]
    disk_used_gb: u64,
    #[cfg(feature = "disk")]
    disk_total_gb: u64,
    #[cfg(feature = "disk")]
    disks: Disks,
    sys: System,
}

pub fn main() -> iced::Result {
    // Créer le menu de la barre de menu
    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("Quitter", true, None);
    tray_menu.append(&quit_item).ok();

    // Créer l'icône de la barre de menu
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("System Monitor - Cliquez pour voir les détails")
        .with_title("⚡")
        .build();

    iced::application(new, update, view)
        .subscription(subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(280.0, 270.0),
            position: iced::window::Position::Specific(iced::Point::new(
                1600.0,
                30.0,
            )),
            decorations: false,
            transparent: false,
            level: iced::window::Level::AlwaysOnTop,
            ..Default::default()
        })
        .run()
}

fn new() -> State {
    let mut sys = System::new_all();
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    #[cfg(feature = "network")]
    let mut networks = Networks::new_with_refreshed_list();
    #[cfg(feature = "network")]
    networks.refresh(true);

    #[cfg(feature = "disk")]
    let disks = Disks::new_with_refreshed_list();

    #[cfg(feature = "battery")]
    let (battery_percent, battery_charging) = get_battery_info();
    
    #[cfg(feature = "disk")]
    let (disk_percent, disk_used_gb, disk_total_gb) = get_disk_usage(&disks);

    let mut state = State {
        cpu: sys.global_cpu_usage(),
        used_mem_mb: sys.used_memory() / 1024,
        total_mem_mb: sys.total_memory() / 1024,
        current_tab: Tab::System,
        #[cfg(feature = "network")]
        networks,
        #[cfg(feature = "network")]
        down_mbps: 0.0,
        #[cfg(feature = "network")]
        up_mbps: 0.0,
        #[cfg(feature = "battery")]
        battery_percent,
        #[cfg(feature = "battery")]
        battery_charging,
        #[cfg(feature = "disk")]
        disk_percent,
        #[cfg(feature = "disk")]
        disk_used_gb,
        #[cfg(feature = "disk")]
        disk_total_gb,
        #[cfg(feature = "disk")]
        disks,
        sys,
    };

    state.update_metrics();
    state
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
            state.update_metrics();
        }
        Message::TabSelected(tab) => {
            state.current_tab = tab;
        }
    }

    Task::none()
}

fn subscription(_state: &State) -> Subscription<Message> {
    time::every(Duration::from_millis(1_000)).map(|_| Message::Tick)
}

fn view(state: &State) -> Element<'_, Message> {
    let cpu_percent = state.cpu;
    let ram_percent = if state.total_mem_mb > 0 {
        (state.used_mem_mb as f32 / state.total_mem_mb as f32) * 100.0
    } else {
        0.0
    };

    #[cfg(feature = "battery")]
    let battery_color = if state.battery_percent > 50.0 {
        Color::from_rgb8(0x10, 0xb9, 0x81)
    } else if state.battery_percent > 20.0 {
        Color::from_rgb8(0xf5, 0x9e, 0x0b)
    } else {
        Color::from_rgb8(0xef, 0x44, 0x44)
    };

    #[cfg(feature = "battery")]
    let battery_icon = if state.battery_charging { "⚡" } else { "🔋" };
    #[cfg(feature = "battery")]
    let battery_label = format!("{} Batterie", battery_icon);

    // Créer les boutons d'onglets
    let tabs = row![
        create_tab_button("Système", Tab::System, state.current_tab),
        create_tab_button("Réseau", Tab::Network, state.current_tab),
        create_tab_button("Énergie", Tab::Power, state.current_tab),
    ]
    .spacing(4)
    .padding(8);

    // Contenu selon l'onglet sélectionné
    let content = match state.current_tab {
        Tab::System => {
            let mut col = column![
                create_metric_row(
                    "💻 CPU".to_string(),
                    format!("{:.0}%", cpu_percent),
                    Color::from_rgb8(0x3b, 0x82, 0xf6),
                ),
                create_metric_row(
                    "🧠 RAM".to_string(),
                    format!("{:.0}%", ram_percent),
                    Color::from_rgb8(0xec, 0x48, 0x99),
                ),
            ]
            .spacing(6);

            #[cfg(feature = "disk")]
            {
                col = col.push(create_metric_row(
                    "💾 Stockage".to_string(),
                    format!("{:.0}% ({}/{}Go)", state.disk_percent, state.disk_used_gb, state.disk_total_gb),
                    Color::from_rgb8(0xf5, 0x9e, 0x0b),
                ));
            }

            col
        }
        Tab::Network => {
            let mut col = column![];
            
            #[cfg(feature = "network")]
            {
                col = col.push(create_metric_row(
                    "📥 Download".to_string(),
                    format!("{:.1} Mb/s", state.down_mbps),
                    Color::from_rgb8(0x10, 0xb9, 0x81),
                ))
                .push(create_metric_row(
                    "📤 Upload".to_string(),
                    format!("{:.1} Mb/s", state.up_mbps),
                    Color::from_rgb8(0x06, 0x99, 0x68),
                ));
            }

            #[cfg(not(feature = "network"))]
            {
                col = col.push(
                    container(text("Module réseau non activé").size(12))
                        .padding(20)
                        .center(Length::Fill)
                );
            }

            col.spacing(6)
        }
        Tab::Power => {
            let mut col = column![];

            #[cfg(feature = "battery")]
            {
                col = col.push(create_metric_row(
                    battery_label,
                    format!("{:.0}%", state.battery_percent),
                    battery_color,
                ));
            }

            #[cfg(not(feature = "battery"))]
            {
                col = col.push(
                    container(text("Module batterie non activé").size(12))
                        .padding(20)
                        .center(Length::Fill)
                );
            }

            col.spacing(6)
        }
    };

    container(
        column![
            container(
                text("System Monitor")
                    .size(14)
                    .color(Color::WHITE)
            )
            .padding(8)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgb8(0x1f, 0x29, 0x37).into()),
                    ..Default::default()
                }
            })
            .width(Length::Fill),
            
            tabs,
            
            content.padding(10)
        ]
        .spacing(0)
    )
    .style(|_theme: &Theme| {
        container::Style {
            background: Some(Color::from_rgb8(0xf3, 0xf4, 0xf6).into()),
            border: Border {
                radius: 12.0.into(),
                color: Color::from_rgb8(0xd1, 0xd5, 0xdb),
                width: 1.0,
            },
            shadow: Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.25),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        }
    })
    .into()
}

fn create_metric_row(
    label: String,
    value: String,
    color: Color,
) -> Element<'static, Message> {
    container(
        row![
            text(label)
                .size(13)
                .color(Color::WHITE)
                .width(Length::Fill),
            text(value)
                .size(16)
                .color(Color::WHITE)
        ]
        .align_y(iced::Alignment::Center)
        .spacing(10)
        .padding(8)
    )
    .style(move |_theme: &Theme| {
        container::Style {
            background: Some(color.into()),
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .width(Length::Fill)
    .into()
}

fn create_tab_button(label: &'static str, tab: Tab, current_tab: Tab) -> Element<'static, Message> {
    let is_active = tab == current_tab;
    
    button(
        text(label)
            .size(12)
            .color(if is_active {
                Color::WHITE
            } else {
                Color::from_rgb8(0x6b, 0x7c, 0x93)
            })
    )
    .padding([6, 12])
    .style(move |_theme: &Theme, _status| {
        button::Style {
            background: Some(if is_active {
                Color::from_rgb8(0x3b, 0x82, 0xf6).into()
            } else {
                Color::from_rgb8(0xe5, 0xe7, 0xeb).into()
            }),
            border: Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            text_color: if is_active {
                Color::WHITE
            } else {
                Color::from_rgb8(0x6b, 0x7c, 0x93)
            },
            ..Default::default()
        }
    })
    .on_press(Message::TabSelected(tab))
    .into()
}

impl State {
    fn update_metrics(&mut self) {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        
        #[cfg(feature = "network")]
        self.networks.refresh(true);
        
        #[cfg(feature = "disk")]
        self.disks.refresh(true);

        self.cpu = self.sys.global_cpu_usage();
        self.used_mem_mb = self.sys.used_memory() / 1024;
        self.total_mem_mb = self.sys.total_memory() / 1024;

        #[cfg(feature = "network")]
        {
            let (delta_rx, delta_tx) = network_deltas(&self.networks);
            self.down_mbps = delta_rx as f32 * 8.0 / 1_000_000.0;
            self.up_mbps = delta_tx as f32 * 8.0 / 1_000_000.0;
        }

        #[cfg(feature = "battery")]
        {
            let (battery_percent, battery_charging) = get_battery_info();
            self.battery_percent = battery_percent;
            self.battery_charging = battery_charging;
        }

        #[cfg(feature = "disk")]
        {
            let (disk_percent, disk_used_gb, disk_total_gb) = get_disk_usage(&self.disks);
            self.disk_percent = disk_percent;
            self.disk_used_gb = disk_used_gb;
            self.disk_total_gb = disk_total_gb;
        }
    }
}
