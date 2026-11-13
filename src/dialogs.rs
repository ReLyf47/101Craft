//! Small windows: go to address, compare, template editor, insert/fill, confirmations.

use egui::{Align2, Key, RichText};

use crate::app::App;
use crate::template;
use crate::tools;

impl App {
    pub fn dialogs(&mut self, ctx: &egui::Context) {
        let center = Align2::CENTER_CENTER;
        if self.goto_open {
            let mut open = true;
            let mut go = false;
            egui::Window::new("Gehe zu Adresse")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(center, [0.0, -120.0])
                .show(ctx, |ui| {
                    ui.label("0x1A0 oder 1A0h (hex), 416 (dezimal), +16 / -16 (relativ)");
                    let r = ui.add(egui::TextEdit::singleline(&mut self.goto_text).desired_width(260.0));
                    if self.goto_focus {
                        r.request_focus();
                        self.goto_focus = false;
                    }
                    if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                        go = true;
                    }
                    if ui.button("Springen").clicked() {
                        go = true;
                    }
                });
            if go {
                let txt = self.goto_text.clone();
                if let Some(d) = self.doc_mut() {
                    match tools::parse_offset(&txt, d.cursor, false) {
                        Some(p) => {
                            let p = p.min(d.len());
                            d.set_cursor(p, false);
                            self.goto_open = false;
                            self.focus_hex = true;
                        }
                        None => self.status = "Ungültige Adresse".into(),
                    }
                }
            }
            if !open {
                self.goto_open = false;
            }
        }

        if self.compare.open {
            let mut open = true;
            let mut run = false;
            let mut open_more = false;
            egui::Window::new("Dateien vergleichen")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(center, [0.0, -80.0])
                .show(ctx, |ui| {
                    if self.docs.len() < 2 {
                        ui.label("Bitte zuerst zwei Dateien öffnen.");
                        if ui.button("📂 Weitere Datei öffnen…").clicked() {
                            open_more = true;
                        }
                        return;
                    }
                    let n = self.docs.len();
                    self.compare.a = self.compare.a.min(n - 1);
                    self.compare.b = self.compare.b.min(n - 1);
                    egui::Grid::new("cmpgrid").num_columns(2).show(ui, |ui| {
                        ui.label("Datei A:");
                        egui::ComboBox::from_id_salt("cmpa")
                            .width(260.0)
                            .selected_text(self.docs[self.compare.a].name.clone())
                            .show_ui(ui, |ui| {
                                for (i, d) in self.docs.iter().enumerate() {
                                    ui.selectable_value(&mut self.compare.a, i, &d.name);
                                }
                            });
                        ui.end_row();
                        ui.label("Datei B:");
                        egui::ComboBox::from_id_salt("cmpb")
                            .width(260.0)
                            .selected_text(self.docs[self.compare.b].name.clone())
                            .show_ui(ui, |ui| {
                                for (i, d) in self.docs.iter().enumerate() {
                                    ui.selectable_value(&mut self.compare.b, i, &d.name);
                                }
                            });
                        ui.end_row();
                    });
                    ui.checkbox(&mut self.compare.split, "Nebeneinander anzeigen");
                    ui.label(RichText::new("Byteweiser Vergleich an gleicher Position.").weak());
                    if ui.button("Vergleichen").clicked() {
                        run = true;
                    }
                });
            if open_more {
                self.open_dialog();
            }
            if run {
                self.run_compare();
            }
            if !open {
                self.compare.open = false;
            }
        }

        if self.tpl_editor.open {
            self.template_editor(ctx);
        }

