# 101Craft

Ein Hex- und Binär-Editor für Windows, macOS und Linux, geschrieben in Rust mit eframe/egui.

## Funktionen

- Mehrere Dateien in Tabs, Hex- und Textspalte, Einfüge- und Überschreibmodus
- Rückgängig/Wiederholen, Kopieren/Ausschneiden/Einfügen (Hex oder Text)
- Suchen und Ersetzen (Text, UTF-16, Hex-Bytes, Ganzzahl), Gehe zu Adresse
- Lesezeichen
- Inspektor: Werte an der Cursorposition als Ganzzahl, Kommazahl, Datum usw., direkt bearbeitbar
- Vorlagen: eine C-ähnliche Sprache zerlegt Dateien in benannte, farbig markierte Felder.
  Mitgeliefert: BMP, PNG, GIF, WAV, ZIP, EXE/DLL. Eigene Vorlagen im eingebauten Editor.
- Prüfsummen: CRC-32, Adler-32, Summen, XOR, MD5, SHA-1, SHA-256
- Byteweiser Dateivergleich (nebeneinander), Histogramm mit Entropie
- Textansicht für UTF-8-Dateien, helles und dunkles Design

## Bauen und starten

```
cargo build --release
cargo run --release -- [Dateien...]
```

`--screenshot bild.png` startet das Programm, zeichnet etwa 120 Bilder, speichert ein Bildschirmfoto und beendet sich.

## Tastenkürzel

| Taste | Aktion |
|---|---|
| Strg+O / Strg+S | Öffnen / Speichern |
| Strg+Z / Strg+Y | Rückgängig / Wiederholen |
| Strg+F / Strg+H | Suchen / Ersetzen |
| F3 / Umschalt+F3 | Weiter / rückwärts suchen |
| Strg+G | Gehe zu Adresse |
| Strg+B / F2 | Lesezeichen setzen / nächstes |
| F5 | Vorlage ausführen |
| Einfg | Einfügen / Überschreiben |
| Tab | Zwischen Hex- und Textspalte wechseln |
