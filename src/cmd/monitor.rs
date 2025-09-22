use std::collections::HashMap;
use std::io::{self};
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::ValueEnum;
use crossterm::event::KeyEventKind;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use humansize::{format_size, BINARY};
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Modifier, Color},
    text::Span,
    widgets::{Block, Borders, Row, Table, Clear},
    Terminal,
};
use termtree::Tree;

use crate::cli::Cli;
use crate::collector::iface::collect_all_interfaces;
use crate::cli::MonitorArgs;
use crate::renderer::tree::{fmt_bps, fmt_flags, tree_label};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum SortKey { 
    Total,
    TotalRx,
    TotalTx,
    Rx, 
    Tx
}

impl SortKey {
    fn cycle(self) -> Self {
        match self { 
            SortKey::Total => SortKey::TotalRx, 
            SortKey::TotalRx => SortKey::TotalTx,
            SortKey::TotalTx => SortKey::Rx,
            SortKey::Rx => SortKey::Tx, 
            SortKey::Tx => SortKey::Total, 
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Unit {
    Bytes,
    Bits,
}

impl Default for Unit {
    fn default() -> Self {
        Unit::Bytes
    }
}

#[derive(Debug, Clone)]
struct StatPoint {
    rx_bytes: u64,
    tx_bytes: u64,
    ts: Instant,
}

#[derive(Debug, Default, Clone)]
struct Rate {
    rx_per_s: f64,
    tx_per_s: f64,
}

#[derive(Debug)]
struct RowData {
    index: u32,
    name: String,
    friendly_name: Option<String>,
    total: u64,
    total_tx: u64,
    total_rx: u64,
    rx: f64,
    tx: f64,
}

pub fn monitor_interfaces(_cli: &Cli, args: &MonitorArgs) -> Result<()> {
    // Settings
    let mut sort = args.sort;
    let target_iface = args.iface.clone(); // Option<String>
    let tick = Duration::from_secs(args.interval.max(1));

    // Switch terminal to TUI mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut ifs = collect_all_interfaces();
    // Collect (target IF only or all)
    if let Some(ref name) = target_iface {
        ifs.retain(|it| &it.name == name);
    }

    let max_name_len = get_max_if_name_len(&ifs);

    let mut prev: HashMap<String, StatPoint> = HashMap::new();
    for itf in &mut ifs {
        let _ = itf.update_stats();
        if let Some(st) = &itf.stats {
            prev.insert(itf.name.clone(), StatPoint {
                rx_bytes: st.rx_bytes,
                tx_bytes: st.tx_bytes,
                ts: Instant::now(),
            });
        }
    }
    let mut rows_cache: Vec<RowData> = Vec::new();
    let mut next_tick = Instant::now();
    let mut selected: usize = 0;
    let mut popup_open = false;
    let mut popup_scroll: u16 = 0;

    // Main loop
    let res = (|| -> Result<()> {
        loop {
            // Calculate remaining time until next tick
            let now = Instant::now();
            let remain = if now >= next_tick {
                Duration::from_millis(0)
            } else {
                next_tick.saturating_duration_since(now)
            };

            // Input processing (wait for the remaining time. If tick comes, exit with false)
            if event::poll(remain)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
                            KeyCode::Char('o') => sort = sort.cycle(),
                            KeyCode::Char('r') => {
                                ifs = collect_all_interfaces();
                                if let Some(ref name) = target_iface { ifs.retain(|it| &it.name == name); }
                                prev.clear();
                            },
                            KeyCode::Up | KeyCode::Char('w') if !popup_open => {
                                if selected > 0 { selected -= 1; }
                            },
                            KeyCode::Down | KeyCode::Char('s') if !popup_open => {
                                if selected + 1 < rows_cache.len() { selected += 1; }
                            },
                            KeyCode::Up | KeyCode::Char('w') if popup_open => { 
                                popup_scroll = popup_scroll.saturating_sub(1); 
                            },
                            KeyCode::Down | KeyCode::Char('s') if popup_open => { 
                                popup_scroll = popup_scroll.saturating_add(1); 
                            },
                            KeyCode::Enter => {
                                popup_open = true;
                                popup_scroll = 0;
                            },
                            KeyCode::Esc => {
                                popup_open = false;
                            },
                            _ => {}
                        }
                    }
                }
            }

            // Tick processing
            if Instant::now() >= next_tick {
                //next_tick = Instant::now() + tick;
                next_tick += tick;
                let tick_ts = Instant::now();
                let mut rows: Vec<RowData> = Vec::with_capacity(ifs.len());
                for itf in &mut ifs {
                    // Update stats
                    let _ = itf.update_stats();

                    if let Some(st) = itf.stats.as_ref() {
                        let key = itf.name.clone();
                        // Current snapshot
                        let nowp = StatPoint {
                            rx_bytes: st.rx_bytes,
                            tx_bytes: st.tx_bytes,
                            ts: tick_ts,
                        };
                        // If there is a previous snapshot, calculate the difference; otherwise, use 0
                        let rate = if let Some(prevp) = prev.get(&key) {
                            let dt = nowp.ts.duration_since(prevp.ts).as_secs_f64().max(0.001);
                            Rate {
                                rx_per_s: (nowp.rx_bytes.saturating_sub(prevp.rx_bytes) as f64) / dt,
                                tx_per_s: (nowp.tx_bytes.saturating_sub(prevp.tx_bytes) as f64) / dt,
                            }
                        } else {
                            Rate { rx_per_s: 0.0, tx_per_s: 0.0 }
                        };

                        // Update prev for next time (only on tick)
                        prev.insert(key.clone(), nowp);

                        rows.push(RowData {
                            index: itf.index,
                            name: itf.name.clone(),
                            friendly_name: itf.friendly_name.clone(),
                            total_rx: st.rx_bytes,
                            total_tx: st.tx_bytes,
                            total: st.rx_bytes + st.tx_bytes,
                            rx: rate.rx_per_s,
                            tx: rate.tx_per_s,
                        });
                    }
                }

                // Sort and replace cache (only on tick)
                match sort {
                    SortKey::Total => rows.sort_by(|a,b| b.total.cmp(&a.total)),
                    SortKey::TotalRx => rows.sort_by(|a,b| b.total_rx.cmp(&a.total_rx)),
                    SortKey::TotalTx => rows.sort_by(|a,b| b.total_tx.cmp(&a.total_tx)),
                    SortKey::Rx => rows.sort_by(|a,b| b.rx.total_cmp(&a.rx)),
                    SortKey::Tx => rows.sort_by(|a,b| b.tx.total_cmp(&a.tx)),
                }
                rows_cache = rows;
                if !rows_cache.is_empty() {
                    if selected >= rows_cache.len() { selected = rows_cache.len() - 1; }
                }
            }

            // Draw using rows_cache at all times (maintain "previous value" when not tick)
            terminal.draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(3),
                        Constraint::Length(1)
                        ].as_ref())
                    .split(size);

