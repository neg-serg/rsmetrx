use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::{Duration, Instant};
use regex::Regex;
use serde::Serialize;

#[derive(Serialize)]
struct Sample {
    iface: String,
    rx_kib_s: f64,
    tx_kib_s: f64,
    up: bool,
}

fn read_to_string(p: &str) -> String {
    fs::read_to_string(p).unwrap_or_default().trim().to_string()
}

fn list_up_ifaces() -> Vec<String> {
    let mut v = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name == "lo" { continue; }
            let state = read_to_string(&format!("/sys/class/net/{}/operstate", name));
            let carrier = read_to_string(&format!("/sys/class/net/{}/carrier", name));
            let up = state == "up" || (state == "unknown" && carrier == "1");
            if up { v.push(name); }
        }
    }
    v
}

fn default_route_iface() -> Option<String> {
    if let Ok(s) = fs::read_to_string("/proc/net/route") {
        for line in s.lines().skip(1) {
            // If Dest == 00000000 -> default route
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() > 2 && cols[1] == "00000000" {
                return Some(cols[0].to_string());
            }
        }
    }
    None
}

fn pick_iface(re: Option<&Regex>) -> Option<String> {
    if let Some(r) = re {
        let ups = list_up_ifaces();
        for n in ups {
            if r.is_match(&n) { return Some(n); }
        }
    }
    if let Some(def) = default_route_iface() {
        return Some(def);
    }
    list_up_ifaces().into_iter().next()
}

fn read_counters(iface: &str) -> Option<(u64, u64, bool)> {
    let rxp = format!("/sys/class/net/{}/statistics/rx_bytes", iface);
    let txp = format!("/sys/class/net/{}/statistics/tx_bytes", iface);
    let opp = format!("/sys/class/net/{}/operstate", iface);

    let rx = fs::read_to_string(&rxp).ok()?.trim().parse::<u64>().ok()?;
    let tx = fs::read_to_string(&txp).ok()?.trim().parse::<u64>().ok()?;
    let up = read_to_string(&opp) == "up";
    Some((rx, tx, up))
}

fn main() -> io::Result<()> {
    // Optional: first CLI arg = regex for iface match
    let re = std::env::args().nth(1).and_then(|pat| Regex::new(&pat).ok());

    // EMA smoothing
    let alpha: f64 = 0.35;

    let mut iface = pick_iface(re.as_ref());
    let mut last_check = Instant::now();

    let mut prev_rx = 0u64;
    let mut prev_tx = 0u64;
    let mut have_prev = false;
    let mut ema_rx = 0.0f64;
    let mut ema_tx = 0.0f64;
    let mut prev_t = Instant::now();

    loop {
        // Re-pick iface every 15s (roaming/cable changes)
        if last_check.elapsed() >= Duration::from_secs(15) {
            let new_iface = pick_iface(re.as_ref());
            if new_iface != iface {
                iface = new_iface;
                have_prev = false;
            }
            last_check = Instant::now();
        }

        if let Some(ref ifn) = iface {
            if let Some((rx, tx, up)) = read_counters(ifn) {
                let now = Instant::now();
                if have_prev {
                    let dt = now.duration_since(prev_t).as_secs_f64().max(1e-3);
                    let drx = rx.saturating_sub(prev_rx) as f64 / 1024.0 / dt; // KiB/s
                    let dtx = tx.saturating_sub(prev_tx) as f64 / 1024.0 / dt; // KiB/s
                    ema_rx = alpha * drx + (1.0 - alpha) * ema_rx;
                    ema_tx = alpha * dtx + (1.0 - alpha) * ema_tx;
                    let sample = Sample {
                        iface: ifn.clone(),
                        rx_kib_s: ema_rx.max(0.0),
                        tx_kib_s: ema_tx.max(0.0),
                        up,
                    };
                    println!("{}", serde_json::to_string(&sample).unwrap());
                    io::stdout().flush().ok();
                } else {
                    // Initialize baseline on first sample
                    have_prev = true;
                }
                prev_rx = rx;
                prev_tx = tx;
                prev_t = now;
            } else {
                // iface vanished; re-pick on next loop
                iface = None;
                have_prev = false;
            }
        } else {
            // try to pick again
            iface = pick_iface(re.as_ref());
            have_prev = false;
        }

        std::thread::sleep(Duration::from_secs(1));
    }
}
