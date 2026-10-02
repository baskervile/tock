use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Mode {
    Clock,
    Stopwatch,
    Countdown,
}

/// A terminal ASCII digital clock and timer
#[derive(Parser, Debug)]
#[command(
    name = "tock",
    version,
    about = "A terminal ASCII digital clock and timer"
)]
pub struct Cli {
    /// Start directly in clock, stopwatch or countdown mode
    #[arg(long, value_enum, default_value = "clock")]
    pub mode: Mode,
    /// Start in stopwatch mode
    #[arg(short = 's', long, conflicts_with = "time")]
    pub stopwatch: bool,
    /// Start a countdown, e.g. 5m, 1h30m or 45s
    #[arg(short = 't', long, value_parser = parse_duration)]
    pub time: Option<std::time::Duration>,
    /// Disable alerts
    #[arg(long)]
    pub silent: bool,
    /// Print the active configuration directory and exit
    #[arg(long)]
    pub config_dir: bool,

    /// Specify font name (standard, small, big, slant) or .flf file path
    #[arg(short, long)]
    pub font: Option<String>,

    /// Global background color (e.g., black, "#1e1e1e", reset)
    #[arg(short = 'b', long)]
    pub bg: Option<String>,

    /// Clock text color (e.g., cyan, "#00ff00", white)
    #[arg(short = 'c', long)]
    pub fg: Option<String>,

    /// Subtitle text color (e.g., cyan, "#00ff00", white)
    #[arg(long)]
    pub subtitle_fg: Option<String>,

    /// Panel height ratio when visible (0-100, default 50)
    #[arg(short = 'r', long)]
    pub panel_ratio: Option<u8>,

    /// Panel background color (e.g., black, "#1e1e1e", reset)
    #[arg(long)]
    pub panel_bg: Option<String>,

    /// Panel foreground/text color (e.g., cyan, "#00ff00", white)
    #[arg(long)]
    pub panel_fg: Option<String>,

    /// Panel border color (e.g., cyan, "#00ff00", white)
    #[arg(long)]
    pub panel_border: Option<String>,

    /// Panel border sides: none, all, left, right, top, bottom, horizontal, vertical (default: all)
    #[arg(long)]
    pub panel_border_sides: Option<String>,

    /// Panel border style: plain, rounded, double, thick (default: plain)
    #[arg(long)]
    pub panel_border_style: Option<String>,

    /// Custom alarm sound (default: bundled alert)
    #[arg(long)]
    pub alarm_sound: Option<String>,

    /// Custom countdown sound (default: bundled alert)
    #[arg(long)]
    pub countdown_sound: Option<String>,

    /// Clock gradient (default: warm; use none for a solid color): preset name (rainbow, sinebow, viridis, etc.) or custom colors (red,blue,green)
    #[arg(long, num_args(1..), value_delimiter(','))]
    pub color: Option<Vec<String>>,

    /// Hide help text in footer and panel
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub hidden_help: bool,
}

fn parse_duration(value: &str) -> Result<std::time::Duration, String> {
    let mut total = 0u64;
    let mut digits = String::new();
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            continue;
        }
        let factor = match ch {
            'h' => 3600,
            'm' => 60,
            's' => 1,
            _ => return Err("Use a duration such as 5m, 1h30m or 45s".into()),
        };
        let amount: u64 = digits.parse().map_err(|_| "Missing duration number")?;
        total = total
            .checked_add(amount.checked_mul(factor).ok_or("Duration too large")?)
            .ok_or("Duration too large")?;
        digits.clear();
    }
    if !digits.is_empty() {
        return Err("Add a unit: h, m or s".into());
    }
    if total == 0 {
        return Err("Duration must be greater than zero".into());
    }
    Ok(std::time::Duration::from_secs(total))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn countdown_duration_validation() {
        assert_eq!(parse_duration("1h30m").unwrap().as_secs(), 5400);
        for input in ["", "0s", "5", "-1m", "ms", "999999999999999999999h"] {
            assert!(parse_duration(input).is_err());
        }
    }
}
