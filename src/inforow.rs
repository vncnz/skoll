use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Orientation, Image, Label, Scale, Adjustment,
    builders::{
        BoxBuilder, LabelBuilder, ListBoxBuilder, ScaleBuilder
    }
};

pub struct InfoRow {
    pub container: gtk::Box,
    // icon: gtk::Image,
    icon: gtk::Label,
    label: gtk::Label,
    value: gtk::Label,
}

impl InfoRow {
    pub fn new(icon_text: &str, label_text: &str, value_text: &str) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        
        // let icon = gtk::Image::from_icon_name(Some(icon_name));
        let icon = LabelBuilder::new()
            .label(icon_text)
            .margin(10)
            .halign(gtk::Align::Start)
            .build();
        // icon.set_pixel_size(16);

        let label = gtk::Label::new(Some(label_text));
        // label.set_xalign(0.0);

        let value = gtk::Label::new(Some(value_text));
        // value.set_xalign(1.0);
        // value.set_halign(gtk::Align::End);

        container.add(&icon);
        container.add(&label);
        container.add(&value);

        Self { container, icon, label, value }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.container
    }

    pub fn set_value(&self, text: &str) {
        self.value.set_text(text);
    }

    pub fn set_label(&self, text: &str) {
        self.label.set_text(text);
    }

    pub fn set_icon(&self, icon_name: &str) {
        // self.icon.set_from_icon_name(Some(icon_name));
        self.icon.set_text(icon_name);
    }

    pub fn set_color_class(&self, class: &str) {
        self.value.style_context().remove_class("ok");
        self.value.style_context().remove_class("warn");
        self.value.style_context().remove_class("crit");
        self.value.style_context().add_class(class);
    }
}
