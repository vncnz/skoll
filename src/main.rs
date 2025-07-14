/*
Copyright (C) 2020 Dorian Rudolph
Modified by Vincenzo Minolfi for skoll, a fork of sirula, in 2025.

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

use fuzzy_matcher::skim::SkimMatcherV2;
use gdk::keys::constants;
use gio::prelude::*;
use gtk::{
    builders::{
        BoxBuilder, EntryBuilder, LabelBuilder, ListBoxBuilder, ScrolledWindowBuilder
    }, prelude::*, ListBoxRow
};
use libc::LC_ALL;
use serde_derive::Deserialize;
use std::env::args;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use std::fs;

mod consts;
use consts::*;

mod config;
use config::*;

mod util;
use util::*;

mod app_entry;
use app_entry::*;

mod locale;
use locale::*;

mod history;
use history::*;

mod niri;
use niri::*;

mod infogrid;
use infogrid::*;

use std::process::{Command, Stdio};
// use std::error::Error;

use std::io::{BufReader, BufRead};

use serde_json::{self, Value};

use sysinfo::{Disks, System};

use bytesize::ByteSize;

use std::time::Instant;

/* pub fn get_from_map<'a, K: Eq + std::hash::Hash, V>(map: &'a HashMap<K, V>, key: &K) -> Option<&'a V> {
    map.get(key) // .expect(&format!("Key not found in map"))
} */

static TEST_COLORS: bool = false;

#[derive(Deserialize)]
pub struct WeatherObj {
    pub icon: String,
    pub icon_name: String,
    pub temp: i8,
    pub temp_real: i8,
    pub temp_unit: String,
    pub text: String,
    pub day: String,
    pub sunrise: String,
    pub sunset: String,
    pub sunrise_mins: u64,
    pub sunset_mins: u64,
    pub daylight: f64,
    pub locality: String,
    pub humidity: u8
}

#[derive(Deserialize)]
pub struct VolumeObj {
    pub icon: String,
    pub value: i8,
    pub clazz: String
}

#[derive(Deserialize)]
pub struct BrightnessObj {
    pub icon: String,
    pub percentage: i8,
    pub clazz: String
}

#[derive(Deserialize)]
pub struct NetworkObj {
    // '{"essid": "'"$essid"'", "signal": '"$signal"', "icon": "'"$icon"'", "wired": '"$wired"', "wifi": '"$wifi"', "class": "'"$class"'"}'
    pub icon: String,
    pub signal: i8,
    pub class: String,
    pub essid: String,
    pub wired: i8,
    pub wifi: i8
}

