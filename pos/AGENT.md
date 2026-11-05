# POS — Agent Workspace Guidelines & References

## Build & UI
- The desktop UI is **iced 0.14**; egui/eframe are no longer dependencies.
- Build/test from `standalone/`: `cargo check`, `cargo test`, `cargo build`.
- Run the app: `cargo run --manifest-path app/Cargo.toml`.
- Keep pure state/logic in `main_ui.rs` and `actions.rs`; UI code belongs in `app/src/ui/`.
- The iced dependency pulls `winit` transitively. The workspace `[patch.crates-io]` winit fork is the shared Craft-suite IME/language-support fork (W1–W27) and must stay.

## Parent Repository Reference
- Selalu jadikan folder 1 level di atas (`..` atau `f:\GitHub\designcraft-pubgen`) sebagai repositori referensi utama.
- **Arsitektur & Konvensi**: Rujuk `../AGENTS.md`, `../ROADMAP.md`, dan dokumen di `../docs/`.
- **Crates & Engine**: Rujuk crates di `../crates/` (seperti `crates/engine`, `crates/ui-egui`, `crates/fonts`, dll.) untuk implementasi komponen, format dokumen, styling, dan integrasi DesignCraft.
- **Perencanaan & Aset**: Rujuk `../plan/` dan `../assets/` untuk sampel publikasi, tipografi, dan spesifikasi format (misal InDesign parity, BRS, DDA).
- **Inventaris Kode**: Periksa dan perbarui `reusable_code_inventory.md` bila ada modul dari parent repo yang relevan atau dapat digunakan kembali di `POS`.