        if let Some((count, val)) = self.insert_dialog.clone() {
            let mut keep = true;
            let mut apply = false;
            let (mut c, mut v) = (count, val);
            egui::Window::new("Bytes einfügen")
                .collapsible(false)
                .resizable(false)
                .anchor(center, [0.0, -80.0])
                .show(ctx, |ui| {
                    egui::Grid::new("ins").show(ui, |ui| {
                        ui.label("Anzahl:");
                        ui.text_edit_singleline(&mut c);
                        ui.end_row();
                        ui.label("Wert (Hex):");
                        ui.text_edit_singleline(&mut v);
                        ui.end_row();
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Einfügen").clicked() {
                            apply = true;
                        }
                        if ui.button("Abbrechen").clicked() {
                            keep = false;
                        }
                    });
                });
            if apply {
                let n = tools::parse_offset(&c, 0, false);
                match (n, tools::parse_hex(&v)) {
                    (Some(n), Some(p)) if n > 0 && n < 512 * 1024 * 1024 => {
                        let bytes: Vec<u8> = p.iter().cycle().take(n).copied().collect();
                        if let Some(d) = self.doc_mut() {
                            let pos = d.cursor;
                            d.splice(pos, 0, &bytes);
                        }
                        keep = false;
                    }
                    _ => self.status = "Ungültige Eingabe".into(),
                }
            }
            self.insert_dialog = if keep { Some((c, v)) } else { None };
        }

        if let Some(val) = self.fill_dialog.clone() {
            let mut keep = true;
            let mut apply = false;
            let mut v = val;
            egui::Window::new("Auswahl füllen")
                .collapsible(false)
                .resizable(false)
                .anchor(center, [0.0, -80.0])
                .show(ctx, |ui| {
                    ui.label("Muster (Hex), wird über die Auswahl wiederholt:");
                    ui.text_edit_singleline(&mut v);
                    ui.horizontal(|ui| {
                        if ui.button("Füllen").clicked() {
                            apply = true;
                        }
                        if ui.button("Abbrechen").clicked() {
                            keep = false;
                        }
                    });
                });
            if apply {
                if let Some(p) = tools::parse_hex(&v) {
                    if let Some(d) = self.doc_mut() {
                        if let Some((s, e)) = d.selection() {
                            let bytes: Vec<u8> = p.iter().cycle().take(e - s).copied().collect();
                            d.splice(s, e - s, &bytes);
                            d.anchor = Some(s);
                            d.cursor = e - 1;
                        }
                    }
                    keep = false;
                } else {
                    self.status = "Ungültiges Muster".into();
                }
            }
            self.fill_dialog = if keep { Some(v) } else { None };
        }

        if let Some(idx) = self.confirm_close {
            let name = self.docs.get(idx).map(|d| d.name.clone()).unwrap_or_default();
            let mut choice = None;
            egui::Window::new("Ungespeicherte Änderungen")
                .collapsible(false)
                .resizable(false)
                .anchor(center, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(format!("'{name}' wurde geändert. Änderungen speichern?"));
                    ui.horizontal(|ui| {
                        if ui.button("Speichern").clicked() {
                            choice = Some(0);
                        }
                        if ui.button("Nicht speichern").clicked() {
                            choice = Some(1);
                        }
                        if ui.button("Abbrechen").clicked() {
                            choice = Some(2);
                        }
                    });
                });
            match choice {
                Some(0) => {
                    if self.save(idx, false) {
                        self.close_doc(idx, true);
                    }
                    self.confirm_close = None;
                }
                Some(1) => {
                    self.close_doc(idx, true);
                    self.confirm_close = None;
                }
                Some(_) => self.confirm_close = None,
                None => {}
            }
        }

        if self.confirm_quit {
            let mut choice = None;
            let n = self.docs.iter().filter(|d| d.modified).count();
            egui::Window::new("101Craft beenden")
                .collapsible(false)
                .resizable(false)
                .anchor(center, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(format!("{n} Datei(en) haben ungespeicherte Änderungen. Trotzdem beenden?"));
                    ui.horizontal(|ui| {
                        if ui.button("Beenden ohne Speichern").clicked() {
                            choice = Some(true);
                        }
                        if ui.button("Abbrechen").clicked() {
                            choice = Some(false);
                        }
                    });
                });
            match choice {
                Some(true) => {
                    self.quitting = true;
                    self.confirm_quit = false;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                Some(false) => self.confirm_quit = false,
                None => {}
            }
        }

        if self.about_open {
            egui::Window::new("Über 101Craft")
                .open(&mut self.about_open)
                .collapsible(false)
                .resizable(false)
                .anchor(center, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("101Craft");
                    ui.label("Hex- und Binär-Editor mit Vorlagen.");
                    ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                    ui.add_space(6.0);
                    ui.label("Bytes ansehen und bearbeiten, Dateiformate mit Vorlagen zerlegen, suchen, vergleichen und Prüfsummen berechnen.");
                });
        }
        if self.keys_open {
            egui::Window::new("Tastenkürzel")
                .open(&mut self.keys_open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    egui::Grid::new("keys").striped(true).show(ui, |ui| {
                        for (k, v) in [
                            ("Strg+O / Strg+S", "Öffnen / Speichern"),
                            ("Strg+Z / Strg+Y", "Rückgängig / Wiederholen"),
                            ("Strg+C / Strg+X / Strg+V", "Kopieren / Ausschneiden / Einfügen"),
                            ("Strg+A", "Alles auswählen"),
                            ("Strg+F / Strg+H", "Suchen / Ersetzen"),
                            ("F3 / Umschalt+F3", "Weiter / rückwärts suchen"),
                            ("Strg+G", "Gehe zu Adresse"),
                            ("Strg+B / F2", "Lesezeichen setzen / nächstes"),
                            ("F5", "Vorlage ausführen"),
                            ("Einfg", "Einfügen / Überschreiben"),
                            ("Tab", "Zwischen Hex- und Textspalte wechseln"),
                            ("Umschalt+Pfeile", "Auswahl erweitern"),
                            ("Strg+Tab", "Nächste Datei"),
                        ] {
                            ui.label(RichText::new(k).monospace());
                            ui.label(v);
                            ui.end_row();
                        }
                    });
                });
        }
    }

    fn template_editor(&mut self, ctx: &egui::Context) {
        let mut open = true;
        let mut run = false;
        let mut save = false;
        let mut load = false;
        egui::Window::new("Vorlagen-Editor")
            .open(&mut open)
            .default_size([560.0, 460.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.tpl_editor.name);
                    egui::ComboBox::from_id_salt("tplload")
                        .selected_text("Beispiel laden…")
                        .show_ui(ui, |ui| {
                            for b in template::BUILTINS {
                                if ui.selectable_label(false, b.name).clicked() {
                                    self.tpl_editor.name = b.name.to_string();
                                    self.tpl_editor.source = b.source.to_string();
                                }
                            }
                        });
                });
                ui.horizontal(|ui| {
                    if ui.button("▶ Auf aktuelle Datei ausführen").clicked() {
                        run = true;
                    }
                    if ui.button("📂 Laden…").clicked() {
                        load = true;
                    }
                    if ui.button("💾 Speichern…").clicked() {
                        save = true;
                    }
                });
                ui.label(
                    RichText::new("C-ähnliche Sprache: struct, typedef, enum, if/while/for, local-Variablen, FTell(), FSeek(), ReadUInt(), Printf() …")
                        .weak()
                        .small(),
                );
                egui::ScrollArea::vertical().id_salt("tplsrc").show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.tpl_editor.source)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .desired_rows(24),
                    );
                });
            });
        if run {
            let (n, s) = (self.tpl_editor.name.clone(), self.tpl_editor.source.clone());
            self.run_template(self.active, &n, &s);
        }
        if load {
            self.run_template_file();
        }
        if save {
            if let Some(p) = rfd::FileDialog::new()
                .set_file_name(format!("{}.tpl", self.tpl_editor.name))
                .add_filter("Vorlagen", &["tpl", "bt"])
                .save_file()
            {
                match std::fs::write(&p, &self.tpl_editor.source) {
                    Ok(()) => self.log(format!("Vorlage gespeichert: {}", p.display())),
                    Err(e) => self.log(format!("Speichern fehlgeschlagen: {e}")),
                }
            }
        }
        if !open {
            self.tpl_editor.open = false;
        }
    }
}
