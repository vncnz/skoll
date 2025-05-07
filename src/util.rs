/*
Copyright (C) 2020 Dorian Rudolph

sirula is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

sirula is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with sirula.  If not, see <https://www.gnu.org/licenses/>.
*/

use crate::consts::*;
use freedesktop_entry_parser::parse_entry;
use gio::{prelude::AppInfoExt, AppInfo};
use glib::{shell_parse_argv, GString, ObjectExt};
use gtk::traits::{StyleContextExt, WidgetExt};
use gtk::{prelude::CssProviderExt, CssProvider, StyleContext, STYLE_PROVIDER_PRIORITY_USER};
use std::path::PathBuf;
use std::process::{id, Command};
use shlex::Shlex;

pub fn get_xdg_dirs() -> xdg::BaseDirectories {
    xdg::BaseDirectories::with_prefix(APP_NAME).unwrap()
}

pub fn get_config_file(file: &str) -> Option<PathBuf> {
    get_xdg_dirs().find_config_file(file)
}

pub fn get_history_file(place: bool) -> Option<PathBuf> {
    let xdg = get_xdg_dirs();
    if place {
        xdg.place_cache_file(HISTORY_FILE).ok()
    } else {
        xdg.find_cache_file(HISTORY_FILE)
    }
}

pub fn load_css() {
    if let Some(file) = get_config_file(STYLE_FILE) {
        let provider = CssProvider::new();
        if let Err(err) = provider.load_from_path(file.to_str().unwrap()) {
            eprintln!("Failed to load CSS: {}", err);
        }
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        /* let provider2 = CssProvider::new();
        provider2.load_from_data(
            b"window { background-color: rgba(120, 0, 0, 0.6); }"
        ).expect("Failed to load CSS");
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::default().expect("Error initializing gtk css provider2."),
            &provider2,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        ); */
    }
}

pub fn is_cmd(text: &str, cmd_prefix: &str) -> bool {
    !cmd_prefix.is_empty() && text.starts_with(cmd_prefix)
}

pub fn launch_cmd(cmd_line: &str) {
    let mut parts = shell_parse_argv(cmd_line).expect("Error parsing command line");
    let mut parts_iter = parts.iter_mut();

    let cmd = parts_iter.next().expect("Expected command");

    let mut child = Command::new(cmd);
    child.args(parts_iter);
    child.spawn().expect("Error spawning command");
}

pub fn launch_app(info: &AppInfo, term_command: Option<&str>, launch_cgroups: bool) {
    let command_string = info
        .commandline()
        .unwrap_or_else(|| info.executable())
        .to_str()
        .unwrap()
        .to_string()
        .replace("%U", "")
        .replace("%F", "")
        .replace("%u", "")
        .replace("%f", "");
    let mut command: Vec<String> = Shlex::new(&command_string).collect();

    if info
        .try_property::<GString>("filename")
        .ok()
        .and_then(|s| parse_entry(&s).ok())
        .and_then(|e| {
            e.section("Desktop Entry")
                .attr("Terminal")
                .map(|t| t == "1" || t == "true")
        })
        .unwrap_or_default()
    {
        if let Some(term) = term_command {
            let command_string = term.to_string().replace("{}", &command_string);
		    command = Shlex::new(&command_string).collect();
        } else if let Some(term) = std::env::var_os("TERMINAL") {
        	let term = term.into_string().expect("couldn't convert to string");
        	let mut command_new = vec![term, "-e".into()];
        	command_new.extend(command);
        	command = command_new;
        } else {
            return;
        };
    }
    if launch_cgroups {
        let mut name = info.id().unwrap().to_string();
        name.truncate(name.len() - 8); // remove .desktop extension
        let parsed = Command::new("systemd-escape")
            .arg(name)
            .output()
            .unwrap()
            .stdout;
        let unit = format!(
            "--unit=app-skoll-{}-{}",
            String::from_utf8_lossy(&parsed).trim(),
            id()
        );
        let mut command_new: Vec<String> = vec!["systemd-run".into(), "--scope".into(), "--user".into(), unit];
        command_new.extend(command);
        command = command_new;
    }

    Command::new(&command[0])
        .args(&command[1..])
        .spawn()
        .expect("Error launching app");
}

#[macro_export]
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let r = ((r1 + m) * 255.0).round() as u8;
    let g = ((g1 + m) * 255.0).round() as u8;
    let b = ((b1 + m) * 255.0).round() as u8;

    (r, g, b)
}

pub fn get_color_gradient(min: f64, max: f64, value: f64) -> String {
    let clamped = value.clamp(min, max);
    let ratio = if (max - min).abs() < f64::EPSILON {
        0.5
    } else {
        (clamped - min) / (max - min)
    };

    // Interpola l'hue da 120° (verde) a 0° (rosso)
    let hue = 120.0 * (1.0 - ratio); // 120 -> 0
    let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);

    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

pub fn apply_scale_color(scale: &gtk::Scale, hex_color: &str) {
    let css = format!(
        "
        scale {{
            color: {};
        }}
        scale trough highlight {{
            background-color: {};
        }}
        ",
        hex_color, hex_color
    );

    let provider = CssProvider::new();
    provider.load_from_data(css.as_bytes()).expect("CSS non valido");

    StyleContext::add_provider(
        &scale.style_context(),
        &provider,
        STYLE_PROVIDER_PRIORITY_USER,
    );
}