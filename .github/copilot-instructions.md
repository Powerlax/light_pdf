# Copilot instructions for the light_pdf repo

Purpose
- This file gives repository-specific guidance for an AI coding assistant (Copilot) working on light_pdf.
- Aim: keep suggestions and edits safe, minimal, and aligned with project conventions.

Project overview (short)
- Rust native GUI using eframe/egui.
- PDF handling is in `src/pdf.rs` (PdfDocument + PdfMetadata).
- UI logic is refactored into `src/app.rs` (MyApp and render_ui helpers).
- `src/main.rs` wires the app into eframe and delegates update to `app::render_ui`.
- Metadata for each PDF is stored as a JSON file next to the PDF by default (filename + `.meta.json`).

Primary goals for edits
- Keep changes small and well-scoped.
- Prefer feature-gated platform-specific code (e.g., `rfd` only on Windows) to avoid pulling heavy native deps on other platforms.
- Preserve API stability: don't rename public types or change serialized formats without migration strategy.

How to run locally (developer)
- Build: `cargo build`
- Run: `cargo run`
- Tests: `cargo test`

Important implementation details & conventions
- Metadata writes must be atomic: write to a temporary file then rename. See `PdfDocument::actually_write_to_metadata_file`.
- Metadata persistence changed to immediate writes (no background throttling) — callers call `save_metadata()` or `persist()` after changing page/zoom.
- When adding new metadata fields, extend `PdfMetadata` and mark them `serde` serializable. Maintain backward compatibility when loading older metadata files.
- Platform-native dialogs are enabled only for Windows. If adding other OS-specific features, gate them in `Cargo.toml` and the code with cfg(target_os = ...).
- Avoid blocking the UI thread on long operations. For heavy work (e.g., rendering PDFs via pdfium), consider moving work to a background thread and send results back to the UI.

Code style / small rules
- Keep UI functions short and composable. `app.rs` contains small helpers: `render_top_menu`, `render_central_panel`, and `render_file_browser`.
- Avoid deeply nested closures in the UI; split into helper functions to make borrow-checker reasoning simpler and code easier to read.
- When editing files, preserve existing style and formatting. For Rust code, follow `rustfmt` conventions.

Testing guidance
- There are unit tests for the `PdfDocument` metadata persistence and navigation.
- Use `tempfile::tempdir()` in tests to avoid global filesystem changes.
- When adding tests that touch the file system, make sure to clean up or use temporary directories.

When adding dependencies
- Prefer small, well-maintained crates.
- For UI-native dialogs, use `rfd` but add it behind a Windows-only dependency or feature flag to keep Linux/WSL builds simple.
- Update `Cargo.toml` carefully; run `cargo build` after changes.

Common tasks with guidance
- Add a new UI menu item: modify `app::render_top_menu` and add a helper in `app.rs`.
- Add keyboard shortcuts: check `egui::Context::input` with the closure form (ctx.input(|i| ...)) and implement behavior in `app::render_ui` so it's centralized.
- Open a PDF: prefer a shared `perform_open_action` helper to keep dialog and keyboard paths consistent.

Security & safety notes
- Do not attempt to read/write files outside user-specified paths. Keep metadata files next to PDFs or in a configurable app directory.
- Don't include secrets in metadata files.

Commit / PR conventions
- Small commits, one feature/fix per PR.
- Include a short description of why the change was made (not just what).
- Run `cargo build` and `cargo test` before requesting review.

Helpful pointers / TODOs (low-risk suggestions)
- Add an in-app Preferences dialog (Edit -> Preferences) to toggle `auto_open_on_select` and set metadata dir.
- Add keyboard shortcuts: Ctrl+O (Open), Ctrl+S (Save metadata), Ctrl+Q (Quit), arrow keys or PgUp/PgDn for page navigation (arrow keys are already implemented).
- Integrate `pdfium-render` more deeply: open the PDF, set `total_pages`, and render previews asynchronously.

If you're an assistant (Copilot)
- Read this file before making changes.
- If you modify metadata serialization, ensure backwards compatibility and tests that load older files.
- Keep edits small and run the build/tests locally in the workspace after changes.

Contact
- If something in the repo is ambiguous, ask a short clarification question.

-- end

