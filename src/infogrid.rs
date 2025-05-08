use gtk::prelude::*;
use gtk::{glib, Align, Grid, Image, Label, Orientation};

use std::collections::HashMap;

pub struct InfoGrid {
    container: Grid,
    rows: HashMap<String, (Image, Label, Label, Label)>,
}

impl InfoGrid {
    pub fn new(info_keys: &[(String, String, String, String)]) -> Self {
        // info_keys: Vec<(id, label, icon_path)>
        let grid = Grid::new();
        grid.set_column_spacing(10);
        grid.set_row_spacing(4);
        grid.set_halign(Align::End);

        let mut rows = HashMap::new();

        for (i, (id, label_text, icon_text, icon_path)) in info_keys.iter().enumerate() {
            let icon = if icon_path.is_empty() { Image::new() } else { Image::from_file(icon_path) };
            icon.set_pixel_size(16);

            let icon_label = Label::new(Some(icon_text));
            icon_label.set_halign(Align::Start);

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
            container: grid,
            rows,
        }
    }

    pub fn widget(&self) -> &Grid {
        &self.container
    }

    pub fn update_value(&self, id: &str, new_value: &str) -> &Self {
        if let Some((_, _, _, value_label)) = self.rows.get(id) {
            value_label.set_text(new_value);
        }
        &self
    }

    pub fn update_path(&self, id: &str, new_icon_path: &str) {
        if let Some((icon, _, _, _)) = self.rows.get(id) {
            icon.set_from_file(Some(new_icon_path));
        }
    }

    pub fn update_color(&self, id: &str, color_css: &str) -> &Self {
        if let Some((_, _, _, value_label)) = self.rows.get(id) {
            value_label.set_markup(&format!(r#"<span foreground="{}">{}</span>"#, color_css, glib::markup_escape_text(&value_label.text())));
        }
        &self
    }
}
