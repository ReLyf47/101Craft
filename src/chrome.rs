//! Menu bar, toolbar, document tabs, find bar, status bar and the central view.

use egui::{Align, Color32, Key, Layout, RichText, Theme, Ui};

use crate::app::{App, BottomTab, RightTab};
use crate::doc::Column;
use crate::hexview::{self, Palette, ViewOptions};
use crate::template;
use crate::tools::{self, SearchKind};

impl App {
    pub fn menu_bar(&mut self, ui: &mut Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("Datei", |ui| {
                if ui.add(egui::Button::new("Neu").shortcut_text("Strg+N")).clicked() {
                    self.new_doc();
                    ui.close();
                }
                if ui.add(egui::Button::new("Öffnen…").shortcut_text("Strg+O")).clicked() {
                    ui.close();
                    self.open_dialog();
                }
                ui.separator();
                let has = !self.docs.is_empty();
                if ui.add_enabled(has, egui::Button::new("Speichern").shortcut_text("Strg+S")).clicked() {
                    ui.close();
                    self.save(self.active, false);
                }
                if ui
                    .add_enabled(has, egui::Button::new("Speichern unter…").shortcut_text("Strg+Umschalt+S"))
                    .clicked()
                {
                    ui.close();
                    self.save(self.active, true);
                }
                if ui.add_enabled(has, egui::Button::new("Schließen").shortcut_text("Strg+W")).clicked() {
                    ui.close();
                    self.close_doc(self.active, false);
                }
                ui.separator();
                if ui.button("Beenden").clicked() {
                    ui.close();
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button("Bearbeiten", |ui| {
                let (cu, cr) = self
                    .doc()
                    .map(|d| (d.can_undo(), d.can_redo()))
                    .unwrap_or((false, false));
                if ui.add_enabled(cu, egui::Button::new("Rückgängig").shortcut_text("Strg+Z")).clicked() {
                    self.undo();
                    ui.close();
                }
                if ui.add_enabled(cr, egui::Button::new("Wiederholen").shortcut_text("Strg+Y")).clicked() {
                    self.redo();
                    ui.close();
                }
                ui.separator();
                if ui.add(egui::Button::new("Als Hex kopieren").shortcut_text("Strg+C")).clicked() {
                    if let Some(d) = self.doc() {
                        let b = if d.selected_bytes().is_empty() {
                            d.data.get(d.cursor..d.cursor + 1).unwrap_or(&[])
                        } else {
                            d.selected_bytes()
                        };
                        ui.ctx().copy_text(tools::hex_spaced(b));
                    }
                    ui.close();
                }
                if ui.button("Als Text kopieren").clicked() {
                    if let Some(d) = self.doc() {
                        ui.ctx()
                            .copy_text(d.selected_bytes().iter().map(|x| *x as char).collect());
                    }
                    ui.close();
                }
                if ui.button("Als C-Array kopieren").clicked() {
                    if let Some(d) = self.doc() {
                        let b = d.selected_bytes();
                        let body = b
                            .chunks(12)
                            .map(|c| c.iter().map(|x| format!("0x{x:02X}")).collect::<Vec<_>>().join(", "))
                            .collect::<Vec<_>>()
                            .join(",\n    ");
                        ui.ctx()
                            .copy_text(format!("unsigned char data[{}] = {{\n    {body}\n}};", b.len()));
                    }
                    ui.close();
                }
                ui.label(RichText::new("Einfügen: Strg+V in der Hex-Ansicht").weak().small());
                ui.separator();
                if ui.add(egui::Button::new("Alles auswählen").shortcut_text("Strg+A")).clicked() {
                    if let Some(d) = self.doc_mut() {
                        let n = d.len();
                        if n > 0 {
                            d.anchor = Some(0);
                            d.cursor = n - 1;
                        }
                    }
                    ui.close();
                }
                if ui.add(egui::Button::new("Auswahl löschen").shortcut_text("Entf")).clicked() {
                    if let Some(d) = self.doc_mut() {
                        d.delete_forward();
                    }
                    ui.close();
                }
                if ui.button("Bytes einfügen…").clicked() {
                    self.insert_dialog = Some(("16".into(), "00".into()));
                    ui.close();
                }
                if ui.button("Auswahl füllen…").clicked() {
                    self.fill_dialog = Some("00".into());
                    ui.close();
                }
                ui.separator();
                let label = if self.insert {
                    "Zum Überschreibmodus wechseln"
                } else {
                    "Zum Einfügemodus wechseln"
                };
                if ui.add(egui::Button::new(label).shortcut_text("Einfg")).clicked() {
                    self.insert = !self.insert;
                    ui.close();
                }
            });
            ui.menu_button("Suchen", |ui| {
                if ui.add(egui::Button::new("Suchen…").shortcut_text("Strg+F")).clicked() {
                    self.open_find(false);
                    ui.close();
                }
                if ui.add(egui::Button::new("Ersetzen…").shortcut_text("Strg+H")).clicked() {
                    self.open_find(true);
                    ui.close();
                }
                if ui.add(egui::Button::new("Weitersuchen").shortcut_text("F3")).clicked() {
                    self.find_step(true);
                    ui.close();
                }
                if ui
                    .add(egui::Button::new("Rückwärts suchen").shortcut_text("Umschalt+F3"))
                    .clicked()
                {
                    self.find_step(false);
                    ui.close();
                }
                ui.separator();
                if ui.add(egui::Button::new("Gehe zu Adresse…").shortcut_text("Strg+G")).clicked() {
                    self.open_goto();
                    ui.close();
                }
                ui.separator();
                if ui
                    .add(egui::Button::new("Lesezeichen setzen/entfernen").shortcut_text("Strg+B"))
                    .clicked()
                {
                    self.toggle_bookmark();
                    ui.close();
                }
                if ui.add(egui::Button::new("Nächstes Lesezeichen").shortcut_text("F2")).clicked() {
                    self.next_bookmark(true);
                    ui.close();
                }
                if ui
                    .add(egui::Button::new("Vorheriges Lesezeichen").shortcut_text("Umschalt+F2"))
                    .clicked()
                {
                    self.next_bookmark(false);
                    ui.close();
                }
            });
            ui.menu_button("Ansicht", |ui| {
                if let Some(d) = self.doc_mut() {
                    let mut t = d.text_mode;
                    if ui.checkbox(&mut t, "Als Text bearbeiten").clicked() {
                        d.text_mode = t;
                        d.text_dirty_from_data = true;
                        ui.close();
                    }
                }
                ui.separator();
                ui.label("Bytes pro Zeile");
                for n in [8usize, 16, 24, 32] {
                    ui.radio_value(&mut self.bytes_per_row, n, format!("{n}"));
                }
                ui.separator();
                ui.label("Byte-Reihenfolge");
                ui.radio_value(&mut self.little_endian, true, "Little Endian (Intel)");
                ui.radio_value(&mut self.little_endian, false, "Big Endian (Motorola)");
                ui.separator();
                ui.label("Schriftgröße");
                ui.add(egui::Slider::new(&mut self.font_size, 10.0..=20.0).step_by(1.0));
                ui.separator();
                if ui.checkbox(&mut self.dark, "Dunkles Design").clicked() {
                    ui.ctx().set_theme(if self.dark { Theme::Dark } else { Theme::Light });
                }
            });
            ui.menu_button("Vorlagen", |ui| {
                ui.checkbox(&mut self.auto_template, "Beim Öffnen automatisch anwenden");
                if ui
                    .add(egui::Button::new("Vorlage erneut ausführen").shortcut_text("F5"))
                    .clicked()
                {
                    self.rerun_or_detect();
                    ui.close();
                }
                ui.separator();
                for b in template::BUILTINS {
                    if ui.button(format!("{} anwenden", b.name)).clicked() {
                        self.run_template(self.active, b.name, b.source);
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("Vorlagen-Editor…").clicked() {
                    self.tpl_editor.open = true;
                    ui.close();
                }
                if ui.button("Vorlage aus Datei ausführen…").clicked() {
                    ui.close();
                    self.run_template_file();
                }
                if ui.button("Vorlagenergebnis entfernen").clicked() {
                    if let Some(d) = self.doc_mut() {
                        d.template = None;
                        d.highlights.clear();
                    }
                    ui.close();
                }
            });
            ui.menu_button("Werkzeuge", |ui| {
                if ui.button("Prüfsumme / Hash…").clicked() {
                    self.right_tab = RightTab::Checksum;
                    ui.close();
                }
                if ui.button("Dateien vergleichen…").clicked() {
                    self.open_compare_dialog();
                    ui.close();
                }
                if ui.button("Histogramm").clicked() {
                    self.bottom_tab = BottomTab::Histogram;
                    ui.close();
                }
                if self.compare.pair.is_some() && ui.button("Vergleich beenden").clicked() {
                    self.end_compare();
                    ui.close();
                }
            });
            ui.menu_button("Hilfe", |ui| {
                if ui.button("Tastenkürzel").clicked() {
                    self.keys_open = true;
                    ui.close();
                }
                if ui.button("Über 101Craft").clicked() {
                    self.about_open = true;
                    ui.close();
                }
            });
        });
    }

    pub fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 3.0;
            let b = |ui: &mut Ui, icon: &str, tip: &str| {
                ui.add(
                    egui::Button::new(RichText::new(icon).size(15.0))
                        .frame(false)
                        .min_size(egui::vec2(28.0, 26.0)),
                )
                .on_hover_text(tip)
            };
            if b(ui, "📄", "Neue Datei (Strg+N)").clicked() {
                self.new_doc();
            }
            if b(ui, "📂", "Öffnen (Strg+O)").clicked() {
                self.open_dialog();
            }
            if b(ui, "💾", "Speichern (Strg+S)").clicked() {
                self.save(self.active, false);
            }
            ui.separator();
            if b(ui, "⟲", "Rückgängig (Strg+Z)").clicked() {
                self.undo();
            }
            if b(ui, "⟳", "Wiederholen (Strg+Y)").clicked() {
                self.redo();
            }
            ui.separator();
            if b(ui, "🔍", "Suchen (Strg+F)").clicked() {
                self.open_find(false);
            }
            if b(ui, "⇄", "Ersetzen (Strg+H)").clicked() {
                self.open_find(true);
            }
            if b(ui, "📍", "Gehe zu Adresse (Strg+G)").clicked() {
                self.open_goto();
            }
            if b(ui, "🔖", "Lesezeichen setzen (Strg+B)").clicked() {
                self.toggle_bookmark();
            }
            ui.separator();
            if b(ui, "▶", "Vorlage ausführen (F5)").clicked() {
                self.rerun_or_detect();
            }
            if b(ui, "✏", "Vorlagen-Editor").clicked() {
                self.tpl_editor.open = true;
            }
            if b(ui, "#", "Prüfsumme / Hash").clicked() {
                self.right_tab = RightTab::Checksum;
            }
            if b(ui, "⚖", "Dateien vergleichen").clicked() {
                self.open_compare_dialog();
            }
            if b(ui, "📊", "Histogramm").clicked() {
                self.bottom_tab = BottomTab::Histogram;
            }
            ui.separator();
            ui.label(RichText::new("Bytes/Zeile").weak());
            egui::ComboBox::from_id_salt("bpr")
                .width(48.0)
                .selected_text(format!("{}", self.bytes_per_row))
                .show_ui(ui, |ui| {
                    for n in [8usize, 16, 24, 32] {
                        ui.selectable_value(&mut self.bytes_per_row, n, format!("{n}"));
                    }
                });
            ui.label(RichText::new("Endian").weak());
            egui::ComboBox::from_id_salt("endian")
                .width(62.0)
                .selected_text(if self.little_endian { "Little" } else { "Big" })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.little_endian, true, "Little");
                    ui.selectable_value(&mut self.little_endian, false, "Big");
                });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let mode = if self.insert { "EINF" } else { "ÜBR" };
                if ui
                    .add(egui::Button::new(RichText::new(mode).monospace().strong()).min_size(egui::vec2(46.0, 22.0)))
                    .on_hover_text("Einfüge-/Überschreibmodus umschalten (Einfg)")
                    .clicked()
                {
                    self.insert = !self.insert;
                }
                ui.label(RichText::new("101Craft").strong().color(Color32::from_rgb(45, 120, 215)));
            });
        });
    }

    fn doc_tabs(&mut self, ui: &mut Ui) {
        let mut close = None;
        let mut activate = None;
        egui::ScrollArea::horizontal()
            .id_salt("tabs")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    for (i, d) in self.docs.iter().enumerate() {
                        let sel = i == self.active;
                        let v = ui.visuals();
                        let frame = egui::Frame::new()
                            .fill(if sel { v.extreme_bg_color } else { v.faint_bg_color })
                            .stroke(egui::Stroke::new(
                                1.0,
                                if sel { v.selection.bg_fill } else { v.widgets.noninteractive.bg_stroke.color },
                            ))
                            .corner_radius(egui::CornerRadius { nw: 5, ne: 5, sw: 0, se: 0 })
                            .inner_margin(egui::Margin::symmetric(8, 3));
                        frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let icon = if d.template.is_some() { "◆ " } else { "◇ " };
                                let t = RichText::new(format!("{icon}{}", d.title()));
                                let t = if sel { t.strong() } else { t };
                                let tip = d
                                    .path
                                    .as_ref()
                                    .map(|p| p.display().to_string())
                                    .unwrap_or_else(|| "Nicht gespeichert".into());
                                if ui
                                    .add(egui::Label::new(t).sense(egui::Sense::click()))
                                    .on_hover_text(tip)
                                    .clicked()
                                {
                                    activate = Some(i);
                                }
                                if ui
                                    .add(egui::Button::new(RichText::new("×").size(13.0)).frame(false))
                                    .on_hover_text("Schließen")
                                    .clicked()
                                {
                                    close = Some(i);
                                }
                            });
                        });
                    }
                });
            });
        if let Some(i) = activate {
            self.active = i;
            self.focus_hex = true;
        }
        if let Some(i) = close {
            self.close_doc(i, false);
        }
    }

    fn find_bar(&mut self, ui: &mut Ui) {
        let mut close = false;
        egui::Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(egui::Margin::symmetric(6, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Suchen:");
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut self.find.text)
                            .desired_width(200.0)
                            .hint_text("Text oder Hex, z. B. 42 4D"),
                    );
                    if self.find.focus {
                        r.request_focus();
                        self.find.focus = false;
                    }
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                    egui::ComboBox::from_id_salt("findkind")
                        .selected_text(self.find.kind.label())
                        .show_ui(ui, |ui| {
                            for k in [SearchKind::Text, SearchKind::TextUtf16, SearchKind::Hex, SearchKind::Int32] {
                                ui.selectable_value(&mut self.find.kind, k, k.label());
                            }
                        });
                    if self.find.kind == SearchKind::Text {
                        ui.checkbox(&mut self.find.ignore_case, "Groß/klein egal");
                    }
                    if ui.button("◀").on_hover_text("Rückwärts (Umschalt+F3)").clicked() {
                        self.find_step(false);
                    }
                    if ui.button("Weiter ▶").on_hover_text("Weitersuchen (F3)").clicked() || enter {
                        self.find_step(true);
                        if enter {
                            r.request_focus();
                        }
                    }
                    if ui.button("Alle finden").clicked() {
                        self.find_all();
                    }
                    ui.label(RichText::new(&self.find.message).weak());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(egui::Button::new("✕").frame(false)).on_hover_text("Schließen").clicked() {
                            close = true;
                        }
                        if ui.selectable_label(self.find.replace_mode, "Ersetzen…").clicked() {
                            self.find.replace_mode = !self.find.replace_mode;
                        }
                    });
                });
                if self.find.replace_mode {
                    ui.horizontal(|ui| {
                        ui.label("Ersetzen:");
                        ui.add(egui::TextEdit::singleline(&mut self.find.replace).desired_width(200.0));
                        if ui.button("Ersetzen").clicked() {
                            self.replace_one();
                        }
                        if ui.button("Alle ersetzen").clicked() {
                            self.replace_all();
                        }
                    });
                }
            });
        if close {
            self.find.open = false;
            if let Some(d) = self.doc_mut() {
                d.hits.clear();
            }
            self.focus_hex = true;
        }
    }

    pub fn status_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let Some(d) = self.docs.get(self.active) else {
                ui.label("Keine Datei geöffnet");
                return;
            };
            let c = d.cursor;
            ui.label(RichText::new(format!("Pos: 0x{c:X} ({c})")).monospace());
            ui.separator();
            if let Some(b) = d.data.get(c) {
                let ch = if (0x20..0x7f).contains(b) {
                    format!(" '{}'", *b as char)
                } else {
                    String::new()
                };
                ui.label(RichText::new(format!("Wert: 0x{b:02X} = {b}{ch}")).monospace());
                ui.separator();
            }
            if let Some((s, e)) = d.selection() {
                ui.label(
                    RichText::new(format!("Auswahl: 0x{s:X}–0x{:X} ({} Bytes)", e - 1, e - s)).monospace(),
                );
                ui.separator();
            }
            ui.label(format!("Größe: {}", tools::human_size(d.len())));
            if let Some(t) = &d.template {
                if let Some(last) = t.path_at(c).last() {
                    ui.separator();
                    ui.label(
                        RichText::new(format!("Feld: {}", last.trim_start_matches('/').replace('/', ".")))
                            .monospace(),
                    );
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(if self.insert { "EINF" } else { "ÜBR" }).monospace().strong());
                ui.separator();
                ui.label(if self.little_endian { "LE" } else { "BE" });
                ui.separator();
                ui.label(match d.column {
                    Column::Hex => "Hex",
                    Column::Text => "Text",
                });
                if d.modified {
                    ui.separator();
                    ui.label(RichText::new("geändert").color(Color32::from_rgb(220, 120, 0)));
                }
            });
        });
    }

    pub fn central(&mut self, ui: &mut Ui) {
        if self.docs.is_empty() {
            self.hex_focused = false;
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 3.0);
                ui.heading("101Craft");
                ui.label("Keine Datei geöffnet. Datei hierher ziehen oder öffnen.");
                ui.add_space(8.0);
                if ui.button("📂 Datei öffnen…").clicked() {
                    self.open_dialog();
                }
                if ui.button("📄 Neue leere Datei").clicked() {
                    self.new_doc();
                }
            });
            return;
        }
        self.doc_tabs(ui);
        if self.find.open {
            self.find_bar(ui);
        }
        let pal = Palette::new(self.dark);
        let opts = ViewOptions {
            bytes_per_row: self.bytes_per_row,
            font_size: self.font_size,
        };
        // Side-by-side compare view.
        if let (Some((a, b)), true) = (self.compare.pair, self.compare.split) {
            if a < self.docs.len() && b < self.docs.len() && (self.active == a || self.active == b) {
                self.compare_view(ui, a, b, &opts, &pal);
                return;
            }
        }
        let idx = self.active;
        if self.docs[idx].text_mode {
            self.text_view(ui, idx);
            self.hex_focused = false;
            return;
        }
        let id = egui::Id::unique("hexview");
        if self.focus_hex {
            ui.memory_mut(|m| m.request_focus(id));
            self.focus_hex = false;
        }
        let r = hexview::hex_view(ui, &mut self.docs[idx], id, &opts, self.insert, &pal);
        self.hex_focused = r.focused;
        self.hovered_byte = r.hovered_byte;
        if r.toggle_insert {
            self.insert = !self.insert;
        }
    }

    fn compare_view(&mut self, ui: &mut Ui, a: usize, b: usize, opts: &ViewOptions, pal: &Palette) {
        let w = ui.available_width();
        let h = ui.available_height();
        let mut res = Vec::new();
        ui.horizontal(|ui| {
            for (k, idx) in [a, b].into_iter().enumerate() {
                ui.allocate_ui(egui::vec2(w / 2.0 - 4.0, h), |ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(format!("{} {}", if k == 0 { "A:" } else { "B:" }, self.docs[idx].name)).strong());
                        let id = egui::Id::unique(("cmpview", k));
                        let r = hexview::hex_view(ui, &mut self.docs[idx], id, opts, self.insert, pal);
                        res.push((idx, r));
                    });
                });
            }
        });
        // Keep both sides scrolled together.
        let (ta, tb) = (self.docs[a].top_row, self.docs[b].top_row);
        if ta != self.compare.last_top {
            self.docs[b].top_row = ta;
            self.compare.last_top = ta;
        } else if tb != self.compare.last_top {
            self.docs[a].top_row = tb;
            self.compare.last_top = tb;
        }
        self.hex_focused = false;
        for (idx, r) in res {
            if r.focused {
                self.active = idx;
                self.hex_focused = true;
            }
            if r.toggle_insert {
                self.insert = !self.insert;
            }
        }
    }

    fn text_view(&mut self, ui: &mut Ui, idx: usize) {
        let font = egui::FontId::monospace(self.font_size);
        let d = &mut self.docs[idx];
        if d.len() > 8 * 1024 * 1024 {
            ui.label("Die Datei ist für die Textansicht zu groß (über 8 MB).");
            return;
        }
        let valid = std::str::from_utf8(&d.data).is_ok();
        if d.text_dirty_from_data {
            d.text_buf = String::from_utf8_lossy(&d.data).into_owned();
            d.text_dirty_from_data = false;
        }
        if !valid {
            ui.label(
                RichText::new("Die Datei enthält Bytes, die kein gültiger UTF-8-Text sind – die Textansicht ist schreibgeschützt.")
                    .color(Color32::from_rgb(200, 110, 0)),
            );
        }
        egui::ScrollArea::both()
            .id_salt("textview")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut buf = d.text_buf.clone();
                let r = ui.add_enabled(
                    valid,
                    egui::TextEdit::multiline(&mut buf)
                        .font(font)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(40),
                );
                if r.changed() && valid {
                    let len = d.data.len();
                    let new = buf.clone().into_bytes();
                    d.splice(0, len, &new);
                    d.text_buf = buf;
                    d.text_dirty_from_data = false;
                }
            });
    }
}
