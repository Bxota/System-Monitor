use iced::widget::canvas::{Canvas, Frame, Geometry, Path, Program, Stroke};
use iced::widget::{button, column, container, progress_bar, row, text};
use iced::{mouse, time, Border, Color, Element, Length, Pixels, Point, Rectangle, Renderer, Shadow, Subscription, Task, Theme};

#[cfg(feature = "battery")]
use monitor_app::get_battery_info;
#[cfg(feature = "disk")]
use monitor_app::get_disk_usage;
#[cfg(feature = "network")]
use monitor_app::{network_deltas, network_totals};

#[cfg(feature = "disk")]
use sysinfo::Disks;
#[cfg(feature = "network")]
use sysinfo::Networks;
use sysinfo::System;

use std::time::Duration;

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
    cpu_history: Vec<f32>,
    ram_history: Vec<f32>,
    #[cfg(feature = "network")]
    networks: Networks,
    #[cfg(feature = "network")]
    down_mbps: f32,
    #[cfg(feature = "network")]
    up_mbps: f32,
    #[cfg(feature = "network")]
    down_history: Vec<f32>,
    #[cfg(feature = "network")]
    up_history: Vec<f32>,
    #[cfg(feature = "battery")]
    battery_percent: f32,
    #[cfg(feature = "battery")]
    battery_charging: bool,
    #[cfg(feature = "battery")]
    battery_history: Vec<f32>,
    #[cfg(feature = "disk")]
    disk_percent: f32,
    #[cfg(feature = "disk")]
    disk_used_gb: u64,
    #[cfg(feature = "disk")]
    disk_total_gb: u64,
    #[cfg(feature = "disk")]
    disk_history: Vec<f32>,
    #[cfg(feature = "disk")]
    disks: Disks,
    sys: System,
}