                // Header
                let unit_label = match args.unit { Unit::Bytes => "bytes", Unit::Bits => "bits" };
                let title = format!(
                    "nifa monitor — sort:{:?} — unit:{} — interval:{}s {}",
                    sort, unit_label, args.interval, target_iface.as_deref().unwrap_or("(all)")
                );

                let header = Row::new(vec![
                    Span::styled("IFACE", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("Total", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("Total RX", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("Total TX", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("RX/s", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("TX/s", Style::default().add_modifier(Modifier::BOLD)),
                ]);

                let rows_iter = rows_cache.iter().enumerate().map(|(i, r)| {
                    let base = Row::new(vec![
                        Span::raw(platform_if_name(r)),
                        Span::raw(human_total(r.total, args.unit)),
                        Span::raw(human_total(r.total_rx, args.unit)),
                        Span::raw(human_total(r.total_tx, args.unit)),
                        Span::raw(human_rate(r.rx, args.unit)),
                        Span::raw(human_rate(r.tx, args.unit)),
                    ]);
                    if i == selected {
                        base.style(Style::default().bg(ratatui::style::Color::DarkGray))
                    } else {
                        base
                    }
                });

                // Table
                let table = Table::new(rows_iter, [
                        Constraint::Length(max_name_len),
                        Constraint::Length(14),
                        Constraint::Length(14),
                        Constraint::Length(14),
                        Constraint::Length(14),
                        Constraint::Length(14),
                    ])
                    .header(header)
                    .block(Block::default().borders(Borders::ALL).title(title))
                    .column_spacing(2);

                f.render_widget(table, chunks[0]);

                // Help
                let help = "Press <q> to quit | <o> cycle sort | <r> rescan interfaces | ↑/↓/w/s select | Enter details | CTRL+C to exit";
                let help_span = Span::styled(help, Style::default().fg(ratatui::style::Color::DarkGray));
                let help_row = Row::new(vec![help_span]);
                let help_table = Table::new(
                    std::iter::once(help_row),
                    [Constraint::Percentage(100)]
                );
                f.render_widget(help_table, chunks[1]);

                // Modal popup
                if popup_open && !ifs.is_empty() && selected < ifs.len() {
                    let sel_if_index = &rows_cache[selected].index;
                    if let Some(iface) = ifs.iter().find(|it| &it.index == sel_if_index) {
                        let area = centered_rect(66, 60, size);

                        // Background fill (black)
                        // f.render_widget(Block::default().style(Style::default().bg(Color::Black)), size);

                        // Clear the area first
                        f.render_widget(Clear, area);

                        let block = Block::default()
                            .title(format!("Details: {} (Esc to close — ↑/↓/w/s scroll)", iface.name))
                            .borders(Borders::ALL)
                            .style(Style::default().bg(Color::Black));

                        let inner = block.inner(area);

                        // Detail text (tree string created by termtree)
                        let detail_text = iface_to_text(iface);

                        // Estimate content height (based on line breaks)
                        let content_lines = detail_text.lines().count() as u16;
                        // Visible lines in the popup
                        let visible_lines = inner.height;

                        // Clamp to scroll limit
                        let max_scroll = content_lines.saturating_sub(visible_lines).saturating_add(2);
                        if popup_scroll > max_scroll { popup_scroll = max_scroll; }

                        let paragraph = Paragraph::new(Text::raw(detail_text))
                            .block(block)
                            .wrap(Wrap { trim: false })   
                            .scroll((popup_scroll, 0));

                        f.render_widget(paragraph, area);
                    }
                }

            })?;
        }
    })();

