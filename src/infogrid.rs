use bytesize::ByteSize;
use gdk_pixbuf::Pixbuf;
use gtk::builders::BoxBuilder;
use gtk::prelude::*;
use gtk::{glib, Align, Grid, Image, Label};

use std::collections::HashMap;

use crate::ratatoskr_socket::PartialMsg;
use crate::util::get_color_gradient;

pub trait InfoView {
    fn new(info_keys: &[(String, String, String, String)]) -> Self;
    fn widget(&self) -> &gtk::Widget;
    fn update_value(&self, id: &str, new_value: &str) -> &Self;
    fn update_path(&self, id: &str, new_icon_path: &str) -> &Self;
    fn update_color(&self, id: &str, color_css: &str) -> &Self;
    fn update_icon(&self, id: &str, icon_text: &str) -> &Self;
    fn update_from_msg(&self, data: PartialMsg) -> &Self;
}

static ICONSIZE: i32 = 16;
/*
pub struct InfoGrid {
    container: gtk::Widget,
    rows: HashMap<String, (Image, Label, Label, Label)>,
}

impl InfoView for InfoGrid {
    fn new(info_keys: &[(String, String, String, String)]) -> Self {
        // info_keys: Vec<(id, label, icon_path)>
        let grid = Grid::new();
        grid.set_column_spacing(10);
        grid.set_row_spacing(4);
        grid.set_halign(Align::End);

        let mut rows = HashMap::new();

        for (i, (id, label_text, icon_text, icon_path)) in info_keys.iter().enumerate() {
            let icon: Image;
            if !icon_path.is_empty() {
                icon = Image::from_file(icon_path);
                let pixbuf = Pixbuf::from_file_at_size(icon_path, ICONSIZE, ICONSIZE);
                if let Ok(pixbuf_) = pixbuf {
                    icon.set_from_pixbuf(Some(&pixbuf_));
                }
            } else {
                icon = Image::new();
            }
            icon.set_pixel_size(ICONSIZE);
            

            let icon_label = Label::new(Some(icon_text));
            icon_label.set_halign(Align::Start);
            icon_label.style_context().add_class("grid-icon");

            let label = Label::new(Some(label_text));
            label.set_halign(Align::Start);

            let value = Label::new(Some("…"));
            value.set_halign(Align::Start);
            value.set_xalign(1.0);

            grid.attach(&icon, 0, i as i32, 1, 1);
            grid.attach(&icon_label, 0, i as i32, 1, 1);
            grid.attach(&label, 1, i as i32, 1, 1);
            grid.attach(&value, 2, i as i32, 1, 1);

            rows.insert(id.clone(), (icon, icon_label, label, value));
        }

        Self {
            container: grid.upcast(),
            rows,
        }
    }

    fn widget(&self) -> &gtk::Widget {
        &self.container.upcast_ref()
    }

    fn update_value(&self, id: &str, new_value: &str) -> &Self {
        if let Some((_, _, _, value_label)) = self.rows.get(id) {
            value_label.set_text(new_value);
        }
        &self
    }

    fn update_path(&self, id: &str, new_icon_path: &str) -> &Self {
        if let Some((icon, _, _, _)) = self.rows.get(id) {
            // icon.set_from_file(Some(new_icon_path));
            let pixbuf = Pixbuf::from_file_at_size(new_icon_path, ICONSIZE, ICONSIZE).unwrap();
            icon.set_from_pixbuf(Some(&pixbuf));
        }
        &self
    }

    fn update_color(&self, id: &str, color_css: &str) -> &Self {
        if let Some((_, _, _, value_label)) = self.rows.get(id) {
            value_label.set_markup(&format!(r#"<span foreground="{}">{}</span>"#, color_css, glib::markup_escape_text(&value_label.text())));
        }
        &self
    }

    fn update_icon(&self, id: &str, icon_text: &str) -> &Self {
        if let Some((_, icon_label, _, _)) = self.rows.get(id) {
            icon_label.set_text(icon_text);
        }
        &self
    }

    fn update_from_msg (&self, data: PartialMsg) -> &Self{
        let res = data.resource.as_str();
        let color = get_color_gradient(data.warning);
        match res {
            "loadavg" => {
                if let Some(info) = &data.data {
                    self
                        .update_value("loadavg", &*format!("[{:.2} {:.2} {:.2}]", info["m1"], info["m5"], info["m15"]))
                        .update_color("loadavg", &color);
                }
            },
            "ram" => {
                if let Some(info) = &data.data {
                    let tmh = ByteSize::b(info["total_memory"].as_u64().unwrap()).display().iec().to_string();
                    let tsh = ByteSize::b(info["total_swap"].as_u64().unwrap()).display().iec().to_string();
                    
                    self.update_value("ram", &*format!("M: {:.0}% of {}\nS: {:.0}% of {}", info["mem_percent"], tmh, info["swap_percent"], tsh));
                    self.update_color("ram", &color);
                }
            },
            "disk" => {
                if let Some(info) = &data.data {
                    let totalh = ByteSize::b(info["total_size"].as_u64().unwrap()).display().iec().to_string();
                    self.update_value("disk", &*format!("{:.0}% of {}", info["used_percent"], totalh));
                    self.update_color("disk", &color);
                }
            },
            "network" => {
                if let Some(info) = &data.data {
                    let text = if info["conn_type"] == "ethernet" {
                        "ETH"
                    } else {
                        &format!("{} {}%", info["ssid"].as_str().unwrap(), info["signal"]).to_string()
                    };
                    self.update_value("network", &text);
                    self.update_icon("network", &data.icon);
                    self.update_color("network", &color);
                    // span = Some(Span::styled(format!("[WLAN {}%] [IP {}] [NET {}] ", info["signal"], info["ip"].as_str().unwrap(), info["ssid"].as_str().unwrap
                }
            },
            "temperature" => {
                if let Some(info) = &data.data {
                    let v = info["value"].as_f64().unwrap();
                    let text = if v > 0.0 { format!("{:.0}°C", v) } else { "N/A".into() };
                    self.update_value("temp", &text);
                    let def_icon = if v < 80.0 { "" } else 
                                    if v < 85.0 { "" } else
                                    if v < 90.0 { "" } else
                                    if v < 95.0 { "" } else { "" };
                    let icon = if data.icon == "" { def_icon } else { &data.icon };
                    self.update_icon("temp", icon);
                    self.update_color("temp", &color);
                }
            },
            "volume" => {
                if let Some(info) = &data.data {
                    let v = info["value"].as_u64().unwrap();
                    let text = if v == 0 { "Muted".into() } else { format!("{}%", &v) };
                    self.update_value("volume", &text);
                    self.update_icon("volume", info["icon"].as_str().unwrap());
                    self.update_color("volume", &color);
                }
            },
            "battery" => {
                if let Some(info) = &data.data {
                    let bat_symb = match info["state"].as_str() {
                        Some("Charging") => { "󱐋" },
                        Some("Discharging") => { "󰯆" },
                        _ => { info["icon"].as_str().unwrap() }
                    };

                    let eta = info["eta"].as_f64().unwrap_or_default().round() as i32;
                    let h = eta / 60;
                    let m = eta % 60;

                    let second_text = if eta > 0 { format!("Eta {}h{}m", h, m) } else { "Stable level".into() };
                    let text = format!("Level {:.0}%\n{}", info["percentage"].as_f64().unwrap_or(0.0), second_text);
                    self.update_value("battery", &text);
                    self.update_icon("battery", &bat_symb);
                    self.update_color("battery", &color);
                }
            },
            "weather" => {
                // {"icon": "", "text": "Fog", "temp": 8, "temp_real": 9, "temp_unit": "°C", "day": "0", "icon_name": "fog.svg", "sunrise": "07:15", "sunset": "16:48", "sunrise_mins": 435, "sunset_mins": 1008, "daylight": 34385.75, "locality": "Desenzano Del Garda", "humidity": 99}
                if let Some(info) = &data.data {
                    // span = Some(Span::styled(format!("[{} {}] ", info["icon"], info["text"]), Style::default().fg(color)));
                    let temp_text = format!("{}\n{}{} / {}%", info["text"].as_str().unwrap(), info["temp"], info["temp_unit"].as_str().unwrap(), info["humidity"]);
                    self.update_value("weather", &temp_text);
                    // self.update_path("weather", &format!("/home/vncnz/.config/eww/images/weather/{}", info["icon_name"].as_str().unwrap()));
                    self.update_icon("weather", info["icon"].as_str().unwrap());
                }
            },
            "display" => {
                if let Some(info) = &data.data {
                    let temp_text = format!("{}%", info["percentage"]);
                    self.update_value("brightness", &temp_text);
                    self.update_icon("brightness", info["icon"].as_str().unwrap());
                }
            },
            /*"ratatoskr" => {
                if data.warning == 1.0 { span = Some(Span::styled(format!("Ratatoskr disconnected"), Style::default().fg(color))); }
            }*/
            _ => {
                // span = Some(Span::styled(format!("[{}] ", data.resource), Style::default().fg(color)));
            }
        }
        &self
    }
}