pub fn main() -> iced::Result {
    iced::application(new, update, view)
        .subscription(subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(1400.0, 900.0),
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
        cpu_history: Vec::new(),
        ram_history: Vec::new(),
        #[cfg(feature = "network")]
        networks,
        #[cfg(feature = "network")]
        down_mbps: 0.0,
        #[cfg(feature = "network")]
        up_mbps: 0.0,
        #[cfg(feature = "network")]
        down_history: Vec::new(),
        #[cfg(feature = "network")]
        up_history: Vec::new(),
        #[cfg(feature = "battery")]
        battery_percent,
        #[cfg(feature = "battery")]
        battery_charging,
        #[cfg(feature = "battery")]
        battery_history: Vec::new(),
        #[cfg(feature = "disk")]
        disk_percent,
        #[cfg(feature = "disk")]
        disk_used_gb,
        #[cfg(feature = "disk")]
        disk_total_gb,
        #[cfg(feature = "disk")]
        disk_history: Vec::new(),
        #[cfg(feature = "disk")]
        disks,
        sys,
    };

    state.push_samples();
    state
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
            state.sys.refresh_cpu_usage();
            state.sys.refresh_memory();
            
            #[cfg(feature = "network")]
            state.networks.refresh(true);
            
            #[cfg(feature = "disk")]
            state.disks.refresh(true);

            state.cpu = state.sys.global_cpu_usage();
            state.used_mem_mb = state.sys.used_memory() / 1024;
            state.total_mem_mb = state.sys.total_memory() / 1024;

            #[cfg(feature = "network")]
            {
                let (delta_rx, delta_tx) = network_deltas(&state.networks);
                state.down_mbps = delta_rx as f32 * 8.0 / 1_000_000.0;
                state.up_mbps = delta_tx as f32 * 8.0 / 1_000_000.0;
            }

            #[cfg(feature = "battery")]
            {
                let (battery_percent, battery_charging) = get_battery_info();
                state.battery_percent = battery_percent;
                state.battery_charging = battery_charging;
            }

            #[cfg(feature = "disk")]
            {
                let (disk_percent, disk_used_gb, disk_total_gb) = get_disk_usage(&state.disks);
                state.disk_percent = disk_percent;
                state.disk_used_gb = disk_used_gb;
                state.disk_total_gb = disk_total_gb;
            }

            state.push_samples();
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
    let ram_text = if state.total_mem_mb > 0 {
        let used_gib = state.used_mem_mb as f32 / 1024.0;
        let total_gib = state.total_mem_mb as f32 / 1024.0;
        format!("{:.2} / {:.2} GiB", used_gib, total_gib)
    } else {
        "(en attente)".to_string()
    };

    let ram_percent = if state.total_mem_mb > 0 {
        (state.used_mem_mb as f32 / state.total_mem_mb as f32) * 100.0
    } else {
        0.0
    };

    #[cfg(feature = "network")]
    let (total_rx_gib, total_tx_gib) = network_totals(&state.networks);

    #[cfg(feature = "network")]
    let down_max = state.down_history.iter().copied().fold(1.0_f32, f32::max);
    #[cfg(feature = "network")]
    let up_max = state.up_history.iter().copied().fold(1.0_f32, f32::max);

    let cpu_chart = Canvas::new(Sparkline {
        data: &state.cpu_history,
        color: Color::from_rgb8(0xFF, 0xFF, 0xFF),
        max_value: 100.0,
    })
    .height(Pixels(100.0))
    .width(Length::Fill);

    let ram_chart = Canvas::new(Sparkline {
        data: &state.ram_history,
        color: Color::from_rgb8(0xFF, 0xFF, 0xFF),
        max_value: 100.0,
    })
    .height(Pixels(100.0))
    .width(Length::Fill);

    #[cfg(feature = "network")]
    let net_down_chart = Canvas::new(Sparkline {
        data: &state.down_history,
        color: Color::from_rgb8(0xFF, 0xFF, 0xFF),
        max_value: down_max,
    })
    .height(Pixels(80.0))
    .width(Length::Fill);

    #[cfg(feature = "network")]
    let net_up_chart = Canvas::new(Sparkline {
        data: &state.up_history,
        color: Color::from_rgb8(0xFF, 0xFF, 0xFF),
        max_value: up_max,
    })
    .height(Pixels(80.0))
    .width(Length::Fill);

    #[cfg(feature = "battery")]
    let battery_chart = Canvas::new(Sparkline {
        data: &state.battery_history,
        color: Color::from_rgb8(0xFF, 0xFF, 0xFF),
        max_value: 100.0,
    })
    .height(Pixels(80.0))
    .width(Length::Fill);

    #[cfg(feature = "battery")]
    let battery_color = if state.battery_percent > 50.0 {
        Color::from_rgb8(0x10, 0xb9, 0x81)
    } else if state.battery_percent > 20.0 {
        Color::from_rgb8(0xf5, 0x9e, 0x0b)
    } else {
        Color::from_rgb8(0xef, 0x44, 0x44)
    };

    #[cfg(feature = "battery")]
    let battery_status = if state.battery_charging {
        "⚡ En charge"
    } else {
        "🔋 Sur batterie"
    };

    let cpu_card = create_card(
        "💻 PROCESSEUR",
        Color::from_rgb8(0x3b, 0x82, 0xf6),
        column![
            text(format!("{:.1} %", cpu_percent))
                .size(32)
                .color(Color::WHITE),
            progress_bar(0.0..=100.0, cpu_percent),
            text("Historique (2 min)")
                .size(14)
                .color(Color::from_rgba8(255, 255, 255, 0.8)),
            cpu_chart
        ]
        .spacing(10)
    );

    let ram_card = create_card(
        "🧠 MÉMOIRE",
        Color::from_rgb8(0xec, 0x48, 0x99),
        column![
            text(format!("{:.1} %", ram_percent))
                .size(32)
                .color(Color::WHITE),
            progress_bar(0.0..=100.0, ram_percent),
            text(ram_text)
                .size(14)
                .color(Color::from_rgba8(255, 255, 255, 0.8)),
            text("Historique (2 min)")
                .size(14)
                .color(Color::from_rgba8(255, 255, 255, 0.8)),
            ram_chart
        ]
        .spacing(10)
    );

    #[cfg(feature = "network")]
    let network_card = create_card(
        "🌐 RÉSEAU",
        Color::from_rgb8(0x10, 0xb9, 0x81),
        column![
            row![
                column![
                    text("↓ Téléchargement")
                        .size(14)
                        .color(Color::from_rgba8(255, 255, 255, 0.8)),
                    text(format!("{:.2} Mbps", state.down_mbps))
                        .size(24)
                        .color(Color::WHITE),
                ]
                .spacing(4)
                .width(Length::Fill),
                column![
                    text("↑ Upload")
                        .size(14)
                        .color(Color::from_rgba8(255, 255, 255, 0.8)),
                    text(format!("{:.2} Mbps", state.up_mbps))
                        .size(24)
                        .color(Color::WHITE),
                ]
                .spacing(4)
                .width(Length::Fill),
            ]
            .spacing(16),
            text(format!("Total: ↓ {:.2} GiB  ↑ {:.2} GiB", total_rx_gib, total_tx_gib))
                .size(14)
                .color(Color::from_rgba8(255, 255, 255, 0.8)),
            text("Historique (2 min)")
                .size(14)
                .color(Color::from_rgba8(255, 255, 255, 0.8)),
            net_down_chart,
            net_up_chart,
        ]
        .spacing(10)
    );

    #[cfg(feature = "battery")]
    let battery_card = create_card(
        "🔋 BATTERIE",
        battery_color,
        column![
            text(format!("{:.0} %", state.battery_percent))
                .size(32)
                .color(Color::WHITE),
            progress_bar(0.0..=100.0, state.battery_percent),
            text(battery_status)
                .size(14)
                .color(Color::from_rgba8(255, 255, 255, 0.8)),
            text("Historique (2 min)")
                .size(14)
                .color(Color::from_rgba8(255, 255, 255, 0.8)),
            battery_chart
        ]
        .spacing(10)
    );

    #[cfg(feature = "disk")]
    let disk_chart = Canvas::new(Sparkline {
        data: &state.disk_history,
        color: Color::from_rgb8(0xFF, 0xFF, 0xFF),
        max_value: 100.0,
    })
    .height(Pixels(80.0))
    .width(Length::Fill);

    #[cfg(feature = "disk")]
    let disk_card = create_card(
        "💾 STOCKAGE",
        Color::from_rgb8(0xf5, 0x9e, 0x0b),
        column![
            text(format!("{:.0} %", state.disk_percent))
                .size(32)
                .color(Color::WHITE),
            progress_bar(0.0..=100.0, state.disk_percent),
            text(format!("{} / {} Go", state.disk_used_gb, state.disk_total_gb))
                .size(14)
                .color(Color::from_rgba8(255, 255, 255, 0.8)),
            text("Historique (2 min)")
                .size(14)
                .color(Color::from_rgba8(255, 255, 255, 0.8)),
            disk_chart
        ]
        .spacing(10)
    );

    // Créer les boutons d'onglets
    let tabs = row![
        create_tab_button("Système", Tab::System, state.current_tab),
        create_tab_button("Réseau", Tab::Network, state.current_tab),
        create_tab_button("Énergie", Tab::Power, state.current_tab),
    ]
    .spacing(10);

    // Contenu selon l'onglet sélectionné
    let content_cards = match state.current_tab {
        Tab::System => {
            let mut cards = column![
                row![
                    container(cpu_card).width(Length::Fill),
                    container(ram_card).width(Length::Fill),
                ]
                .spacing(20)
            ]
            .spacing(20);

            #[cfg(feature = "disk")]
            {
                cards = cards.push(
                    row![container(disk_card).width(Length::Fill)].spacing(20)
                );
            }

            cards
        }
        Tab::Network => {
            let mut cards = column![];

            #[cfg(feature = "network")]
            {
                cards = cards.push(
                    row![container(network_card).width(Length::Fill)].spacing(20)
                );
            }

            #[cfg(not(feature = "network"))]
            {
                cards = cards.push(
                    container(
                        text("Module réseau non activé")
                            .size(24)
                            .color(Color::from_rgb8(0x6b, 0x7c, 0x93))
                    )
                    .padding(60)
                    .center(Length::Fill)
                );
            }

            cards.spacing(20)
        }
        Tab::Power => {
            let mut cards = column![];

            #[cfg(feature = "battery")]
            {
                cards = cards.push(
                    row![container(battery_card).width(Length::Fill)].spacing(20)
                );
            }

            #[cfg(not(feature = "battery"))]
            {
                cards = cards.push(
                    container(
                        text("Module batterie non activé")
                            .size(24)
                            .color(Color::from_rgb8(0x6b, 0x7c, 0x93))
                    )
                    .padding(60)
                    .center(Length::Fill)
                );
            }

            cards.spacing(20)
        }
    };

    let content = column![
        text("⚡ Moniteur Système")
            .size(40)
            .color(Color::from_rgb8(0x1f, 0x29, 0x37)),
        tabs,
        content_cards
    ]
    .spacing(20)
    .padding(30);

    container(content)
    .center(Length::Fill)
    .style(|_theme: &Theme| {
        container::Style {
            background: Some(Color::from_rgb8(0xf3, 0xf4, 0xf6).into()),
            ..Default::default()
        }
    })
    .into()
}

