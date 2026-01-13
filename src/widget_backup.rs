use iced::widget::{column, container, row, text};
use iced::{time, Border, Color, Element, Length, Shadow, Subscription, Task, Theme};
use sysinfo::{Networks, System};
use std::time::Duration;
use tray_icon::{
    menu::{Menu, MenuItem},
    TrayIconBuilder,
};

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

struct State {
    cpu: f32,
    used_mem_mb: u64,
    total_mem_mb: u64,
    networks: Networks,
    down_mbps: f32,
    up_mbps: f32,
    battery_percent: f32,
    battery_charging: bool,
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
            size: iced::Size::new(280.0, 220.0),
            position: iced::window::Position::Specific(iced::Point::new(
                // Position en haut à droite de l'écran
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

    let mut networks = Networks::new_with_refreshed_list();
    networks.refresh(true);

    let (battery_percent, battery_charging) = get_battery_info();

    let mut state = State {
        cpu: sys.global_cpu_usage(),
        used_mem_mb: sys.used_memory() / 1024,
        total_mem_mb: sys.total_memory() / 1024,
        networks,
        down_mbps: 0.0,
        up_mbps: 0.0,
        battery_percent,
        battery_charging,
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

    // Couleur de la batterie selon le niveau
    let battery_color = if state.battery_percent > 50.0 {
        Color::from_rgb8(0x10, 0xb9, 0x81) // Vert
    } else if state.battery_percent > 20.0 {
        Color::from_rgb8(0xf5, 0x9e, 0x0b) // Orange
    } else {
        Color::from_rgb8(0xef, 0x44, 0x44) // Rouge
    };

    let battery_icon = if state.battery_charging { "⚡" } else { "🔋" };
    let battery_label = format!("{} Batterie", battery_icon);

    container(
        column![
            // Header
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
            
            // Métriques compactes
            column![
                // CPU
                create_metric_row(
                    "💻 CPU".to_string(),
                    format!("{:.0}%", cpu_percent),
                    Color::from_rgb8(0x3b, 0x82, 0xf6),
                ),
                
                // RAM
                create_metric_row(
                    "🧠 RAM".to_string(),
                    format!("{:.0}%", ram_percent),
                    Color::from_rgb8(0xec, 0x48, 0x99),
                ),
                
                // Network
                create_metric_row(
                    "🌐 Réseau".to_string(),
                    format!("↓{:.1} ↑{:.1}", state.down_mbps, state.up_mbps),
                    Color::from_rgb8(0x10, 0xb9, 0x81),
                ),
                
                // Battery
                create_metric_row(
                    battery_label,
                    format!("{:.0}%", state.battery_percent),
                    battery_color,
                ),
            ]
            .spacing(6)
            .padding(10)
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

impl State {
    fn update_metrics(&mut self) {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.networks.refresh(true);

        self.cpu = self.sys.global_cpu_usage();
        self.used_mem_mb = self.sys.used_memory() / 1024;
        self.total_mem_mb = self.sys.total_memory() / 1024;

        // Réseau : débit en Mbps sur l'intervalle
        let (delta_rx, delta_tx) = network_deltas(&self.networks);
        self.down_mbps = delta_rx as f32 * 8.0 / 1_000_000.0;
        self.up_mbps = delta_tx as f32 * 8.0 / 1_000_000.0;

        // Batterie
        let (battery_percent, battery_charging) = get_battery_info();
        self.battery_percent = battery_percent;
        self.battery_charging = battery_charging;
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

// Fonction pour obtenir les informations de batterie sur macOS
fn get_battery_info() -> (f32, bool) {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        // Utilise pmset pour obtenir les infos de batterie sur macOS
        if let Ok(output) = Command::new("pmset")
            .arg("-g")
            .arg("batt")
            .output()
        {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                // Parse la sortie pour extraire le pourcentage
                // Format: "Now drawing from 'Battery Power'\n -InternalBattery-0 (id=12345678)\t95%; discharging; 3:45 remaining present: true"
                for line in stdout.lines() {
                    if line.contains("InternalBattery") && line.contains("%") {
                        // Diviser par les espaces/tabs et chercher le pourcentage
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        for part in parts {
                            if part.ends_with("%;") || part.ends_with('%') {
                                // Retirer le % et le ; si présent
                                let clean = part.trim_end_matches(';').trim_end_matches('%');
                                if let Ok(percent) = clean.parse::<f32>() {
                                    // Vérifier si en charge
                                    let charging = line.contains("charging") && !line.contains("discharging");
                                    let ac_power = stdout.contains("AC Power");
                                    return (percent, charging || ac_power);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Valeur par défaut si la commande échoue
        (100.0, false)
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // Pour les autres plateformes, retourne des valeurs par défaut
        // (peut être étendu avec d'autres méthodes pour Linux/Windows)
        (100.0, false)
    }
}