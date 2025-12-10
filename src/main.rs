use iced::widget::canvas::{Canvas, Frame, Geometry, Path, Program, Stroke};
use iced::widget::{column, progress_bar, row, text};
use iced::{mouse, time, Color, Element, Length, Pixels, Point, Rectangle, Renderer, Subscription, Task, Theme};
use sysinfo::{Networks, System};
use std::time::Duration;

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

struct State {
    cpu: f32,
    used_mem_mb: u64,
    total_mem_mb: u64,
    cpu_history: Vec<f32>,
    ram_history: Vec<f32>,
    networks: Networks,
    down_mbps: f32,
    up_mbps: f32,
    down_history: Vec<f32>,
    up_history: Vec<f32>,
    sys: System,
}

pub fn main() -> iced::Result {
    iced::application(new, update, view)
        .subscription(subscription)
        .run()
}

fn new() -> State {
    let mut sys = System::new_all();
    sys.refresh_cpu_usage();
    sys.refresh_memory();

    let mut networks = Networks::new_with_refreshed_list();
    networks.refresh(true);

    let mut state = State {
        cpu: sys.global_cpu_usage(),
        used_mem_mb: sys.used_memory() / 1024,
        total_mem_mb: sys.total_memory() / 1024,
        cpu_history: Vec::new(),
        ram_history: Vec::new(),
        networks,
        down_mbps: 0.0,
        up_mbps: 0.0,
        down_history: Vec::new(),
        up_history: Vec::new(),
        sys,
    };

    state.push_samples();
    state
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
            // On rafraîchit CPU + mémoire régulièrement pour alimenter les graphiques
            state.sys.refresh_cpu_usage();
            state.sys.refresh_memory();
            state.networks.refresh(true);

            state.cpu = state.sys.global_cpu_usage();
            state.used_mem_mb = state.sys.used_memory() / 1024;
            state.total_mem_mb = state.sys.total_memory() / 1024;

            // Réseau : débit en Mbps sur l'intervalle
            let (delta_rx, delta_tx) = network_deltas(&state.networks);
            // 8 bits par octet, division par 1_000_000 pour des Mbps lisibles
            state.down_mbps = delta_rx as f32 * 8.0 / 1_000_000.0;
            state.up_mbps = delta_tx as f32 * 8.0 / 1_000_000.0;

            state.push_samples();
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
        format!("RAM : {:.2} / {:.2} GiB", used_gib, total_gib)
    } else {
        "RAM : (en attente des mesures)".to_string()
    };

    let ram_percent = if state.total_mem_mb > 0 {
        (state.used_mem_mb as f32 / state.total_mem_mb as f32) * 100.0
    } else {
        0.0
    };

    let (total_rx_gib, total_tx_gib) = network_totals(&state.networks);

    let down_max = state
        .down_history
        .iter()
        .copied()
        .fold(1.0_f32, f32::max);
    let up_max = state
        .up_history
        .iter()
        .copied()
        .fold(1.0_f32, f32::max);

    let cpu_chart = Canvas::new(Sparkline {
        data: &state.cpu_history,
        color: Color::from_rgb8(0x32, 0x6d, 0xf8),
        max_value: 100.0,
    })
    .height(Pixels(80.0))
    .width(Length::Fill);

    let ram_chart = Canvas::new(Sparkline {
        data: &state.ram_history,
        color: Color::from_rgb8(0xf8, 0x64, 0x4f),
        max_value: 100.0,
    })
    .height(Pixels(80.0))
    .width(Length::Fill);

    let net_down_chart = Canvas::new(Sparkline {
        data: &state.down_history,
        color: Color::from_rgb8(0x34, 0xd3, 0x6b),
        max_value: down_max,
    })
    .height(Pixels(80.0))
    .width(Length::Fill);

    let net_up_chart = Canvas::new(Sparkline {
        data: &state.up_history,
        color: Color::from_rgb8(0xd9, 0x7a, 0x0b),
        max_value: up_max,
    })
    .height(Pixels(80.0))
    .width(Length::Fill);

    column![
        text("Simple System Monitor"),
        row![
            column![
                text(format!("CPU : {:.1} %", cpu_percent)),
                progress_bar(0.0..=100.0, cpu_percent),
                text("Historique CPU"),
                cpu_chart
            ]
            .spacing(8)
            .width(Length::Fill),
            column![
                text(format!("RAM : {:.1} %", ram_percent)),
                progress_bar(0.0..=100.0, ram_percent),
                text(ram_text),
                text("Historique RAM"),
                ram_chart
            ]
            .spacing(8)
            .width(Length::Fill),
            column![
                text(format!("Réseau : ↓ {:.2} Mbps ↑ {:.2} Mbps", state.down_mbps, state.up_mbps)),
                text(format!("Totaux : ↓ {:.2} GiB ↑ {:.2} GiB", total_rx_gib, total_tx_gib)),
                text("Historique réseau (Mbps)"),
                net_down_chart,
                net_up_chart,
            ]
            .spacing(8)
            .width(Length::Fill),
        ]
        .spacing(16),
    ]
    .spacing(16)
    .padding(16)
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

        self.down_history.push(self.down_mbps);
        Self::trim_history(&mut self.down_history);

        self.up_history.push(self.up_mbps);
        Self::trim_history(&mut self.up_history);
    }

    fn trim_history(history: &mut Vec<f32>) {
        if history.len() > Self::HISTORY {
            let extra = history.len() - Self::HISTORY;
            history.drain(0..extra);
        }
    }
}

fn network_deltas(networks: &Networks) -> (u64, u64) {
    let mut rx = 0;
    let mut tx = 0;

    for (_name, data) in networks { // received/transmitted since last refresh
        rx += data.received();
        tx += data.transmitted();
    }

    (rx, tx)
}

fn network_totals(networks: &Networks) -> (f32, f32) {
    let mut rx = 0_u64;
    let mut tx = 0_u64;

    for (_name, data) in networks { // cumulative totals
        rx += data.total_received();
        tx += data.total_transmitted();
    }

    (
        rx as f32 / 1_073_741_824.0,
        tx as f32 / 1_073_741_824.0,
    )
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