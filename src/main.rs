use eframe::{egui, App, Frame};
use egui::TextureHandle;
use image::ImageReader as ImageReader;
use std::{collections::HashMap, fs, path::PathBuf, process::Command};

use std::path::Path;

struct AppEntry {
    name: String,
    description: String,
    exec: String,
    icon_name: String,
}

struct LauncherApp {
    filter: String,
    apps: Vec<AppEntry>,
    icons: HashMap<String, TextureHandle>,
    first_frame: bool
}

impl LauncherApp {
    fn load_apps() -> Vec<AppEntry> {
        let mut entries = Vec::new();
        let paths = vec![
            dirs::data_dir().unwrap().join("applications"),
            PathBuf::from("/usr/share/applications"),
        ];
    
        for dir in paths {
            if let Ok(read_dir) = fs::read_dir(dir) {
                for entry in read_dir.flatten() {
                    if let Some(app) = parse_desktop_file(&entry.path()) {
                        entries.push(app);
                    }
                }
            }
        }
    
        entries
    }

    fn resolve_icon(icon_name: &str) -> Option<PathBuf> {
        let exts = ["png", "svg", "xpm"];
        let dirs = vec![
            "/usr/share/icons/hicolor/48x48/apps/",
            "/usr/share/icons/hicolor/64x64/apps/",
            "/usr/share/icons/hicolor/scalable/apps/",
            "/usr/share/pixmaps/",
        ];

        for dir in dirs {
            for ext in &exts {
                let path = PathBuf::from(format!("{dir}{icon_name}.{ext}"));
                if path.exists() {
                    return Some(path);
                }
            }
        }

        None
    }

    fn load_icons(ctx: &egui::Context, apps: &[AppEntry]) -> HashMap<String, TextureHandle> {
        let mut map = HashMap::new();

        for app in apps {
            if map.contains_key(&app.icon_name) || app.icon_name.is_empty() {
                continue;
            }

            if let Some(path) = Self::resolve_icon(&app.icon_name) {
                if let Ok(Ok(img)) = ImageReader::open(path).and_then(|r| Ok(r.decode())) {
                    let size = [img.width() as usize, img.height() as usize];
                    let rgba = img.to_rgba8();
                    let pixels = rgba.into_vec();
                    let tex = ctx.load_texture(
                        app.icon_name.clone(),
                        egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
                        egui::TextureOptions::LINEAR,
                    );
                    map.insert(app.icon_name.clone(), tex);
                }
            }
        }

        map
    }

    fn filtered_apps(&self) -> Vec<&AppEntry> {
        if self.filter.trim().is_empty() {
            self.apps.iter().collect()
        } else {
            self.apps
                .iter()
                .filter(|a| a.name.to_lowercase().contains(&self.filter.to_lowercase()))
                .collect()
        }
    }

    fn launch_app(&self, app: &AppEntry) {
        let command = app.exec.replace("%u", "").replace("%f", "");
        let mut parts = command.split_whitespace();
        if let Some(cmd) = parts.next() {
            let args: Vec<&str> = parts.collect();
            let _ = Command::new(cmd).args(args).spawn();
        }
    }
}

impl App for LauncherApp {
    /* fn name(&self) -> &str {
        "egui-launcher"
    } */

    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        let search_id = egui::Id::new("search");
        if self.first_frame {
            ctx.set_visuals(egui::Visuals {
                window_fill: egui::Color32::RED, // egui::Color32::from_rgba_premultiplied(110, 10, 10, 200),
                ..Default::default()
            });
    
            self.icons = Self::load_icons(ctx, &self.apps);
            self.first_frame = false;
            
            /*{
                frame.
                
                let mut memory = ui.memory();
                if memory.focus() == None {
                    memory.request_focus(button.id);
                }                      
            }*/
            
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            // frame.close();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            ui.heading("Launcher");
            ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("Cerca app...").id(search_id));
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for app in self.filtered_apps() {
                    let response = ui.horizontal(|ui| {
                        if let Some(icon) = self.icons.get(&app.icon_name) {
                            ui.image(icon); // , egui::vec2(32.0, 32.0));
                        } else {
                            ui.label("[ ]");
                        }

                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(&app.name).strong());
                            if !app.description.is_empty() {
                                ui.label(&app.description);
                            }
                        });
                    });

                    /* if response.inner.response.clicked() {
                        self.launch_app(app);
                        // frame.close();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        return;
                    } */

                    ui.add_space(5.0);
                }
            });
        });
    }
}

fn parse_desktop_file(path: &Path) -> Option<AppEntry> {
    let content = fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut description = None;
    let mut exec = None;
    let mut icon = None;
    let mut nodisplay = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("Name=") {
            name = Some(line[5..].to_string());
        } else if line.starts_with("Comment=") {
            description = Some(line[8..].to_string());
        } else if line.starts_with("Exec=") {
            exec = Some(line[5..].to_string());
        } else if line.starts_with("Icon=") {
            icon = Some(line[5..].to_string());
        } else if line.starts_with("NoDisplay=") && line.contains("true") {
            nodisplay = true;
        }
    }

    if nodisplay || name.is_none() || exec.is_none() {
        return None;
    }

    Some(AppEntry {
        name: name.unwrap(),
        description: description.unwrap_or_default(),
        exec: exec.unwrap(),
        icon_name: icon.unwrap_or_default(),
    })
}

fn main() -> eframe::Result<()> {
    let apps = LauncherApp::load_apps();

    let options = eframe::NativeOptions {
        // Impostazione fullscreen (non sempre rispettata da tutti i WM, specialmente su Wayland)
        run_and_return: true,
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top(),
        ..Default::default()
    };

    eframe::run_native(
        "Launcher",
        options,
        Box::new(|_cc| {
            Box::new(LauncherApp {
                filter: String::new(),
                apps,
                icons: HashMap::new(),
                first_frame: true
            })
        }),
    )
}
