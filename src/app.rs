use crate::cli::{Cli, Mode};
use crate::config::load_config;
use crate::gradient::{GradientBox, parse_color};
use crate::sound::{BUILTIN_SOUND, SoundPlayer, get_sound_path};
use crate::state::{AppMode, AppState};
use crate::util::{parse_border_sides, parse_border_style};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use figlet_rs::FIGlet;
use ratatui::{Terminal, backend::CrosstermBackend, style::Color};
use std::{collections::HashMap, io, str::FromStr, sync::Arc, time::Duration};

pub fn run() -> Result<(), io::Error> {
    let cli = Cli::parse();
    if cli.config_dir {
        println!(
            "{}",
            crate::config::config_dir()
                .ok_or_else(|| io::Error::other("HOME is not set"))?
                .display()
        );
        return Ok(());
    }
    let mut config = build_config(cli)?;

    enable_raw_mode()?;
    let _restore = TerminalRestore;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    main_loop(&mut terminal, &mut config.app_state, &config.ui_config)
}

struct AppConfig {
    app_state: AppState,
    ui_config: UiConfig,
}

fn build_config(cli: Cli) -> Result<AppConfig, io::Error> {
    let config = load_config();
    let solid_color_requested = cli.fg.is_some() || config.fg.is_some();

    let font_choice = cli
        .font
        .or(config.font)
        .unwrap_or_else(|| "standard".to_string());
    let bg_str = cli.bg.or(config.bg).unwrap_or_else(|| "reset".to_string());
    let fg_str = cli.fg.or(config.fg).unwrap_or_else(|| "cyan".to_string());
    let subtitle_fg_str = cli
        .subtitle_fg
        .or(config.subtitle_fg)
        .unwrap_or_else(|| "cyan".to_string());
    let hidden_help = cli.hidden_help || config.hidden_help.unwrap_or(false);
    let panel_ratio = cli.panel_ratio.or(config.panel_ratio).unwrap_or(50);

    let panel_bg_str = cli
        .panel_bg
        .or(config.panel_bg)
        .unwrap_or_else(|| "reset".to_string());
    let panel_fg_str = cli
        .panel_fg
        .or(config.panel_fg)
        .unwrap_or_else(|| "cyan".to_string());
    let panel_border_str = cli
        .panel_border
        .or(config.panel_border)
        .unwrap_or_else(|| "cyan".to_string());
    let panel_border_sides_str = cli
        .panel_border_sides
        .or(config.panel_border_sides)
        .unwrap_or_else(|| "vertical".to_string());
    let panel_border_style_str = cli
        .panel_border_style
        .or(config.panel_border_style)
        .unwrap_or_else(|| "rounded".to_string());

    let gradient_choice = cli
        .color
        .map(|c| c.join(","))
        .or(config.color)
        .unwrap_or_else(|| {
            if solid_color_requested {
                "none".into()
            } else {
                "warm".into()
            }
        });
    let gradient = if gradient_choice.eq_ignore_ascii_case("none") {
        None
    } else {
        Some(parse_color(&gradient_choice, &HashMap::new()).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, format!("Unknown gradient '{gradient_choice}'. Try --color warm or --color '#89b4fa,#cba6f7'")))?)
    };
    let silent = cli.silent || config.silent.unwrap_or(false);
    let resolve_sound = |name: Option<String>| -> Result<Option<String>, io::Error> {
        if silent {
            return Ok(None);
        }
        match name {
            None => Ok(Some(BUILTIN_SOUND.into())),
            Some(name) => get_sound_path(&name).map(|p| Some(p.to_string_lossy().into_owned()))
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("Alert sound '{name}' not found. Use an existing file or omit the sound option for the bundled alert."))),
        }
    };
    let alarm_sound = resolve_sound(cli.alarm_sound.or(config.alarm_sound))?;
    let countdown_sound = resolve_sound(cli.countdown_sound.or(config.countdown_sound))?;
    let sound_player = (!silent).then(|| Arc::new(SoundPlayer::new()));

    let bg_color = Color::from_str(&bg_str).unwrap_or(Color::Reset);
    let clock_color = Color::from_str(&fg_str).unwrap_or(Color::Cyan);
    let subtitle_color = Color::from_str(&subtitle_fg_str).unwrap_or(Color::Cyan);
    let panel_bg = Color::from_str(&panel_bg_str).unwrap_or(Color::Reset);
    let panel_fg = Color::from_str(&panel_fg_str).unwrap_or(Color::Cyan);
    let panel_border = Color::from_str(&panel_border_str).unwrap_or(Color::Cyan);
    let panel_border_sides = parse_border_sides(&panel_border_sides_str);
    let panel_border_style = parse_border_style(&panel_border_style_str);

    let mut app_state = AppState::new(sound_player, alarm_sound, countdown_sound);
    app_state.mode = if cli.stopwatch {
        AppMode::Stopwatch
    } else if cli.time.is_some() {
        AppMode::Countdown
    } else {
        match cli.mode {
            Mode::Clock => AppMode::Clock,
            Mode::Stopwatch => AppMode::Stopwatch,
            Mode::Countdown => AppMode::Countdown,
        }
    };
    if let Some(duration) = cli.time {
        app_state.cd_initial = duration;
        app_state.cd_remaining = duration;
        app_state.cd_name = "Timer".into();
        app_state.toggle_pause();
    } else if app_state.mode == AppMode::Countdown {
        app_state.show_panel = true;
    }

    let font = {
        let font_choice_expanded = expand_path(&font_choice);
        let font_choice_lower = font_choice.to_lowercase();
        match font_choice_lower.as_str() {
            "standard" => FIGlet::standard().expect("Failed to load standard font"),
            "small" => FIGlet::small().expect("Failed to load small font"),
            "big" => FIGlet::big().expect("Failed to load big font"),
            "slant" => FIGlet::slant().expect("Failed to load slant font"),
            _ => FIGlet::from_file(&font_choice_expanded).unwrap_or_else(|_| {
                eprintln!(
                    "Warning: Failed to load font file '{}', using standard",
                    font_choice
                );
                FIGlet::standard().expect("Failed to load standard font")
            }),
        }
    };

    let ui_config = UiConfig {
        font,
        bg_color,
        clock_color,
        subtitle_color,
        gradient,
        hidden_help,
        panel_ratio,
        panel_bg,
        panel_fg,
        panel_border,
        panel_border_sides,
        panel_border_style,
    };

    Ok(AppConfig {
        app_state,
        ui_config,
    })
}