fn app_startup(application: &gtk::Application) {

    let t0 = Instant::now();

    // Stampa le finestre
    // println!("Finestre aperte:");
    /* for window in windows {
        println!(
            "ID: {}, Titolo: {}, App ID: {}",
            window.id,
            window.title.clone().unwrap_or_else(|| "N/A".to_string()),
            window.app_id.clone().unwrap_or_else(|| "N/A".to_string())
        );
    } */


    let config = Config::load();
    let config2 = Config::load();
    let launch_cgroups = config.cgroups;
    let cmd_prefix = config.command_prefix.clone();

    let window = gtk::ApplicationWindow::new(application);
    window.fullscreen();
    window.set_size_request(1000, 700);

    gtk_layer_shell::init_for_window(&window);
    gtk_layer_shell::set_keyboard_interactivity(&window, true);
    gtk_layer_shell::set_layer(&window, gtk_layer_shell::Layer::Overlay);
    gtk_layer_shell::set_namespace(&window, "skoll");

    /* if config.exclusive {
        gtk_layer_shell::auto_exclusive_zone_enable(&window);
    } */

    window.set_decorated(false);
    window.set_app_paintable(true);

    let container = BoxBuilder::new()
        .name(ROOT_BOX_NAME)
        .orientation(gtk::Orientation::Vertical)
        .width_request(1800)
        .width_request(500)
        .valign(gtk::Align::Fill)
        .halign(gtk::Align::Fill)
        .vexpand(true)
        .hexpand(true)
        .build();

    let second_row = BoxBuilder::new()
        .name("second_row")
        .orientation(gtk::Orientation::Horizontal)
        .expand(true)
        .halign(gtk::Align::Fill)
        .valign(gtk::Align::Fill)
        // .hexpand(false)
        .build();
    second_row.set_hexpand(true);

    let search_container = BoxBuilder::new()
        .name("search_container")
        .orientation(gtk::Orientation::Vertical)
        .margin_top(config.margin_top)
        .margin_end(config.margin_right)
        .margin_bottom(config.margin_bottom)
        .margin_start(config.margin_left)
        .vexpand(true)
        .hexpand(true)
        .halign(gtk::Align::Fill)
        .valign(gtk::Align::Fill)
        .build();

        let mut info_items = vec![
            ("loadavg".into(), "Load avg".into(), "󰬢".into(), "".into()),
            ("ram".into(), "RAM".into(), "󰍛".into(), "".into()),
            // ("swap".into(), "SWAP".into(), "󰍛".into(), "".into()),
            ("disk".into(), "Main disk".into(), "󰋊".into(), "".into()),
            ("weather".into(), "Weather".into(), "".into(), "".into()),
            // ("cpu".into(), "CPU".into(), "IC".into(), "/path/to/icons/cpu.png".into()),
            ("volume".into(), "Volume".into(), "󱄡".into(), "".into()),
            ("brightness".into(), "Brightness".into(), "󱧤".into(), "".into()),
            ("temp".into(), "Temperature".into(), "󱤋".into(), "".into()),
            ("network".into(), "Network".into(), "󰲊".into(), "".into()),
            // ("col0".into(), "col0".into(), "".into(), "".into()),
            // ("col1".into(), "col1".into(), "".into(), "".into()),
            // ("col2".into(), "col1".into(), "".into(), "".into()),
            // ("col3".into(), "col1".into(), "".into(), "".into()),
            // ("col4".into(), "col1".into(), "".into(), "".into()),
            // ("col5".into(), "col1".into(), "".into(), "".into()),
            // ("col6".into(), "col1".into(), "".into(), "".into()),
            // ("col7".into(), "col1".into(), "".into(), "".into()),
            // ("col8".into(), "col1".into(), "".into(), "".into()),
            // ("col9".into(), "col1".into(), "".into(), "".into()),
            // ("col10".into(), "col11".into(), "".into(), "".into())
        ];
        if TEST_COLORS {
            let colors_test = vec![
                ("col0".into(), "col0".into(), "".into(), "".into()),
                ("col1".into(), "col1".into(), "".into(), "".into()),
                ("col2".into(), "col1".into(), "".into(), "".into()),
                ("col3".into(), "col1".into(), "".into(), "".into()),
                ("col4".into(), "col1".into(), "".into(), "".into()),
                ("col5".into(), "col1".into(), "".into(), "".into()),
                ("col6".into(), "col1".into(), "".into(), "".into()),
                ("col7".into(), "col1".into(), "".into(), "".into()),
                ("col8".into(), "col1".into(), "".into(), "".into()),
                ("col9".into(), "col1".into(), "".into(), "".into()),
                ("col10".into(), "col11".into(), "".into(), "".into())
            ];
            info_items.extend_from_slice(&colors_test);
        }
        let info_grid = InfoBar::new(&info_items);
        container.add(info_grid.widget());

        if TEST_COLORS {
            info_grid.update_color("col0", &*get_color_gradient(0., 1., 0.0, false));
            info_grid.update_color("col1", &*get_color_gradient(0., 1., 0.1, false));
            info_grid.update_color("col2", &*get_color_gradient(0., 1., 0.2, false));
            info_grid.update_color("col3", &*get_color_gradient(0., 1., 0.3, false));
            info_grid.update_color("col4", &*get_color_gradient(0., 1., 0.4, false));
            info_grid.update_color("col5", &*get_color_gradient(0., 1., 0.5, false));
            info_grid.update_color("col6", &*get_color_gradient(0., 1., 0.6, false));
            info_grid.update_color("col7", &*get_color_gradient(0., 1., 0.7, false));
            info_grid.update_color("col8", &*get_color_gradient(0., 1., 0.8, false));
            info_grid.update_color("col9", &*get_color_gradient(0., 1., 0.9, false));
            info_grid.update_color("col10", &*get_color_gradient(0., 1., 1.0, false));
        }
    
        // Altrove, ad esempio in un async task:
        // info_grid.update_value("ram", "2.9 GiB");
        // info_grid.update_icon("vol", "/path/to/icons/volume-muted.png");
        // info_grid.update_color("cpu", "orange");

    second_row.add(&search_container);    
    // container.add(&extra_info_box);
    container.add(&second_row);

    // vbox.set_css_classes(&["debug"]);

    let entry = EntryBuilder::new().name(SEARCH_ENTRY_NAME).build(); // .width_request(300)
    search_container.pack_start(&entry, false, false, 0);

    let scroll = ScrolledWindowBuilder::new()
        .name(SCROLL_NAME)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();
    search_container.pack_end(&scroll, true, true, 0);

    let listbox = ListBoxBuilder::new().name(LISTBOX_NAME).build();
    scroll.add(&listbox);

    let history = Rc::new(RefCell::new(load_history(config.prune_history)));

    let tn0 = Instant::now();
    let (windows, workspaces_map) = get_niri_windows();
    let tn1 = Instant::now();
    let entry_windows_hash_map = load_entries_running(&config, windows, workspaces_map);
    let tn2 = Instant::now();

    println!("⏱️ get_niri_windows: {:?}", tn1 - tn0);
    println!("⏱️ compute_niri_entries: {:?}", tn2 - tn1);

    let mut entry_hash_map = load_entries(&config, &history.borrow());

    entry_hash_map.extend(entry_windows_hash_map);

    let entries = Rc::new(RefCell::new(entry_hash_map));

    for row in (&entries.borrow() as &HashMap<ListBoxRow, AppEntry>).keys() {
        listbox.add(row);
    }

    window.connect_key_press_event(clone!(entry, listbox, entries => move |window, event| {
        use constants::*;
        #[allow(non_upper_case_globals)]
        Inhibit(match event.keyval() {
            Escape => {
                window.close();
                true
            },
            Down | KP_Down | Tab if entry.has_focus() => {
/*let (windows, workspaces_map) = get_niri_windows();
//let tn1 = Instant::now();

let entry_windows_hash_map = load_entries_running(&config2, windows, workspaces_map);

let entries = Rc::new(RefCell::new(entry_windows_hash_map));

for row in (&entries.borrow() as &HashMap<ListBoxRow, AppEntry>).keys() {
    listbox.add(row);
}*/

                if let Some(r0) = listbox.row_at_index(0) {
                    let es = entries.borrow();
                    if r0.is_selected() {
                        if let Some(r1) = listbox.row_at_index(1) {
                            if let Some(app_entry) = es.get(&r1) {
                                if !app_entry.hidden() {
                                    listbox.select_row(Some(&r1));
                                }
                            }
                        }
                    } else if let Some(app_entry) = es.get(&r0) {
                        if !app_entry.hidden() {
                            listbox.select_row(Some(&r0));
                        }
                    }
                }
                false
            },
            Up | Down | KP_Up | KP_Down | Page_Up | Page_Down | KP_Page_Up | KP_Page_Down | Tab
            | Shift_L | Shift_R | Control_L | Control_R | Alt_L | Alt_R | ISO_Left_Tab | Return
            | KP_Enter => false,
            _ => {
                if !event.is_modifier() && !entry.has_focus() {
                    entry.grab_focus_without_selecting();
                }
                false
            }
        })
    }));

	if config.close_on_unfocus {
	    window.connect_focus_out_event(|window, _| {
    	    window.close();
    	    Inhibit(false)
    	});
    }

    let matcher = SkimMatcherV2::default();
    let term_command = config.term_command.clone();
    entry.connect_changed(clone!(entries, listbox, cmd_prefix => move |e| {
        let text = e.text();
        let is_cmd = is_cmd(&text, &cmd_prefix);
        {
            let mut entries = entries.borrow_mut();
            for entry in entries.values_mut() {
                if is_cmd {
                    entry.hide(); // hide entries in command mode
                } else {
                    entry.update_match(&text, &matcher, &config);
                }
            }
        }
        listbox.invalidate_filter();
        listbox.invalidate_sort();
        listbox.select_row(listbox.row_at_index(0).as_ref());
    }));

    entry.connect_activate(clone!(listbox, window => move |e| {
        let text = e.text();
        if is_cmd(&text, &cmd_prefix) { // command execution direct
            let cmd_line = &text[cmd_prefix.len()..].trim();
            launch_cmd(cmd_line);
            window.close();
        } else if let Some(row) = listbox.row_at_index(0) {
            row.activate();
        }
    }));

    listbox.connect_row_activated(clone!(entries, window, history => move |_, r| {
        let es = entries.borrow();
        let e = &es[r];
        if !e.hidden() {
            match &e.custom_cmd {
                Some(cmd) => {
                    let cmd_parts: Vec<&str> = cmd.split_whitespace().collect();
                    Command::new(&cmd_parts[0])
                        .args(&cmd_parts[1..])
                        .spawn()
                        .expect("Error focusing open app");
                }
                _ => {
                    launch_app(&e.info, term_command.as_deref(), launch_cgroups);
                }
            }

            let mut history = history.borrow_mut();
            update_history(&mut history, e.info.id().unwrap().as_str());
            save_history(&history);

            window.close();
        }
    }));

    listbox.set_filter_func(Some(Box::new(clone!(entries => move |r| {
        let e = entries.borrow();
        !e[r].hidden()
        // true
    }))));

    listbox.set_sort_func(Some(Box::new(clone!(entries => move |a, b| {
        let e = entries.borrow();
        // e[a].cmp(&e[b]) as i32

        if let (Some(e_a), Some(e_b)) = (e.get(a), e.get(b)) {
            if e_a.display == e_b.display {
                e_a.cmp(&e_b) as i32
            } else {
                e_b.display.cmp(&e_a.display) as i32
            }
        } else {
            0
        }
    }))));

    listbox.select_row(listbox.row_at_index(0).as_ref());







    // TIPS SECTION

    let tips_box = BoxBuilder::new()
        .name("tips")
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::End)
        .valign(gtk::Align::End)
        .vexpand(true)
        .build();

    for txt in [
        "1. Tray usage: tray-tui",
        "2. Bluetooth management: bluetui",
        "3. Network management: impala"
    ] {
        let label_tip_1 = LabelBuilder::new()
            .label(txt)
            .margin(10)
            .halign(gtk::Align::Start)
            .build();

        tips_box.add(&label_tip_1);
    }
    second_row.add(&tips_box);
    
    // let memory_adjustment = Adjustment::new(0.0, 0.0, 100.0, 1.0, 10.0, 0.0);
    // let range_sys_ram = ScaleBuilder::new().orientation(gtk::Orientation::Horizontal).adjustment(&memory_adjustment).draw_value(false).sensitive(false).build();

    enum SysUpdate {
        LoadAvg(f64, f64, f64, Option<String>),
        RAM(u64, u64, u64, u64),
        Disk(String, String, u64, u64),
        Weather(WeatherObj),
        Volume(VolumeObj),
        Brightness(BrightnessObj),
        Temperature(String, f32),
        Network(NetworkObj),
        Error(String)
    }


    /* fn get_load_avg() -> SysUpdate {
        if let Ok(output) = std::fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = output.split_whitespace().collect();
            SysUpdate::LoadAvg(parts[0].parse().expect("Error 1m"), parts[1].parse().expect("Error 5m"), parts[2].parse().expect("Error 15m"))
        } else {
            SysUpdate::Error("Errore".into())
        }
    } */

    /* fn get_ram_info() -> SysUpdate {
        let mut sys = System::new();
        sys.refresh_memory();

        SysUpdate::RAM(sys.total_memory(), sys.used_memory(), sys.total_swap(), sys.used_swap())
    } */

    fn get_disk_info() -> SysUpdate {
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            if (disk as &sysinfo::Disk).mount_point() == std::path::Path::new("/") {
                if let Some(name_str) = (disk as &sysinfo::Disk).name().to_str() {
                    if let Some(mount_str) = (disk as &sysinfo::Disk).mount_point().to_str() {
                        return SysUpdate::Disk(
                            name_str.to_string(),
                            mount_str.to_string(),
                            (disk as &sysinfo::Disk).available_space(),
                            (disk as &sysinfo::Disk).total_space()
                        )
                    }
                }
            }
        }
        SysUpdate::Error("Disk not found".to_string())
    }

    fn get_weather () -> SysUpdate {
        let output = Command::new("/home/vncnz/.config/eww/scripts/meteo.sh").arg("'Desenzano Del Garda'").arg("45.457692").arg("10.570684").output();
        let stdout = String::from_utf8(output.unwrap().stdout).unwrap();
        // println!("\n{:?}", stdout);
        // let weather: WeatherObj;
        if let Ok(weather) = serde_json::from_str(&stdout) {
            SysUpdate::Weather(weather)
        } else {
            SysUpdate::Error("Error with serde and weather data".to_string())}
        
    }

    /* fn get_volume () -> SysUpdate {
        let output = Command::new("/home/vncnz/.config/eww/scripts/volume.sh").arg("json").output();
        let stdout = String::from_utf8(output.unwrap().stdout).unwrap();
        // println!("\n{:?}", stdout);
        if let Ok(volume) = serde_json::from_str(&stdout) {
            SysUpdate::Volume(volume)
        } else {
            SysUpdate::Error("Error with serde and volume data".to_string())
        }
    } */

    fn get_brightness () -> SysUpdate {
        let output = Command::new("/home/vncnz/.config/eww/scripts/brightness.sh").arg("json").output();
        let stdout = String::from_utf8(output.unwrap().stdout).unwrap();
        // println!("\n{:?}", stdout);
        if let Ok(brightness) = serde_json::from_str(&stdout) {
            SysUpdate::Brightness(brightness)
        } else {
            SysUpdate::Error("Error with serde and brightness data".to_string())
        }
    }

    fn spawn_network_monitor (sender: glib::Sender<SysUpdate>) {
        let mut child = Command::new("/home/vncnz/.config/eww/scripts/network.sh")
            .arg(&"json")
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn network monitor");
    
        let stdout = child.stdout.take().expect("Failed to open stdout");
        let reader = BufReader::new(stdout);
    
        std::thread::spawn(move || {
            for line in reader.lines() {
                match line {
                    Ok(data) => {
                        // println!("Evento di rete: {}", data);
                        if let Ok(net) = serde_json::from_str(&data) {
                            let _ = sender.send(SysUpdate::Network(net));
                        } else {
                            let _ = sender.send(SysUpdate::Error("Error with serde and network data".to_string()));
                        }
                    }
                    Err(err) => {
                        eprintln!("Errore lettura output network: {}", err);
                        break;
                    }
                }
            }
        });
    }

    /* fn get_sys_temperatures () -> SysUpdate {
        let components = sysinfo::Components::new_with_refreshed_list();
        for component in &components {
            // println!("{component:?}");
            if component.label() == "Tctl" {
                if let Some(temp) = component.temperature() {
                    return SysUpdate::Temperature(component.label().into(), temp);
                } else {
                    return SysUpdate::Error("Temperature Tctl found but none".to_string());
                }
            }
        }
        SysUpdate::Error("Temperature Tctl not found".to_string())
    } */

    fn get2 (sender: glib::Sender<SysUpdate>) {
        if let Ok(contents) = fs::read_to_string("/tmp/ratatoskr.json") {
            let res: Result<Value, serde_json::Error> = serde_json::from_str(&contents);
            if let Ok(data) = res {
                if let [Some(tm), Some(um), Some(ts), Some(us)] = [
                    data["ram"]["total_memory"].as_u64(),
                    data["ram"]["used_memory"].as_u64(),
                    data["ram"]["total_swap"].as_u64(),
                    data["ram"]["used_swap"].as_u64()
                ] {
                    sender.send(SysUpdate::RAM(tm, um, ts, us)).expect("Send error");
                } else {
                    println!("File opened, ram not found");
                }

                if let (Some(m1), Some(m5), Some(m15), color) = (
                    data["loadavg"]["m1"].as_f64(),
                    data["loadavg"]["m5"].as_f64(),
                    data["loadavg"]["m15"].as_f64(),
                    data["loadavg"]["color"].as_str()
                ) {
                    let c = if let Some(col) = color { Some(col.to_string()) } else { None };
                    sender.send(SysUpdate::LoadAvg(m1, m5, m15, c)).expect("Send error");
                } else {
                    println!("File opened, loadavg not found");
                }

                if let (Some(name), Some(value)) = (
                    data["temperature"]["sensor"].as_str(),
                    data["temperature"]["value"].as_f64()
                 ) {
                    sender.send(SysUpdate::Temperature(name.to_string(), value as f32)).expect("Send error");
                } else {
                    println!("File opened, temperature not found");
                }

                if let (Some(icon), Some(value), Some(clazz)) = (
                    data["volume"]["icon"].as_str(),
                    data["volume"]["value"].as_f64(),
                    data["volume"]["clazz"].as_str()
                 ) {
                    sender.send(SysUpdate::Volume(VolumeObj {
                        icon: icon.to_string(),
                        value: value as i8,
                        clazz: clazz.to_string()
                    })).expect("Send error");
                } else {
                    println!("File opened, volume not found");
                }

            } else {
                // File exists but contains shit
                println!("File exists but contains shit");
            }
        } else {
            // No file
            println!("No file");
        }
    }

    /* fn spawn_volume_monitor(sender: glib::Sender<SysUpdate>) {
        std::thread::spawn(move || {
            let child = Command::new("pactl")
                .arg("subscribe")
                .stdout(std::process::Stdio::piped())
                .spawn();

            if let Ok(mut child) = child {
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if line.contains("sink") {
                                let volume = get_current_volume(); // definisci tu
                                let _ = sender.send(SysUpdate::Volume(volume));
                            }
                        }
                    }
                }
            } else {
                eprintln!("Errore nell'eseguire pactl subscribe");
            }
        });
    } */

    

    let (sender, receiver) = glib::MainContext::channel::<SysUpdate>(glib::PRIORITY_DEFAULT);

    // get2(sender.clone());

    // In main thread: connessione all'aggiornamento
    receiver.attach(None, move |info: SysUpdate| {
        match info {
            SysUpdate::LoadAvg(m1, m5, m15, rat_color) => {
                // label_sys_avg_clone.set_text(&format!("󰬢 {} {} {}", m1, m5, m15));
                /* let max = f64::max(m1, f64::max(m5, m15));
                let min = f64::min(m1, f64::min(m5, m15));
                let r = Some("#FF0000".to_string());
                let y = Some("#FFFF00".to_string());
                let g = Some("#00FF00".to_string());
                let m1color = if m1 == max { r.clone() } else { if m1 == min { g.clone() } else { y.clone() }};
                let m5color = if m5 == max { r.clone() } else { if m5 == min { g.clone() } else { y.clone() }};
                let m15color = if m15 == max { r } else { if m15 == min { g } else { y }};
                avg_infobox.update_data([
                    (&*format!("{}", m1), m1/max * 100.0, m1color),
                    (&*format!("{}", m5), m5/max * 100.0, m5color),
                    (&*format!("{}", m15), m15/max * 100.0, m15color)
                ].to_vec()); */
                let color = if rat_color == None { get_color_gradient(1.2, 2., m1/m5, false) } else { rat_color.unwrap() };
                info_grid
                    .update_value("loadavg", &*format!("[{:.2} {:.2} {:.2}]", m1, m5, m15))
                    .update_color("loadavg", &color);
            },
            SysUpdate::RAM(tm, um, ts, us) => {
                // let umh = ByteSize::b(um).display().iec().to_string();
                let tmh = ByteSize::b(tm).display().iec().to_string();
                let tsh = ByteSize::b(ts).display().iec().to_string();
                // let uwh = ByteSize::b(uw).display().iec().to_string();
                let memory_ratio = um as f64 / tm as f64;
                let memory_color = get_color_gradient(60.0, 90.0, memory_ratio * 100.0, false);

                let swap_ratio = us as f64 / ts as f64;
                // let swap_color = get_color_gradient(40.0, 90.0, swap_ratio * 100.0);

                /*  range_sys_ram_clone.set_value(memory_ratio * 100.0);
                apply_scale_color(&range_sys_ram_clone, &memory_color);

                range_sys_swap_clone.set_value(swap_ratio * 100.0);
                apply_scale_color(&range_sys_swap_clone, &swap_color); */

                /* ram_infobox.update_data([
                    (&*format!("{:.0}% of {}", memory_ratio * 100.0, tmh), memory_ratio * 100.0, Some(memory_color.clone())),
                    (&*format!("{:.0}% of {}", swap_ratio * 100.0, tsh), 50.0, Some(swap_color.clone()))
                ].to_vec()); */

                info_grid.update_value("ram", &*format!("M: {:.0}% of {}\nS: {:.0}% of {}", memory_ratio * 100.0, tmh, swap_ratio * 100.0, tsh));
                info_grid.update_color("ram", &memory_color);

                // info_grid.update_value("swap", &*format!("{:.0}% of {}", swap_ratio * 100.0, tsh));
                // info_grid.update_color("swap", &swap_color);
            },
            SysUpdate::Disk(_name, _mount_point, avb, total) => {
                let totalh = ByteSize::b(total).display().iec().to_string();
                let disk_ratio = (total - avb) as f64 / total as f64;
                let disk_color = get_color_gradient(60.0, 90.0, disk_ratio * 100.0, false);
                // range_sys_disk_clone.set_value(disk_ratio * 100.0);
                // apply_scale_color(&range_sys_disk_clone, &disk_color);
                // label_sys_disk_clone.set_markup(&format!("<span foreground=\"{}\">󰋊 {:.0}% of {} on {}</span>", disk_color, disk_ratio * 100.0, totalh, name));

                info_grid.update_value("disk", &*format!("{:.0}% of {}", disk_ratio * 100.0, totalh));
                info_grid.update_color("disk", &disk_color);
            },
            SysUpdate::Weather(weather) => {
                let temp_text = format!("{}{}", weather.temp, weather.temp_unit);
                info_grid.update_value("weather", &temp_text);
                info_grid.update_path("weather", &format!("/home/vncnz/.config/eww/images/weather/{}", weather.icon_name));
            },
            SysUpdate::Volume(volume) => {
                let text = if volume.value == 0 { "Muted".into() } else { format!("{}%", volume.value) };
                let volume_color = get_color_gradient(40.0, 100.0, volume.value as f64, false);
                info_grid.update_value("volume", &text);
                info_grid.update_icon("volume", &*volume.icon);
                info_grid.update_color("volume", &volume_color);
            },
            SysUpdate::Brightness(brightness) => {
                let text = format!("{}%", brightness.percentage);
                // let brightness_color = get_color_gradient(30.0, 60.0, brightness.percentage as f64);
                info_grid.update_value("brightness", &text);
                info_grid.update_icon("brightness", &*brightness.icon);
                // info_grid.update_color("brightness", &brightness_color);
            },
            SysUpdate::Temperature(sensor, value) => {
                let text = format!("{:.0}°C", value);
                let temp_color = get_color_gradient(80.0, 99.0, value as f64, false);
                info_grid.update_value("temp", &text);
                let icon = if value < 80.0 { "" } else 
                                         if value < 85.0 { "" } else
                                         if value < 90.0 { "" } else
                                         if value < 95.0 { "" } else { "" };
                info_grid.update_icon("temp", icon);
                info_grid.update_color("temp", &temp_color);
            },
            SysUpdate::Network(net) => {
                let text = format!("{}%", net.signal);
                let color = get_color_gradient(20.0, 60.0, net.signal as f64, true);
                info_grid.update_value("network", &text);
                info_grid.update_icon("network", &net.icon);
                // info_grid.update_icon("temp", "");
                info_grid.update_color("network", &color);
            },
            SysUpdate::Error(error) => {
                println!("ERROR: {}", error);
            }
        }
        // sysdata.loadavg = Some(info);
        // println!("\n\n\n\n{}\n\n\n\n", sysdata.loadavg.unwrap_or_default());
        glib::Continue(true)
    });


    spawn_network_monitor(sender.clone());

    std::thread::spawn(move || {
        sender.send(get_disk_info()).expect("Send failed");
        sender.send(get_weather()).expect("Send failed");
        // sender.send(get_weather()).expect("Send failed");
        let mut counter = 0;
        loop {
            // if counter % 2 == 0 { sender.send(get_load_avg()).expect("Send failed") };
            // if counter % 2 == 0 { sender.send(get_ram_info()).expect("Send failed") };
            // if counter % 2 == 0 { sender.send(get_sys_temperatures()).expect("Send failed") };
            // sender.send(get_volume()).expect("Send failed");
            sender.send(get_brightness()).expect("Send failed");

            if counter % 2 == 0 { get2(sender.clone()) };

            counter += 1;
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    });

    window.set_child(Some(&container));

    if let Some(display) = gdk::Display::default() {
        if let Some(monitor) = display.monitor(0) {
            let geometry = monitor.geometry();
            window.set_size_request(geometry.width(), geometry.height());
            window.move_(geometry.x(), geometry.y());
        } else {
            println!("\n\nNO MONITOR\n\n");
        }
    } else {
        println!("\n\nNO DISPLAY\n\n");
    }

    let t1 = Instant::now();
    println!("⏱️ app_startup:        {:?}", t1 - t0);

    window.connect_realize(move |_| {
        let t_realized = Instant::now();
        println!("🖼️ Window realized at {:?}", t_realized - t0);
    });

    window.show_all()
}

fn main() {
    set_locale(LC_ALL, "");

    let application = gtk::Application::new(Some(APP_ID), Default::default());

    application.connect_startup(|app| {
        load_css();
        app_startup(app);
    });

    application.connect_activate(|_| {
        //do nothing
    });

    application.run_with_args(&args().collect::<Vec<_>>());
}