    // Cleanup
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    // Return result of main loop
    res
}

/// Get the maximum interface name length for table column width
/// On Windows, consider friendly_name if available
fn get_max_if_name_len(ifs: &[netdev::Interface]) -> u16 {
    let max_len: usize =if cfg!(windows) {
        ifs.iter().map(|it| {
            if let Some(fn_name) = &it.friendly_name {
                fn_name.len().max(it.name.len())
            } else {
                it.name.len()
            }
        }).max().unwrap_or(0)
    } else {
        ifs.iter().map(|it| it.name.len()).max().unwrap_or(0)
    };
    (max_len as u16).max(5)
}

/// Platform-specific interface name specification
/// Linux/Unix: use `name` as-is
/// Windows: use `friendly_name` if available; otherwise, use `name`
fn platform_if_name(row: &RowData) -> &str {
    if cfg!(windows) {
        if let Some(friendly_name) = &row.friendly_name {
            friendly_name
        } else {
            &row.name
        }
    } else {
        &row.name
    }
}

// Total (Bytes or Bits)
fn human_total(v_bytes: u64, unit: Unit) -> String {
    match unit {
        Unit::Bytes => format_size(v_bytes, BINARY),
        Unit::Bits => {
            let vb = (v_bytes as f64) * 8.0;
            if vb < 1000.0 {
                format!("{:.0} b", vb)
            } else if vb < 1_000_000.0 {
                format!("{:.1} Kb", vb / 1_000.0)
            } else if vb < 1_000_000_000.0 {
                format!("{:.1} Mb", vb / 1_000_000.0)
            } else {
                format!("{:.2} Gb", vb / 1_000_000_000.0)
            }
        }
    }
}

