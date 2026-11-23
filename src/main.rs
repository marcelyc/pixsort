mod files;

use eframe::egui::{self, ColorImage, Key, TextureHandle};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Default, Serialize, Deserialize)]
struct Settings {
    source: Option<PathBuf>,
    targets: [Option<PathBuf>; 9],
}

fn config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "Pixsort", "Pixsort")
        .map(|d| d.config_dir().join("settings.json"))
}
fn save_settings(s: &Settings) -> Result<(), String> {
    let path = config_path().ok_or("无法确定配置目录")?;
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::write(
        path,
        serde_json::to_vec_pretty(s).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
fn load_settings() -> Settings {
    config_path()
        .and_then(|p| fs::read(p).ok())
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}
struct MoveRecord {
    from: PathBuf,
    to: PathBuf,
}
struct App {
    settings: Settings,
    images: Vec<PathBuf>,
    index: usize,
    texture: Option<TextureHandle>,
    loaded: Option<PathBuf>,
    undo: Vec<MoveRecord>,
    counts: [Option<usize>; 9],
    target_inputs: [String; 9],
    message: String,
}
impl App {
    fn new() -> Self {
        let settings = load_settings();
        let target_inputs = std::array::from_fn(|i| {
            settings.targets[i]
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        });
        let mut app = Self {
            settings,
            images: Vec::new(),
            index: 0,
            texture: None,
            loaded: None,
            undo: Vec::new(),
            counts: [None; 9],
            target_inputs,
            message: String::new(),
        };
        app.refresh_source();
        app.refresh_counts();
        app
    }
    fn refresh_source(&mut self) {
        let current = self.images.get(self.index).cloned();
        self.images = self
            .settings
            .source
            .as_deref()
            .and_then(|p| files::images_in(p).ok())
            .unwrap_or_default();
        self.index = current
            .and_then(|p| self.images.iter().position(|x| *x == p))
            .unwrap_or(self.index.min(self.images.len().saturating_sub(1)));
        self.loaded = None;
    }
    fn refresh_counts(&mut self) {
        for i in 0..9 {
            self.counts[i] = self.settings.targets[i]
                .as_deref()
                .and_then(|p| files::count_images(p).ok());
        }
    }
    fn set_source(&mut self, p: PathBuf) {
        self.settings.source = Some(p);
        self.index = 0;
        self.refresh_source();
        if let Err(e) = save_settings(&self.settings) {
            self.message = e;
        }
    }
    fn set_target(&mut self, i: usize, p: PathBuf) {
        self.target_inputs[i] = p.display().to_string();
        self.settings.targets[i] = Some(p);
        self.refresh_counts();
        if let Err(e) = save_settings(&self.settings) {
            self.message = e;
        }
    }
    fn save_target_input(&mut self, i: usize) {
        let path = PathBuf::from(self.target_inputs[i].trim());
        if !path.is_absolute() || !path.is_dir() {
            self.message = format!("数字 {}：请输入已存在的绝对文件夹路径", i + 1);
            return;
        }
        self.set_target(i, path);
        self.message = format!("已保存数字 {} 的目标文件夹", i + 1);
    }
    fn move_current(&mut self, i: usize) {
        let Some(from) = self.images.get(self.index).cloned() else {
            return;
        };
        let Some(dir) = self.settings.targets[i].as_ref() else {
            self.message = format!("请先设置数字 {} 的文件夹", i + 1);
            return;
        };
        if !dir.is_dir() {
            self.message = format!("目标文件夹不存在：{}", dir.display());
            return;
        }
        if from.parent() == Some(dir.as_path()) {
            self.message = "图片已在目标文件夹中".into();
            return;
        }
        let Some(name) = from.file_name() else {
            return;
        };
        let to = match files::available_path(dir, name) {
            Ok(p) => p,
            Err(e) => {
                self.message = e.to_string();
                return;
            }
        };
        match files::move_file(&from, &to) {
            Ok(()) => {
                self.undo.push(MoveRecord {
                    from: from.clone(),
                    to,
                });
                self.images.remove(self.index);
                self.index = self.index.min(self.images.len().saturating_sub(1));
                self.loaded = None;
                self.refresh_counts();
                self.message = format!("已移动 {}", from.file_name().unwrap().to_string_lossy());
            }
            Err(e) => self.message = format!("移动失败：{e}"),
        }
    }
    fn undo(&mut self) {
        let Some(r) = self.undo.last() else {
            self.message = "没有可撤销的操作".into();
            return;
        };
        if r.from.exists() {
            self.message = "撤销失败：原路径已被占用".into();
            return;
        }
        if !r.to.exists() {
            self.message = "撤销失败：目标图片已不存在".into();
            return;
        }
        match files::move_file(&r.to, &r.from) {
            Ok(()) => {
                let r = self.undo.pop().unwrap();
                self.refresh_source();
                if let Some(pos) = self.images.iter().position(|p| *p == r.from) {
                    self.index = pos;
                }
                self.loaded = None;
                self.refresh_counts();
                self.message = "已撤销移动".into();
            }
            Err(e) => self.message = format!("撤销失败：{e}"),
        }
    }
    fn load_preview(&mut self, ctx: &egui::Context) {
        let path = self.images.get(self.index).cloned();
        if path == self.loaded {
            return;
        }
        self.loaded = path.clone();
        self.texture = None;
        if let Some(path) = path {
            match image::open(&path) {
                Ok(img) => {
                    let img = img
                        .resize(2400, 2400, image::imageops::FilterType::Triangle)
                        .to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];
                    let color = ColorImage::from_rgba_unmultiplied(size, &img.into_raw());
                    self.texture =
                        Some(ctx.load_texture("preview", color, egui::TextureOptions::LINEAR));
                }
                Err(e) => self.message = format!("无法读取图片：{e}"),
            }
        }
    }
}
impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::Z)) {
            self.undo();
        } else if !ctx.egui_wants_keyboard_input() {
            let keys = [
                Key::Num1,
                Key::Num2,
                Key::Num3,
                Key::Num4,
                Key::Num5,
                Key::Num6,
                Key::Num7,
                Key::Num8,
                Key::Num9,
            ];
            for (n, key) in keys.iter().enumerate() {
                if ctx.input(|i| i.key_pressed(*key) && !i.modifiers.command && !i.modifiers.alt) {
                    self.move_current(n);
                    break;
                }
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
                self.index = (self.index + 1).min(self.images.len().saturating_sub(1));
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
                self.index = self.index.saturating_sub(1);
            }
        }
        self.load_preview(&ctx);
        egui::Panel::top("top").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("选择待整理文件夹").clicked() {
                    if let Some(p) = rfd::FileDialog::new().pick_folder() {
                        self.set_source(p);
                    }
                }
                if ui.button("刷新").clicked() {
                    self.refresh_source();
                    self.refresh_counts();
                }
                if ui.button("撤销 Ctrl/⌘+Z").clicked() {
                    self.undo();
                }
            });
            ui.label(
                self.settings
                    .source
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "请先选择待整理文件夹".into()),
            );
        });
        egui::Panel::right("targets")
            .min_size(270.0)
            .show(ui, |ui| {
                ui.heading("分类文件夹");
                ui.label("按 1–9 移动当前图片");
                ui.separator();
                for i in 0..9 {
                    ui.horizontal(|ui| {
                        ui.strong(format!("{}", i + 1));
                        if ui.button("选择").clicked() {
                            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                self.set_target(i, p);
                            }
                        }
                        ui.label(
                            self.counts[i]
                                .map(|n| format!("{n} 张"))
                                .unwrap_or_else(|| "—".into()),
                        );
                    });
                    ui.horizontal(|ui| {
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.target_inputs[i])
                                .hint_text("输入文件夹绝对路径"),
                        );
                        let enter = response.lost_focus()
                            && ui.input(|input| input.key_pressed(Key::Enter));
                        if ui.button("保存").clicked() || enter {
                            self.save_target_input(i);
                        }
                    });
                    ui.separator();
                }
            });
        egui::Panel::bottom("status").show(ui, |ui| {
            ui.label(format!("{} 张图片 · {}", self.images.len(), self.message));
        });
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("◀ 上一张").clicked() {
                    self.index = self.index.saturating_sub(1);
                }
                ui.label(if self.images.is_empty() {
                    "0 / 0".into()
                } else {
                    format!("{} / {}", self.index + 1, self.images.len())
                });
                if ui.button("下一张 ▶").clicked() {
                    self.index = (self.index + 1).min(self.images.len().saturating_sub(1));
                }
            });
            if let Some(path) = self.images.get(self.index) {
                ui.label(path.file_name().unwrap().to_string_lossy());
            }
            ui.separator();
            if let Some(texture) = &self.texture {
                let available = ui.available_size();
                let size = texture.size_vec2();
                let scale = (available.x / size.x)
                    .min(available.y / size.y)
                    .min(1.0)
                    .max(0.01);
                ui.centered_and_justified(|ui| {
                    ui.image((texture.id(), size * scale));
                });
            } else if self.images.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("此文件夹没有 JPG、PNG 或 BMP 图片");
                });
            }
        });
    }
}
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Pixsort 图片分类")
            .with_inner_size([1100.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native("Pixsort", options, Box::new(|_| Ok(Box::new(App::new()))))
}
