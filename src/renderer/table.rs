use comfy_table::{
    presets::UTF8_FULL,
    modifiers::UTF8_ROUND_CORNERS,
    Table, ContentArrangement, Color, Cell,
};

/// Create a preconfigured comfy-table instance with consistent styling.
pub fn make_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    // cyan header
    let header_cells = headers
        .iter()
        .map(|h| Cell::new(*h).fg(Color::Cyan))
        .collect::<Vec<_>>();
    table.set_header(header_cells);
    table
}

/* /// Helper for colored header text
pub fn header_cell(text: &str) -> Cell {
    Cell::new(text).fg(Color::Cyan)
} */