// Rate (Bytes/s or Bits/s)
fn human_rate(v: f64, unit: Unit) -> String {
    match unit {
        Unit::Bytes => {
            if v < 1000.0 {
                format!("{:.0} B/s", v)
            } else {
                let s = format_size(v as u64, BINARY);
                format!("{}/s", s)
            }
        }
        Unit::Bits => {
            let vb = v * 8.0;
            if vb < 1000.0 {
                format!("{:.0} b/s", vb)
            } else if vb < 1_000_000.0 {
                format!("{:.1} Kb/s", vb / 1_000.0)
            } else if vb < 1_000_000_000.0 {
                format!("{:.1} Mb/s", vb / 1_000_000.0)
            } else {
                format!("{:.2} Gb/s", vb / 1_000_000_000.0)
            }
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);
    area[1]
}

fn iface_to_text(iface: &netdev::Interface) -> String {
    let host = crate::collector::sys::hostname();
    let title = format!(
        "{}{} on {}",
        iface.name,
        if iface.default { " (default)" } else { "" },
        host
    );
    let mut root = Tree::new(tree_label(title));

    // flat fields (no General section)
    root.push(Tree::new(format!("Index: {}", iface.index)));

    if let Some(fn_name) = &iface.friendly_name {
        root.push(Tree::new(format!("Friendly Name: {}", fn_name)));
    }
    if let Some(desc) = &iface.description {
        root.push(Tree::new(format!("Description: {}", desc)));
    }

    root.push(Tree::new(format!("Type: {:?}", iface.if_type)));
    root.push(Tree::new(format!("State: {:?}", iface.oper_state)));

    if let Some(mac) = &iface.mac_addr {
        root.push(Tree::new(format!("MAC: {}", mac)));
    }
    if let Some(mtu) = iface.mtu {
        root.push(Tree::new(format!("MTU: {}", mtu)));
    }

    // link speeds (humanized bps)
    if iface.transmit_speed.is_some() || iface.receive_speed.is_some() {
        let mut speed = Tree::new(tree_label("Link Speed"));
        if let Some(tx) = iface.transmit_speed { speed.push(Tree::new(format!("TX: {}", fmt_bps(tx)))); }
        if let Some(rx) = iface.receive_speed { speed.push(Tree::new(format!("RX: {}", fmt_bps(rx)))); }
        root.push(speed);
    }

    // flags
    root.push(Tree::new(format!("Flags: {}", fmt_flags(iface.flags))));

    // ---- Addresses ----
    if !iface.ipv4.is_empty() {
        let mut ipv4_tree = Tree::new(tree_label("IPv4"));
        for net in &iface.ipv4 { ipv4_tree.push(Tree::new(net.to_string())); }
        root.push(ipv4_tree);
    }

    if !iface.ipv6.is_empty() {
        let mut ipv6_tree = Tree::new(tree_label("IPv6"));
        for (i, net) in iface.ipv6.iter().enumerate() {
            let mut label = net.to_string();
            if let Some(scope) = iface.ipv6_scope_ids.get(i) { label.push_str(&format!(" (scope_id={})", scope)); }
            ipv6_tree.push(Tree::new(label));
        }
        root.push(ipv6_tree);
    }

    // ---- DNS ----
    if !iface.dns_servers.is_empty() {
        let mut dns_tree = Tree::new(tree_label("DNS"));
        for dns in &iface.dns_servers { dns_tree.push(Tree::new(dns.to_string())); }
        root.push(dns_tree);
    }

    // ---- Gateway ----
    if let Some(gw) = &iface.gateway {
        let mut gw_node = Tree::new(tree_label("Gateway"));
        gw_node.push(Tree::new(format!("MAC: {}", gw.mac_addr)));
        if !gw.ipv4.is_empty() {
            let mut gw4 = Tree::new(tree_label("IPv4"));
            for ip in &gw.ipv4 { gw4.push(Tree::new(ip.to_string())); }
            gw_node.push(gw4);
        }
        if !gw.ipv6.is_empty() {
            let mut gw6 = Tree::new(tree_label("IPv6"));
            for ip in &gw.ipv6 { gw6.push(Tree::new(ip.to_string())); }
            gw_node.push(gw6);
        }
        root.push(gw_node);
    }

    // ---- Statistics (snapshot) ----
    if let Some(st) = &iface.stats {
        let mut stats_node = Tree::new(tree_label("Statistics (snapshot)"));
        stats_node.push(Tree::new(format!("RX bytes: {}", st.rx_bytes)));
        stats_node.push(Tree::new(format!("TX bytes: {}", st.tx_bytes)));
        root.push(stats_node);
    }

    //println!("{}", root);
    format!("{}", root)
}
