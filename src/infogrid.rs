use gdk_pixbuf::Pixbuf;
use gtk::builders::BoxBuilder;
use gtk::prelude::*;
use gtk::{glib, Align, Grid, Image, Label};

use std::collections::HashMap;

pub trait InfoView {
    fn new(info_keys: &[(String, String, String, String)]) -> Self;
    fn widget(&self) -> &gtk::Widget;
    fn update_value(&self, id: &str, new_value: &str) -> &Self;
    fn update_path(&self, id: &str, new_icon_path: &str) -> &Self;
    fn update_color(&self, id: &str, color_css: &str) -> &Self;
    fn update_icon(&self, id: &str, icon_text: &str) -> &Self;
}

static ICONSIZE: i32 = 24;

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
}


pub struct InfoBar {
    container: gtk::Widget,
    rows: HashMap<String, (Image, Label, Label)>,
}

impl InfoView for InfoBar {
    fn new(info_keys: &[(String, String, String, String)]) -> Self {
        // info_keys: Vec<(id, label, icon_path)>
        let inforow = BoxBuilder::new()
            .name("info_bar")
            .orientation(gtk::Orientation::Horizontal)
            .vexpand(false)
            .hexpand(true)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Fill)
            .build();

        let mut rows = HashMap::new();

        for (i, (id, _label_text, icon_text, icon_path)) in info_keys.iter().enumerate() {
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
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Start)
                .build();
            innerbox.style_context().add_class("island");

            let icon_label = Label::new(Some(icon_text));
            icon_label.set_halign(Align::Start);
            icon_label.style_context().add_class("grid-icon");

            // let label = Label::new(Some(label_text));
            // label.set_halign(Align::Start);

            let value = Label::new(Some("…"));
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
        if let Some((_, _, value_label)) = self.rows.get(id) {
            value_label.set_markup(&format!(r#"<span foreground="{}">{}</span>"#, color_css, glib::markup_escape_text(&value_label.text())));
        }
        &self
    }

    fn update_icon(&self, id: &str, icon_text: &str) -> &Self {
        if let Some((_, icon_label, _)) = self.rows.get(id) {
            icon_label.set_text(icon_text);
        }
        &self
    }
}
