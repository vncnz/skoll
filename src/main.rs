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

use fuzzy_matcher::skim::SkimMatcherV2;
use gdk::keys::constants;
use gio::prelude::*;
use gtk::{
    builders::{
        BoxBuilder, 
        LabelBuilder,
        EntryBuilder, 
        ListBoxBuilder, 
        ScrolledWindowBuilder
    }, prelude::*, Label, ListBoxRow
};
use libc::LC_ALL;
use std::env::args;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

use std::process::Command;
// use std::error::Error;
use serde_json;

use sysinfo::System;

use bytesize::ByteSize;

pub fn get_from_map<'a, K: Eq + std::hash::Hash, V>(map: &'a HashMap<K, V>, key: &K) -> Option<&'a V> {
    map.get(key) // .expect(&format!("Key not found in map"))
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

fn get_color_gradient(min: f64, max: f64, value: f64) -> String {
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

fn app_startup(application: &gtk::Application) {

    let windows: Vec<NiriWindow>;
    {
        let output = Command::new("niri").arg("msg").arg("-j").arg("windows").output();
        let stdout = String::from_utf8(output.unwrap().stdout).unwrap();
        // println!("\n{:?}", stdout);
        windows = serde_json::from_str(&stdout).unwrap();
    }
    let workspaces: Vec<NiriWorkspace>;
    let workspaces_map: HashMap<u8, NiriWorkspace>;
    {
        let output = Command::new("niri").arg("msg").arg("-j").arg("workspaces").output();
        let stdout = String::from_utf8(output.unwrap().stdout).unwrap();
        // println!("\n{:?}", stdout);
        workspaces = serde_json::from_str(&stdout).unwrap();
        workspaces_map = workspaces.into_iter().map(|ws| (ws.id, ws)).collect();
    }

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

    /* gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Left, 0); // config.margin_left);
    gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Right, 0); // config.margin_right);
    gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Top, 0); // config.margin_top);
    gtk_layer_shell::set_margin(&window, gtk_layer_shell::Edge::Bottom, 0); // config.margin_bottom);

    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Left, config.anchor_left);
    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Right, config.anchor_right);
    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Top, config.anchor_top);
    gtk_layer_shell::set_anchor(&window, gtk_layer_shell::Edge::Bottom, config.anchor_bottom); */

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

    let extra_info_box = BoxBuilder::new()
        .name("extra_info_box")
        .orientation(gtk::Orientation::Horizontal)
        // .width_request(config.width)
        // .height_request(config.height)
        //.margin_top(config.margin_top)
        //.margin_end(config.margin_right)
        //.margin_bottom(config.margin_bottom)
        //.margin_start(config.margin_left)
        .vexpand(false)
        .hexpand(true)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Fill)
        .build();

    let second_row = BoxBuilder::new()
        .name("second_row")
        .orientation(gtk::Orientation::Horizontal)
        // .width_request(config.width)
        // .height_request(config.height)
        // .halign(gtk::Align::Center)
        // .margin_top(config.margin_top)
        // .margin_end(config.margin_right)
        // .margin_bottom(config.margin_bottom)
        // .margin_start(config.margin_left)
        .hexpand(true)
        .vexpand(true)
        .halign(gtk::Align::Fill)
        .valign(gtk::Align::Fill)
        // .hexpand(false)
        .build();
    second_row.set_hexpand(true);

    let search_container = BoxBuilder::new()
        .name("search_container")
        .orientation(gtk::Orientation::Vertical)
        // .width_request(config.width)
        // .height_request(config.height)
        // .halign(gtk::Align::Center)
        .margin_top(config.margin_top)
        .margin_end(config.margin_right)
        .margin_bottom(config.margin_bottom)
        .margin_start(config.margin_left)
        .vexpand(true)
        .hexpand(true)
        .halign(gtk::Align::Fill)
        .valign(gtk::Align::Fill)
        .build();

    second_row.add(&search_container);    
    container.add(&extra_info_box);
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

    let mut entry_hash_map = load_entries(&config, &history.borrow());
    let entry_windows_hash_map = load_entries_running(&config, windows, workspaces_map);
    // let entry_hash_map2 = entry_windows_hash_map.clone();

    entry_hash_map.extend(entry_windows_hash_map);

    let entries = Rc::new(RefCell::new(entry_hash_map));

    /* for win in windows {
        let hbox = BoxBuilder::new()
            .orientation(Orientation::Horizontal)
            .build();
        hbox.pack_start(&image, false, false, 0);
        hbox.pack_end(&label, true, true, 0);

        let row = ListBoxRow::new();
        row.add(&hbox);
        row.style_context().add_class(APP_ROW_CLASS);

        listbox.add(win);
    } */

    for row in (&entries.borrow() as &HashMap<ListBoxRow, AppEntry>).keys() {
        /* let appentry = get_from_map(&entry_hash_map2, &row.clone());
        match appentry {
            Some(val) => {
                if Some(&val.custom_cmd).is_some() {
                    let header_row = ListBoxRow::new();
                    let header_label = Label::new(Some("On display"));
                    header_label.set_markup(&format!("<b>{}</b>", "On display"));
                    header_row.add(&header_label);
                    listbox.add(&header_row);
                }
            }
            None => (),
        } */

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
        // let label_tip_1 = Label::new(Some("Use tray-tui for tray usage!"));

        tips_box.add(&label_tip_1);
    }
    second_row.add(&tips_box);






    let label_sys_avg = Label::new(Some("AVG?"));
    label_sys_avg.set_margin(10);
    
    let label_sys_ram = Label::new(Some("RAM?"));
    label_sys_ram.set_margin(10);

    enum SysUpdate {
        LoadAvg(String),
        RAM(u64, u64, u64, u64),
        Error(String)
    }


    fn get_load_avg() -> SysUpdate {
        if let Ok(output) = std::fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = output.split_whitespace().collect();
            SysUpdate::LoadAvg(format!("{} {} {} 󰬢", parts[0], parts[1], parts[2]))
        } else {
            SysUpdate::Error("Errore".into())
        }
    }

    fn get_ram_info() -> SysUpdate {
        let mut sys = System::new();
        sys.refresh_memory();

        SysUpdate::RAM(sys.total_memory(), sys.used_memory(), sys.total_swap(), sys.used_swap())
    }
    

    let (sender, receiver) = glib::MainContext::channel::<SysUpdate>(glib::PRIORITY_DEFAULT);

    let label_sys_avg_clone = label_sys_avg.clone();
    let label_sys_ram_clone = label_sys_ram.clone();
    // In main thread: connessione all'aggiornamento
    receiver.attach(None, move |info: SysUpdate| {
        match info {
            SysUpdate::LoadAvg(info) => label_sys_avg_clone.set_text(&info),
            SysUpdate::RAM(tm, um, ts, uw) => {
                let umh = ByteSize::b(um).display().iec().to_string();
                let tmh = ByteSize::b(tm).display().iec().to_string();
                let tsh = ByteSize::b(ts).display().iec().to_string();
                let uwh = ByteSize::b(uw).display().iec().to_string();
                let memory_ratio = (um as f64 / tm as f64);
                let color = get_color_gradient(65.0, 90.0, memory_ratio * 100.0);
                label_sys_ram_clone.set_markup(&format!("<span foreground=\"{}\">RAM: {} / {} ({:.2}%) SWAP: {} / {} ({:.2}%)</span>", color, umh, tmh, memory_ratio * 100.0, uwh, tsh, (uw as f64 / ts as f64) * 100.0));
                // label_sys_ram_clone.set_text(&format!("RAM: {} / {} ({:.2}%) SWAP: {} / {} ({:.2}%)", umh, tmh, (um as f64 / tm as f64) * 100.0, uwh, tsh, (uw as f64 / ts as f64) * 100.0))
            },
            SysUpdate::Error(error) => {}
        }
        // sysdata.loadavg = Some(info);
        // println!("\n\n\n\n{}\n\n\n\n", sysdata.loadavg.unwrap_or_default());
        glib::Continue(true)
    });

    // In un altro thread: aggiornamento periodico
    std::thread::spawn(move || {
        loop {
            sender.send(get_load_avg()).expect("Send failed");
            sender.send(get_ram_info()).expect("Send failed");
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    });

    // vbox0.add(&label_sys_avg);
    extra_info_box.add(&label_sys_avg);
    extra_info_box.add(&label_sys_ram);
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