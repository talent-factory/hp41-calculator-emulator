//! TUI widget layout — implemented in Plan 04-02.

use ratatui::Frame;
use crate::app::App;

/// Render the full HP-41 TUI. Called from App::draw() on every frame.
/// Fully implemented in Plan 04-02; this stub satisfies the module declaration.
pub fn render_ui(app: &App, frame: &mut Frame) {
    use ratatui::widgets::Paragraph;
    let _ = app;
    frame.render_widget(
        Paragraph::new("HP-41 TUI — rendering in Plan 04-02"),
        frame.area(),
    );
}