*/
pub struct InfoBar {
    container: gtk::Widget,
    rows: HashMap<String, (Image, Label, Label)>,
}

impl InfoView for InfoBar {
    fn new(info_keys: &[(String, String, String, String)]) -> Self {
        // info_keys: Vec<(id, label, icon_path)>
        let inforow = BoxBuilder::new()
            .name("info_bar")
            .orientation(gtk::Orientation::Vertical)
            .vexpand(true)
            .hexpand(true)
            .halign(gtk::Align::Start)
            .valign(gtk::Align::Start)
            .build();

        let mut rows = HashMap::new();

        for (_i, (id, label_text, icon_text, icon_path)) in info_keys.iter().enumerate() {
            let icon: Image;
            if !icon_path.is_empty() {
                icon = Image::from_file(icon_path);
                let pixbuf = Pixbuf::from_file_at_size(icon_path, ICONSIZE, ICONSIZE);
                if let Ok(pixbuf_) = pixbuf {
                    icon.set_from_pixbuf(Some(&pixbuf_));
                    icon.set_pixel_size(ICONSIZE);
                }
            } else {
                icon = Image::new();
                icon.set_pixel_size(0);
            }
            
            let innerbox = BoxBuilder::new()
                .name("inner_box")
                .orientation(gtk::Orientation::Horizontal)
                .vexpand(false)
                .hexpand(false)
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Start)
                .build();
            innerbox.style_context().add_class("island");

            let icon_label = Label::new(Some(icon_text));
            icon_label.set_valign(Align::Start);
            icon_label.set_halign(Align::Start);
            icon_label.style_context().add_class("grid-icon");

            // let label = Label::new(Some(label_text));
            // label.set_halign(Align::Start);

            let value = Label::new(Some(label_text));
            value.set_halign(Align::Start);
            value.set_xalign(1.0);
            value.style_context().add_class("value");

            innerbox.add(&icon);
            innerbox.add(&icon_label);
            innerbox.add(&value);

            inforow.add(&innerbox);

            rows.insert(id.clone(), (icon, icon_label, value));
        }

