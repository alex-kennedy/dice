use std::io::{self, IsTerminal};

use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Bar, BarChart, Block, BorderType, Padding, Widget};
use ratatui::{Terminal, TerminalOptions, Viewport};

use crate::stats::{self, Stats};

/// What the chart reports about the cost of producing the distribution.
pub enum Timing {
  /// Per-roll cost across a simulation, which varies enough to be worth a spread.
  Simulated {
    runs: usize,
    mean_nanos: f64,
    sd_nanos: f64,
  },
  /// Wall time of the one exact calculation.
  Calculated { nanos: f64 },
}

/// Rows of bars in the chart, excluding the axis labels and the surrounding block.
const BAR_ROWS: u16 = 13;
/// The block adds two borders and a row of axis labels to [`BAR_ROWS`].
const CHART_HEIGHT: u16 = BAR_ROWS + 3;
/// Bars are drawn wide enough to be readable, but a handful of outcomes shouldn't produce
/// slabs that span the terminal.
const MAX_BAR_WIDTH: u16 = 6;
/// Probabilities are scaled into the integers [`BarChart`] works in.
const SCALE: f64 = 1e9;
/// Width assumed when stdout isn't a terminal and there's nothing to measure.
const PIPED_WIDTH: u16 = 80;

const ACCENT: Color = Color::Rgb(120, 226, 240);

/// Draws the distribution above the cursor, leaving it in the scrollback.
///
/// When stdout isn't a terminal the same rendering is dumped as plain text, so the output
/// stays readable through a pipe.
pub fn show(expression: &str, pmf: &[(i32, f64)], timing: &Timing) -> io::Result<()> {
  // A blank line, the chart, a blank line, two lines of stats and a trailing blank.
  let height = CHART_HEIGHT + 5;
  let stats = &stats::summarize(pmf);

  if io::stdout().is_terminal() {
    let options = TerminalOptions {
      viewport: Viewport::Inline(1),
    };
    // Anchoring an inline viewport means asking the terminal where the cursor is, which
    // not every terminal (or CI log) will answer. Fall through to plain text if it doesn't.
    if let Ok(mut terminal) = Terminal::with_options(CrosstermBackend::new(io::stdout()), options) {
      terminal.insert_before(height, |buf| draw(buf, expression, pmf, stats, timing))?;
      terminal.clear()?;
      return Ok(());
    }
  }

  let mut buf = Buffer::empty(Rect::new(0, 0, terminal_width(), height));
  draw(&mut buf, expression, pmf, stats, timing);
  print_plain(&buf);
  Ok(())
}

fn terminal_width() -> u16 {
  ratatui::crossterm::terminal::size().map_or(PIPED_WIDTH, |(width, _)| width)
}

fn draw(buf: &mut Buffer, expression: &str, pmf: &[(i32, f64)], stats: &Stats, timing: &Timing) {
  let [_, chart, _, summary, quantiles, _] = Layout::vertical([
    Constraint::Length(1),
    Constraint::Length(CHART_HEIGHT),
    Constraint::Length(1),
    Constraint::Length(1),
    Constraint::Length(1),
    Constraint::Length(1),
  ])
  .horizontal_margin(1)
  .areas(buf.area);

  draw_chart(buf, chart, expression, pmf, timing);

  Line::from(
    [
      stat("mean", format!("{:.2}", stats.mean)),
      stat("sd", format!("{:.2}", stats.sd)),
      stat("max p", format!("{:.2}%", stats.peak * 100.0)),
    ]
    .concat(),
  )
  .render(summary, buf);

  let mut spans = stat("min", stats.min.to_string());
  for (percent, value) in &stats.quantiles {
    spans.extend(stat(&format!("p{percent}"), value.to_string()));
  }
  spans.extend(stat("max", stats.max.to_string()));
  Line::from(spans).render(quantiles, buf);
}

fn draw_chart(
  buf: &mut Buffer,
  area: Rect,
  expression: &str,
  pmf: &[(i32, f64)],
  timing: &Timing,
) {
  let block = Block::bordered()
    .border_type(BorderType::Rounded)
    .border_style(Style::new().dim())
    .padding(Padding::horizontal(1))
    .title_top(Line::from(vec![
      Span::raw(" "),
      Span::styled(expression, Style::new().bold().fg(ACCENT)),
      Span::raw(" "),
    ]))
    .title_top(Line::from(cost(timing)).right_aligned().dim());
  let inner = block.inner(area);
  block.render(area, buf);

  let (data, bar_width, bar_gap) = fit_bars(pmf, inner.width);
  let labelled = data
    .iter()
    .all(|(label, _)| bar_width >= label.len() as u16);

  // Scaled to the tallest bar rather than the likeliest outcome: a bucketed bar averages
  // the outcomes it covers, so it sits below the peak wherever a bucket straddles the mode.
  let ceiling = data.iter().map(|(_, p)| *p).fold(0.0, f64::max);

  let bars: Vec<Bar> = data
    .iter()
    .map(|(label, p)| {
      let bar = Bar::new((p * SCALE) as u64)
        .style(Style::new().fg(bar_color(p / ceiling)))
        // The value would otherwise be stamped over the foot of the bar.
        .text_value(String::new());
      if labelled {
        bar.label(label.clone())
      } else {
        bar
      }
    })
    .collect();

  // Bars rarely divide the block exactly; centre them rather than leaving a ragged gap
  // down the right hand side.
  let count = bars.len() as u16;
  let used = (count * bar_width + count.saturating_sub(1) * bar_gap).min(inner.width);
  let plot = Rect {
    x: inner.x + (inner.width - used) / 2,
    width: used,
    ..inner
  };

  BarChart::vertical(bars)
    .bar_width(bar_width)
    .bar_gap(bar_gap)
    .max((ceiling * SCALE) as u64)
    .label_style(Style::new().dim())
    .render(plot, buf);
}