fn create_card<'a>(title: &'a str, bg_color: Color, content: iced::widget::Column<'a, Message>) -> Element<'a, Message> {
    container(
        column![
            text(title)
                .size(16)
                .color(Color::from_rgba8(255, 255, 255, 0.9)),
            content
        ]
        .spacing(12)
        .padding(20)
    )
    .padding(0)
    .style(move |_theme: &Theme| {
        container::Style {
            background: Some(bg_color.into()),
            border: Border {
                radius: 16.0.into(),
                ..Default::default()
            },
            shadow: Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.15),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        }
    })
    .into()
}

fn create_tab_button(label: &'static str, tab: Tab, current_tab: Tab) -> Element<'static, Message> {
    let is_active = tab == current_tab;
    
    button(
        text(label)
            .size(18)
            .color(if is_active {
                Color::WHITE
            } else {
                Color::from_rgb8(0x6b, 0x7c, 0x93)
            })
    )
    .padding([12, 24])
    .style(move |_theme: &Theme, _status| {
        button::Style {
            background: Some(if is_active {
                Color::from_rgb8(0x3b, 0x82, 0xf6).into()
            } else {
                Color::from_rgb8(0xe5, 0xe7, 0xeb).into()
            }),
            border: Border {
                radius: 10.0.into(),
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
    const HISTORY: usize = 120;

    fn push_samples(&mut self) {
        self.cpu_history.push(self.cpu);
        Self::trim_history(&mut self.cpu_history);

        let ram_percent = if self.total_mem_mb > 0 {
            (self.used_mem_mb as f32 / self.total_mem_mb as f32) * 100.0
        } else {
            0.0
        };

        self.ram_history.push(ram_percent);
        Self::trim_history(&mut self.ram_history);

        #[cfg(feature = "network")]
        {
            self.down_history.push(self.down_mbps);
            Self::trim_history(&mut self.down_history);

            self.up_history.push(self.up_mbps);
            Self::trim_history(&mut self.up_history);
        }

        #[cfg(feature = "battery")]
        {
            self.battery_history.push(self.battery_percent);
            Self::trim_history(&mut self.battery_history);
        }

        #[cfg(feature = "disk")]
        {
            self.disk_history.push(self.disk_percent);
            Self::trim_history(&mut self.disk_history);
        }
    }

    fn trim_history(history: &mut Vec<f32>) {
        if history.len() > Self::HISTORY {
            let extra = history.len() - Self::HISTORY;
            history.drain(0..extra);
        }
    }
}

struct Sparkline<'a> {
    data: &'a [f32],
    color: Color,
    max_value: f32,
}

impl<'a> Program<Message> for Sparkline<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        if self.data.len() < 2 || self.max_value <= 0.0 {
            return vec![frame.into_geometry()];
        }

        let step_x = if self.data.len() > 1 {
            bounds.width / (self.data.len() as f32 - 1.0)
        } else {
            bounds.width
        };

        let path = Path::new(|builder| {
            for (i, value) in self.data.iter().enumerate() {
                let x = i as f32 * step_x;
                let clamped = value.clamp(0.0, self.max_value);
                let ratio = if self.max_value > 0.0 {
                    clamped / self.max_value
                } else {
                    0.0
                };
                let y = bounds.height - (ratio * bounds.height);

                let point = Point::new(x, y);

                if i == 0 {
                    builder.move_to(point);
                } else {
                    builder.line_to(point);
                }
            }
        });

        frame.stroke(&path, Stroke::default().with_width(2.0).with_color(self.color));

        vec![frame.into_geometry()]
    }
}
