// Bibliothèque partagée entre l'application complète et le widget

#[cfg(feature = "network")]
use sysinfo::Networks;

#[cfg(feature = "disk")]
use sysinfo::Disks;

// ============================================================================
// MODULE BATTERIE (optionnel)
// ============================================================================
#[cfg(feature = "battery")]
pub mod battery {
    pub fn get_battery_info() -> (f32, bool) {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            
            if let Ok(output) = Command::new("pmset")
                .arg("-g")
                .arg("batt")
                .output()
            {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    for line in stdout.lines() {
                        if line.contains("InternalBattery") && line.contains("%") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            for part in parts {
                                if part.ends_with("%;") || part.ends_with('%') {
                                    let clean = part.trim_end_matches(';').trim_end_matches('%');
                                    if let Ok(percent) = clean.parse::<f32>() {
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
            
            (100.0, false)
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            (100.0, false)
        }
    }
}

#[cfg(feature = "battery")]
pub use battery::get_battery_info;

#[cfg(not(feature = "battery"))]
pub fn get_battery_info() -> (f32, bool) {
    (100.0, false)
}

// ============================================================================
// MODULE RÉSEAU (optionnel)
// ============================================================================
#[cfg(feature = "network")]
pub mod network {
    use sysinfo::Networks;

    pub fn network_deltas(networks: &Networks) -> (u64, u64) {
        let mut rx = 0;
        let mut tx = 0;

        for (_name, data) in networks {
            rx += data.received();
            tx += data.transmitted();
        }

        (rx, tx)
    }

    pub fn network_totals(networks: &Networks) -> (f32, f32) {
        let mut rx = 0_u64;
        let mut tx = 0_u64;

        for (_name, data) in networks {
            rx += data.total_received();
            tx += data.total_transmitted();
        }

        (
            rx as f32 / 1_073_741_824.0,
            tx as f32 / 1_073_741_824.0,
        )
    }
}

#[cfg(feature = "network")]
pub use network::{network_deltas, network_totals};

#[cfg(not(feature = "network"))]
pub fn network_deltas(_networks: &sysinfo::Networks) -> (u64, u64) {
    (0, 0)
}

#[cfg(not(feature = "network"))]
pub fn network_totals(_networks: &sysinfo::Networks) -> (f32, f32) {
    (0.0, 0.0)
}

// ============================================================================
// MODULE DISQUE (optionnel)
// ============================================================================
#[cfg(feature = "disk")]
pub mod disk {
    use sysinfo::Disks;

    pub fn get_disk_usage(disks: &Disks) -> (f32, u64, u64) {
        let mut total_space = 0_u64;
        let mut used_space = 0_u64;

        for disk in disks {
            total_space += disk.total_space();
            used_space += disk.total_space() - disk.available_space();
        }

        let percent = if total_space > 0 {
            (used_space as f32 / total_space as f32) * 100.0
        } else {
            0.0
        };

        let total_gb = total_space / 1_073_741_824;
        let used_gb = used_space / 1_073_741_824;

        (percent, used_gb, total_gb)
    }
}

#[cfg(feature = "disk")]
pub use disk::get_disk_usage;

#[cfg(not(feature = "disk"))]
pub fn get_disk_usage(_disks: &sysinfo::Disks) -> (f32, u64, u64) {
    (0.0, 0, 0)
}