/// Chooses bar widths that fill the available space, bucketing adjacent outcomes together
/// when there are more of them than there are columns to draw them in.
fn fit_bars(pmf: &[(i32, f64)], available: u16) -> (Vec<(String, f64)>, u16, u16) {
  let capacity = available.max(1) as usize;
  let data: Vec<(String, f64)> = if pmf.len() > capacity {
    pmf
      .chunks(pmf.len().div_ceil(capacity))
      .map(|chunk| {
        // The mean, not the total, so a bar always means "probability of one outcome".
        // The last chunk is usually short, and would otherwise read as a dip that the
        // distribution doesn't have.
        let total: f64 = chunk.iter().map(|(_, p)| p).sum();
        (chunk[0].0.to_string(), total / chunk.len() as f64)
      })
      .collect()
  } else {
    pmf.iter().map(|(v, p)| (v.to_string(), *p)).collect()
  };

  let slot = capacity / data.len().max(1);
  let (bar_width, bar_gap) = match slot {
    0 | 1 => (1, 0),
    2 => (1, 1),
    _ => ((slot as u16 - 1).min(MAX_BAR_WIDTH), 1),
  };
  (data, bar_width, bar_gap)
}

/// Shades bars from a dim indigo up to [`ACCENT`] at the mode, so the shape of the
/// distribution reads even where the bars are short.
fn bar_color(fraction: f64) -> Color {
  const LOW: (f64, f64, f64) = (62.0, 74.0, 142.0);
  const HIGH: (f64, f64, f64) = (120.0, 226.0, 240.0);

  // The square root keeps the tail from washing out into the background.
  let t = fraction.clamp(0.0, 1.0).sqrt();
  let mix = |low: f64, high: f64| (low + (high - low) * t) as u8;
  Color::Rgb(mix(LOW.0, HIGH.0), mix(LOW.1, HIGH.1), mix(LOW.2, HIGH.2))
}

fn stat<'a>(label: &str, value: String) -> Vec<Span<'a>> {
  vec![
    Span::styled(label.to_string(), Style::new().dim()),
    Span::raw(" "),
    Span::styled(value, Style::new().bold()),
    Span::raw("   "),
  ]
}

/// Summarises what the distribution cost to produce, e.g. `100,000 runs · 412.3 ± 87.1 ns
/// per roll` or `calculated in 1.2 ms`.
fn cost(timing: &Timing) -> String {
  match timing {
    Timing::Simulated {
      runs,
      mean_nanos,
      sd_nanos,
    } => {
      // The unit comes from the mean so that the spread is directly comparable to it.
      let (scale, unit) = unit_for(*mean_nanos);
      format!(
        " {} runs · {:.1} ± {:.1} {unit} per roll ",
        thousands(*runs),
        mean_nanos / scale,
        sd_nanos / scale
      )
    }
    Timing::Calculated { nanos } => {
      let (scale, unit) = unit_for(*nanos);
      format!(" calculated in {:.1} {unit} ", nanos / scale)
    }
  }
}

/// The largest unit that leaves `nanos` above 1, as a (divisor, suffix) pair.
fn unit_for(nanos: f64) -> (f64, &'static str) {
  const UNITS: [(f64, &str); 4] = [(1e9, "s"), (1e6, "ms"), (1e3, "µs"), (1.0, "ns")];

  UNITS
    .into_iter()
    .find(|(scale, _)| nanos >= *scale)
    .unwrap_or((1.0, "ns"))
}

fn thousands(n: usize) -> String {
  let digits = n.to_string();
  let mut out = String::with_capacity(digits.len() + digits.len() / 3);
  for (i, c) in digits.char_indices() {
    if i > 0 && (digits.len() - i).is_multiple_of(3) {
      out.push(',');
    }
    out.push(c);
  }
  out
}

fn print_plain(buf: &Buffer) {
  for y in buf.area.top()..buf.area.bottom() {
    let mut line = String::new();
    for x in buf.area.left()..buf.area.right() {
      line.push_str(buf[(x, y)].symbol());
    }
    println!("{}", line.trim_end());
  }
}
