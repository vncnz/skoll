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

mod ratatoskr_socket;
use ratatoskr_socket::*;

use std::process::Command;

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
        .orientation(gtk::Orientation::Horizontal)
        // .width_request(1000)
        // .width_request(500)
        .valign(gtk::Align::Fill)
        // .halign(gtk::Align::Fill)
        .vexpand(true)
        .hexpand(true)
        .build();

    let first_col = BoxBuilder::new()
        .name("second_row")
        .orientation(gtk::Orientation::Vertical)
        .vexpand(true)
        .hexpand(false)
        // .halign(gtk::Align::Fill)
        .valign(gtk::Align::Fill)
        // .margin(50)
        .margin_top(config.margin_top + 50)
        .margin_bottom(config.margin_bottom + 50)
        .margin_start(config.margin_left)
        .build();
    first_col.set_hexpand(false);

    let search_container = BoxBuilder::new()
        .name("search_container")
        .orientation(gtk::Orientation::Vertical)
        .margin_top(config.margin_top)
        .margin_end(config.margin_right)
        .margin_bottom(config.margin_bottom)
        .margin_start(50)
        .vexpand(true)
        .hexpand(true)
        .halign(gtk::Align::Fill)
        .valign(gtk::Align::Fill)
        .build();

    let mut info_items = vec![
        ("loadavg".into(), "N/A".into(), "󰬢".into(), "".into()),
        ("ram".into(), "N/A".into(), "󰍛".into(), "".into()),
        // ("swap".into(), "SWAP".into(), "󰍛".into(), "".into()),
        ("disk".into(), "N/A".into(), "󰋊".into(), "".into()),
        ("weather".into(), "N/A".into(), "".into(), "".into()),
        // ("cpu".into(), "CPU".into(), "IC".into(), "/path/to/icons/cpu.png".into()),
        ("volume".into(), "N/A".into(), "󱄡".into(), "".into()),
        ("brightness".into(), "N/A".into(), "󱧤".into(), "".into()),
        ("temp".into(), "N/A".into(), "󱤋".into(), "".into()),
        ("network".into(), "N/A".into(), "󰲊".into(), "".into()),
        ("battery".into(), "N/A".into(), "x".into(), "".into()),
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

    if TEST_COLORS {
        info_grid.update_color("col0", &*get_color_gradient(0.0));
        info_grid.update_color("col1", &*get_color_gradient(0.1));
        info_grid.update_color("col2", &*get_color_gradient(0.2));
        info_grid.update_color("col3", &*get_color_gradient(0.3));
        info_grid.update_color("col4", &*get_color_gradient(0.4));
        info_grid.update_color("col5", &*get_color_gradient(0.5));
        info_grid.update_color("col6", &*get_color_gradient(0.6));
        info_grid.update_color("col7", &*get_color_gradient(0.7));
        info_grid.update_color("col8", &*get_color_gradient(0.8));
        info_grid.update_color("col9", &*get_color_gradient(0.9));
        info_grid.update_color("col10", &*get_color_gradient(1.0));
    }

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
        .halign(gtk::Align::Start)
        .valign(gtk::Align::End)
        .vexpand(false)
        .hexpand(true)
        .build();

    for txt in [
        "HINTS",
        "1. Tray usage: tray-tui",
        "2. System monitor: btop",
        "3. Disk usage: diskonaut or gdu",
        "4. Timeout and timer: termdown"
        // "2. Bluetooth management: bluetui",
        // "3. Network management: impala"
    ] {
        let label_tip_1 = LabelBuilder::new()
            .label(txt)
            // .margin(10)
            .valign(gtk::Align::End)
            .halign(gtk::Align::Start)
            .vexpand(true)
            .build();

        tips_box.add(&label_tip_1);
    }
    first_col.add(info_grid.widget());
    first_col.add(&tips_box);
    container.add(&first_col);
    container.add(&search_container);

   let (mut sock, rx) = RatatoskrSocket::new("/tmp/ratatoskr.sock");

    // In main thread: connessione all'aggiornamento
    rx.attach(None, move |data: PartialMsg| {
        info_grid.update_from_msg(data);
        // eprintln!("{:?}", &data);
        glib::Continue(true)
    });

    std::thread::spawn(move || {
        loop {
            sock.poll_messages();
            std::thread::sleep(std::time::Duration::from_millis(300));
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