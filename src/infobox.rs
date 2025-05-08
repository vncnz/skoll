use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Orientation, Image, Label, Scale, Adjustment,
    builders::{
        BoxBuilder, LabelBuilder, ListBoxBuilder, ScaleBuilder
    }
};
use std::rc::Rc;
pub struct InfoBox {
    pub container: GtkBox,
    data_column: GtkBox,
    rows: Vec<(Scale, Label)>,
}

impl InfoBox {
    pub fn new(icon_text: &str, data: Vec<(&str, f64, Option<String>)>) -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 8);
        // let icon = Image::from_file(icon_path);
        let icon = LabelBuilder::new()
            .label(icon_text)
            .margin(0)
            .halign(gtk::Align::Start)
            .build();
        container.add(&icon);
        icon.style_context().add_class("icon");
        // container.style_context().add_class("multirow");

        let data_column = GtkBox::new(Orientation::Vertical, 0);
        container.add(&data_column);

        let rows = Vec::new();
        let mut info_box = InfoBox {
            container,
            data_column,
            rows,
        };

        info_box.update_data(data);
        info_box.update_class_by_row_count();
        info_box
    }

    pub fn update_data(&mut self, new_data: Vec<(&str, f64, Option<String>)>) {
        // Se servono più righe, aggiungile
        while self.rows.len() < new_data.len() {
            let adjustment = Adjustment::new(0.0, 0.0, 100.0, 1.0, 10.0, 0.0);
            let scale = Scale::new(gtk::Orientation::Horizontal, Some(&adjustment));
            scale.set_draw_value(false);
            scale.set_hexpand(false);
    
            let label = Label::new(None);
            label.set_xalign(0.0);
    
            let row = GtkBox::new(Orientation::Horizontal, 0);
            row.add(&scale);
            row.add(&label);
            self.data_column.add(&row);
            self.rows.push((scale, label));
        }
    
        // Se ci sono troppe righe, rimuovi quelle in eccesso
        // TODO: fix this or add a check on vec length
        /* while self.rows.len() > new_data.len() {
            if let Some((scale, label)) = self.rows.pop() {
                if let Some(parent) = scale.parent() {
                    parent.unparent();
                }
                if let Some(parent) = label.parent() {
                    parent.unparent();
                }
            }
        } */
    
        // Ora aggiorna i testi e i valori delle righe esistenti
        for ((scale, label), (text, value, color)) in self.rows.iter().zip(new_data.iter()) {
            scale.set_value(*value);
            label.set_text(text);
            if let Some(color) = color { self.set_color_single(Some(&scale), Some(&label), &color) }
        }

        self.update_class_by_row_count();
    }
    

    pub fn set_color(&self, color: &str) {
        for (scale, label) in &self.rows {
            self.set_color_single(Some(&scale), Some(&label), color);
        }
    }

    pub fn set_color_single(&self, scale: Option<&Scale>, label: Option<&Label>, color: &str) {
        let css = format!(
            "
            scale highlight {{
                background-color: {};
            }}
            scale slider {{
                all: unset;
            }}
            label {{
                color: {};
            }}
            ", color, color);

        let provider = gtk::CssProvider::new();
        provider.load_from_data(css.as_bytes()).unwrap();

        if let Some(scale_) = scale { scale_.style_context().add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_USER) };
        if let Some(label_) = label { label_.style_context().add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_USER) };
    }

    pub fn update_class_by_row_count(&self) {
        let context = self.container.style_context();
        
        // Rimuovi tutte le classi che iniziano con "items-"
        for cls in context.list_classes() {
            if cls.starts_with("items-") {
                context.remove_class(&cls);
            }
        }
    
        // Aggiungi quella nuova
        let class_name = format!("items-{}", self.rows.len());
        context.add_class(&class_name);
    }
}