        Self {
            container: inforow.upcast(),
            rows,
        }
    }

    fn widget(&self) -> &gtk::Widget {
        &self.container.upcast_ref()
    }

    fn update_value(&self, id: &str, new_value: &str) -> &Self {
        if let Some((_, _, value_label)) = self.rows.get(id) {
            value_label.set_text(new_value);
        }
        &self
    }

    fn update_path(&self, id: &str, new_icon_path: &str) -> &Self {
        if let Some((icon, _, _)) = self.rows.get(id) {
            // icon.set_from_file(Some(new_icon_path));
            let pixbuf = Pixbuf::from_file_at_size(new_icon_path, ICONSIZE, ICONSIZE).unwrap();
            icon.set_from_pixbuf(Some(&pixbuf));
            icon.set_pixel_size(ICONSIZE);
        }
        &self
    }

    fn update_color(&self, id: &str, color_css: &str) -> &Self {
        if let Some((_, icon_label, value_label)) = self.rows.get(id) {
            value_label.set_markup(&format!(r#"<span foreground="{}">{}</span>"#, color_css, glib::markup_escape_text(&value_label.text())));
            icon_label.set_markup(&format!(r#"<span foreground="{}">{}</span>"#, color_css, glib::markup_escape_text(&icon_label.text())));
        }
        &self
    }

    fn update_icon(&self, id: &str, icon_text: &str) -> &Self {
        if let Some((_, icon_label, _)) = self.rows.get(id) {
            icon_label.set_text(icon_text);
        }
        &self
    }

    fn update_from_msg (&self, data: PartialMsg) -> &Self{
        let res = data.resource.as_str();
        let color = get_color_gradient(data.warning);
        match res {
            "loadavg" => {
                if let Some(info) = &data.data {
                    self
                        .update_value("loadavg", &*format!("[{:.2} {:.2} {:.2}]", info["m1"], info["m5"], info["m15"]))
                        .update_color("loadavg", &color);
                }
            },
            "ram" => {
                if let Some(info) = &data.data {
                    let tmh = ByteSize::b(info["total_memory"].as_u64().unwrap()).display().iec().to_string();
                    let tsh = ByteSize::b(info["total_swap"].as_u64().unwrap()).display().iec().to_string();
                    
                    self.update_value("ram", &*format!("M: {:.0}% of {}\nS: {:.0}% of {}", info["mem_percent"], tmh, info["swap_percent"], tsh));
                    self.update_color("ram", &color);
                }
            },
            "disk" => {
                if let Some(info) = &data.data {
                    let totalh = ByteSize::b(info["total_size"].as_u64().unwrap()).display().iec().to_string();
                    self.update_value("disk", &*format!("{:.0}% of {}", info["used_percent"], totalh));
                    self.update_color("disk", &color);
                }
            },
            "network" => {
                if let Some(info) = &data.data {
                    let text = if info["conn_type"] == "ethernet" {
                        "ETH"
                    } else {
                        &format!("{} {}%", info["ssid"].as_str().unwrap(), info["signal"]).to_string()
                    };
                    self.update_value("network", &text);
                    self.update_icon("network", &data.icon);
                    self.update_color("network", &color);
                    // span = Some(Span::styled(format!("[WLAN {}%] [IP {}] [NET {}] ", info["signal"], info["ip"].as_str().unwrap(), info["ssid"].as_str().unwrap
                }
            },
            "temperature" => {
                if let Some(info) = &data.data {
                    let v = info["value"].as_f64().unwrap();
                    let text = if v > 0.0 { format!("{:.0}°C", v) } else { "N/A".into() };
                    self.update_value("temp", &text);
                    let def_icon = if v < 80.0 { "" } else 
                                    if v < 85.0 { "" } else
                                    if v < 90.0 { "" } else
                                    if v < 95.0 { "" } else { "" };
                    let icon = if data.icon == "" { def_icon } else { &data.icon };
                    self.update_icon("temp", icon);
                    self.update_color("temp", &color);
                }
            },
            "volume" => {
                if let Some(info) = &data.data {
                    let v = info["value"].as_u64().unwrap();
                    let text = if v == 0 { "Muted".into() } else { format!("{}%", &v) };
                    self.update_value("volume", &text);
                    self.update_icon("volume", info["icon"].as_str().unwrap());
                    self.update_color("volume", &color);
                }
            },
            "battery" => {
                if let Some(info) = &data.data {
                    let bat_symb = match info["state"].as_str() {
                        Some("Charging") => { "󱐋" },
                        Some("Discharging") => { "󰯆" },
                        _ => { info["icon"].as_str().unwrap() }
                    };

                    let eta = info["eta"].as_f64().unwrap_or_default().round() as i32;
                    let h = eta / 60;
                    let m = eta % 60;

                    let second_text = if eta > 0 { format!("Eta {}h{}m", h, m) } else { "Stable level".into() };
                    let text = format!("Level {:.0}%\n{}", info["percentage"].as_f64().unwrap_or(0.0), second_text);
                    self.update_value("battery", &text);
                    self.update_icon("battery", &bat_symb);
                    self.update_color("battery", &color);
                }
            },
            "weather" => {
                // {"icon": "", "text": "Fog", "temp": 8, "temp_real": 9, "temp_unit": "°C", "day": "0", "icon_name": "fog.svg", "sunrise": "07:15", "sunset": "16:48", "sunrise_mins": 435, "sunset_mins": 1008, "daylight": 34385.75, "locality": "Desenzano Del Garda", "humidity": 99}
                if let Some(info) = &data.data {
                    // span = Some(Span::styled(format!("[{} {}] ", info["icon"], info["text"]), Style::default().fg(color)));
                    let temp_text = format!("{}\n{}{} / {}%", info["text"].as_str().unwrap(), info["temp"], info["temp_unit"].as_str().unwrap(), info["humidity"]);
                    self.update_value("weather", &temp_text);
                    // self.update_path("weather", &format!("/home/vncnz/.config/eww/images/weather/{}", info["icon_name"].as_str().unwrap()));
                    self.update_icon("weather", info["icon"].as_str().unwrap());
                }
            },
            "display" => {
                if let Some(info) = &data.data {
                    let temp_text = format!("{}%", info["percentage"]);
                    self.update_value("brightness", &temp_text);
                    self.update_icon("brightness", info["icon"].as_str().unwrap());
                }
            },
            /*"ratatoskr" => {
                if data.warning == 1.0 { span = Some(Span::styled(format!("Ratatoskr disconnected"), Style::default().fg(color))); }
            }*/
            _ => {
                // span = Some(Span::styled(format!("[{}] ", data.resource), Style::default().fg(color)));
            }
        }
        &self
    }
}
