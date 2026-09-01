use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use pastures_core::{Config, Liveness, SessionRecord};

/// Ranks your live AI coding sessions by your own engagement, not the agent's activity.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Emit records as JSON (one array), warmest first.
    #[arg(long)]
    json: bool,

    /// Show the warmth score column.
    #[arg(long)]
    scores: bool,

    /// Print the effective configuration as TOML and exit.
    #[arg(long)]
    dump_config: bool,

    /// Config file (default: ~/.config/pastures/config.toml).
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref())?;

    if cli.dump_config {
        print!("{}", config.to_toml());
        return Ok(());
    }

    let records = pastures_core::scan(&config)?;

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&records)?);
        return Ok(());
    }

    if records.is_empty() {
        eprintln!("no live sessions");
        return Ok(());
    }
    print_table(&records, cli.scores);
    Ok(())
}

fn print_table(records: &[SessionRecord], scores: bool) {
    let home = dirs_home();
    let rows: Vec<Vec<String>> = records
        .iter()
        .map(|r| {
            let mut row = vec![
                truncate(&r.label, 48),
                r.last_turn_at
                    .map(|t| relative(Utc::now().signed_duration_since(t)))
                    .unwrap_or_else(|| "-".into()),
                r.turns.to_string(),
                liveness_glyph(r.liveness).to_string(),
                r.git_branch
                    .as_deref()
                    .filter(|b| *b != "HEAD")
                    .map(|b| truncate(b, 32))
                    .unwrap_or_else(|| "-".into()),
                shorten_home(&r.cwd, home.as_deref()),
            ];
            if scores {
                row.insert(1, format!("{:.2}", r.warmth));
            }
            row
        })
        .collect();

    let mut header = vec!["SESSION", "LAST", "TURNS", "LIVE", "BRANCH", "DIR"];
    if scores {
        header.insert(1, "WARMTH");
    }
    let right_aligned = |i: usize| header[i] == "LAST" || header[i] == "TURNS" || header[i] == "WARMTH";

    let widths: Vec<usize> = (0..header.len())
        .map(|i| {
            rows.iter()
                .map(|r| r[i].chars().count())
                .chain(std::iter::once(header[i].len()))
                .max()
                .unwrap_or(0)
        })
        .collect();

    let render = |cells: Vec<String>| -> String {
        cells
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let pad = widths[i] - c.chars().count();
                if right_aligned(i) {
                    format!("{}{}", " ".repeat(pad), c)
                } else if i == cells.len() - 1 {
                    c.clone()
                } else {
                    format!("{}{}", c, " ".repeat(pad))
                }
            })
            .collect::<Vec<_>>()
            .join("  ")
    };

    println!("{}", render(header.iter().map(|h| h.to_string()).collect()));
    for row in rows {
        println!("{}", render(row));
    }
}

fn liveness_glyph(l: Liveness) -> &'static str {
    match l {
        Liveness::Busy => "busy",
        Liveness::Idle => "idle",
        Liveness::Shell => "shell",
        Liveness::Unknown => "?",
    }
}

fn relative(d: chrono::Duration) -> String {
    let secs = d.num_seconds().max(0);
    if secs < 60 {
        "now".into()
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86_400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86_400)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max - 1).collect();
    format!("{}…", cut.trim_end())
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn shorten_home(p: &std::path::Path, home: Option<&std::path::Path>) -> String {
    if let Some(h) = home {
        if let Ok(rest) = p.strip_prefix(h) {
            return format!("~/{}", rest.display());
        }
    }
    p.display().to_string()
}