struct UiConfig {
    font: FIGlet,
    bg_color: Color,
    clock_color: Color,
    subtitle_color: Color,
    gradient: Option<GradientBox>,
    hidden_help: bool,
    panel_ratio: u8,
    panel_bg: Color,
    panel_fg: Color,
    panel_border: Color,
    panel_border_sides: ratatui::widgets::Borders,
    panel_border_style: ratatui::widgets::BorderType,
}

fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app_state: &mut AppState,
    ui_config: &UiConfig,
) -> Result<(), io::Error> {
    loop {
        app_state.update_countdown();
        app_state.check_alarms();
        let (time_str, subtitle_str) = app_state.tick();
        let headers = app_state.get_headers();
        let mode = app_state.mode;
        let show_panel = app_state.show_panel;
        let items = app_state.get_items();

        let footer_str = format!(
            "{} | Tab mode · p panel · Space {} · r reset · q quit",
            mode.title(),
            mode.space_key_desc()
        );

        terminal.draw(|f| {
            crate::ui::draw(
                f,
                &mut crate::ui::UiData {
                    font: &ui_config.font,
                    time_str: &time_str,
                    subtitle_str: &subtitle_str,
                    footer_str: &footer_str,
                    bg_color: ui_config.bg_color,
                    clock_color: ui_config.clock_color,
                    subtitle_color: ui_config.subtitle_color,
                    gradient: ui_config.gradient.clone(),
                    hidden_help: ui_config.hidden_help,
                    show_panel,
                    panel_ratio: ui_config.panel_ratio,
                    panel_bg: ui_config.panel_bg,
                    panel_fg: ui_config.panel_fg,
                    panel_border: ui_config.panel_border,
                    panel_border_sides: ui_config.panel_border_sides,
                    panel_border_style: ui_config.panel_border_style,
                    mode,
                    items: &items,
                    headers,
                    table_state: &mut app_state.table_state,
                    edit_mode: &app_state.edit_mode,
                    input_buffer: &app_state.input_buffer,
                },
            )
        })?;

        let interval = if mode == AppMode::Clock { 250 } else { 50 };
        if event::poll(Duration::from_millis(interval))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match crate::handler::handle_key(key.code, key.modifiers, app_state, headers) {
                crate::handler::Action::Quit => break,
                crate::handler::Action::Continue => {}
            }
        }
    }

    Ok(())
}

fn expand_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen("~", &home, 1);
        }
    } else if path.starts_with("./")
        && let Ok(cwd) = std::env::current_dir()
    {
        return path.replacen(".", &cwd.to_string_lossy(), 1);
    }
    path.to_string()
}

struct TerminalRestore;
impl Drop for TerminalRestore {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, crossterm::cursor::Show);
    }
}
