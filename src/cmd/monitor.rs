use std::collections::HashMap;
use std::io::{self};
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::ValueEnum;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use humansize::{format_size, BINARY};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Style, Modifier},
    text::Span,
    widgets::{Block, Borders, Row, Table},
    Terminal,
};

use crate::cli::Cli;
use crate::collector::iface::collect_all_interfaces;
use crate::cli::MonitorArgs;

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
    name: String,
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
                match event::read()? {
                    Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => return Ok(()),
                    Event::Key(KeyEvent { code: KeyCode::Char('c'), modifiers, .. }) if modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
                    Event::Key(KeyEvent { code: KeyCode::Char('s'), .. }) => sort = sort.cycle(),
                    Event::Key(KeyEvent { code: KeyCode::Char('r'), .. }) => {
                        ifs = collect_all_interfaces();
                        if let Some(ref name) = target_iface { ifs.retain(|it| &it.name == name); }
                        prev.clear();
                    }
                    _ => {}
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
                            name: itf.name.clone(),
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
            }

            // Draw using rows_cache at all times (maintain "previous value" when not tick)
            terminal.draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        //Constraint::Length(3), 
                        Constraint::Min(3),
                        Constraint::Length(1)
                        ].as_ref())
                    .split(size);

                // Header
                /* let hdr = Block::default().borders(Borders::ALL).title(format!(
                    "nifa monitor — sort:{:?} — interval:{}s {}",
                    sort,
                    args.interval,
                    target_iface.as_deref().unwrap_or("(all)")
                )); */
                //f.render_widget(hdr, chunks[0]);

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

                let rows_iter = rows_cache.iter().map(|r| {
                    Row::new(vec![
                        Span::raw(&r.name),
                        Span::raw(human_total(r.total, args.unit)),
                        Span::raw(human_total(r.total_rx, args.unit)),
                        Span::raw(human_total(r.total_tx, args.unit)),
                        Span::raw(human_rate(r.rx, args.unit)),
                        Span::raw(human_rate(r.tx, args.unit)),
                    ])
                });

                // Table
                let table = Table::new(rows_iter, [
                        Constraint::Length(18),
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
                let help = "Press <q> to quit | <s> cycle sort | <r> rescan interfaces | CTRL+C to exit";
                let help_span = Span::styled(help, Style::default().fg(ratatui::style::Color::DarkGray));
                let help_row = Row::new(vec![help_span]);
                let help_table = Table::new(
                    std::iter::once(help_row),
                    [Constraint::Percentage(100)]
                );
                f.render_widget(help_table, chunks[1]);

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
