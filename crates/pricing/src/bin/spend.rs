//! tokenTracer spend CLI — fixture / stdin → SpendSummary / DailySpendSeries / ImportMeta JSON.

use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};
use pricing::parsers::{
    parse_claude_code_jsonl_with_source, parse_codex_rollout_jsonl_with_source, parse_cursor_local,
};
use pricing::{
    daily_spend_series, filter_events_by_range, price_with_range, read_import_meta, record_import,
    AgentId, Currency, DayBoundary, FxSnapshot, PriceTable, RangeFilterOpts, RangeKind,
    SeriesOpts, UsageEvent,
};
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "spend", about = "tokenTracer spend CLI (notional API estimate)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Price normalized UsageEvent JSON (file or stdin) → SpendSummary JSON.
    Total {
        #[arg(long, value_enum, default_value_t = CurrencyArg::Usd)]
        currency: CurrencyArg,
        /// Path to UsageEvent JSON array. Omit or `-` for stdin.
        #[arg(long)]
        events: Option<PathBuf>,
        /// Range: all|today|7d|30d|90d (OPEN-UI-1).
        #[arg(long, value_enum, default_value_t = RangeArg::All)]
        range: RangeArg,
        #[arg(long, value_enum, default_value_t = DayBoundaryArg::Local)]
        day_boundary: DayBoundaryArg,
        #[arg(long, default_value = "Asia/Taipei")]
        timezone: String,
        #[arg(long)]
        fx_rate: Option<f64>,
        #[arg(long, default_value = "manual")]
        fx_source: String,
        /// Optional: also print a compact by_model human table after JSON.
        #[arg(long, default_value_t = false)]
        by_model: bool,
    },
    /// Emit SpendSummary focused on by_model (AC-F11). Same JSON contract as `total`.
    ByModel {
        #[arg(long, value_enum, default_value_t = CurrencyArg::Usd)]
        currency: CurrencyArg,
        #[arg(long)]
        events: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = RangeArg::All)]
        range: RangeArg,
        #[arg(long, value_enum, default_value_t = DayBoundaryArg::Local)]
        day_boundary: DayBoundaryArg,
        #[arg(long, default_value = "Asia/Taipei")]
        timezone: String,
        #[arg(long)]
        fx_rate: Option<f64>,
        #[arg(long, default_value = "manual")]
        fx_source: String,
    },
    /// Emit SpendSummary focused on by_usage_pool / Cursor dual pool (AC-F13′).
    ByPool {
        #[arg(long, value_enum, default_value_t = CurrencyArg::Usd)]
        currency: CurrencyArg,
        #[arg(long)]
        events: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = RangeArg::All)]
        range: RangeArg,
        #[arg(long, value_enum, default_value_t = DayBoundaryArg::Local)]
        day_boundary: DayBoundaryArg,
        #[arg(long, default_value = "Asia/Taipei")]
        timezone: String,
        #[arg(long)]
        fx_rate: Option<f64>,
        #[arg(long, default_value = "manual")]
        fx_source: String,
    },
    /// Alias for `total --range today`.
    Today {
        #[arg(long, value_enum, default_value_t = CurrencyArg::Usd)]
        currency: CurrencyArg,
        #[arg(long)]
        events: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = DayBoundaryArg::Local)]
        day_boundary: DayBoundaryArg,
        #[arg(long, default_value = "Asia/Taipei")]
        timezone: String,
        #[arg(long)]
        fx_rate: Option<f64>,
        #[arg(long, default_value = "manual")]
        fx_source: String,
    },
    /// Daily spend series (OPEN-UI-2).
    Series {
        #[arg(long, default_value = "day")]
        grain: String,
        #[arg(long, value_enum, default_value_t = CurrencyArg::Usd)]
        currency: CurrencyArg,
        #[arg(long)]
        events: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = RangeArg::Days30)]
        range: RangeArg,
        /// Inclusive calendar date YYYY-MM-DD (wins over --range → kind=custom).
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long, value_enum, default_value_t = DayBoundaryArg::Local)]
        day_boundary: DayBoundaryArg,
        #[arg(long, default_value = "Asia/Taipei")]
        timezone: String,
        #[arg(long)]
        fx_rate: Option<f64>,
        #[arg(long, default_value = "manual")]
        fx_source: String,
    },
    /// Smoke-parse an agent transcript into UsageEvent JSON.
    Parse {
        #[arg(long, value_enum)]
        agent: ParseAgent,
        #[arg(long)]
        file: PathBuf,
    },
    /// Import pipeline status / fake import (OPEN-UI-3).
    Import {
        #[command(subcommand)]
        cmd: ImportCmd,
    },
}

#[derive(Subcommand, Debug)]
enum ImportCmd {
    /// Print ImportMeta JSON from state file.
    Status {
        #[arg(long, default_value = ".token-tracer/import-meta.json")]
        state: PathBuf,
        /// Accepted for contract symmetry; always JSON.
        #[arg(long, default_value_t = true)]
        json: bool,
    },
    /// Record a (fake) import into the state file — for tests / smoke.
    Run {
        #[arg(long, default_value = ".token-tracer/import-meta.json")]
        state: PathBuf,
        #[arg(long, default_value = "0")]
        events_upserted: u64,
        #[arg(long, default_value = "0")]
        parse_error_count: u64,
        #[arg(long)]
        source_id: Vec<String>,
        #[arg(long, default_value_t = true)]
        json: bool,
    },
}

#[derive(Clone, Debug, ValueEnum)]
#[value(rename_all = "UPPER")]
enum CurrencyArg {
    Usd,
    Twd,
}

#[derive(Clone, Debug, ValueEnum)]
enum RangeArg {
    All,
    Today,
    #[value(name = "7d")]
    Days7,
    #[value(name = "30d")]
    Days30,
    #[value(name = "90d")]
    Days90,
}

impl RangeArg {
    fn to_kind(&self) -> RangeKind {
        match self {
            RangeArg::All => RangeKind::All,
            RangeArg::Today => RangeKind::Today,
            RangeArg::Days7 => RangeKind::Days7,
            RangeArg::Days30 => RangeKind::Days30,
            RangeArg::Days90 => RangeKind::Days90,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
enum DayBoundaryArg {
    Local,
    Utc,
}

impl DayBoundaryArg {
    fn to_boundary(&self) -> DayBoundary {
        match self {
            DayBoundaryArg::Local => DayBoundary::Local,
            DayBoundaryArg::Utc => DayBoundary::Utc,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
enum ParseAgent {
    ClaudeCode,
    Codex,
    Cursor,
}

fn read_events(path: Option<&PathBuf>) -> Result<Vec<UsageEvent>> {
    let text = match path {
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf
        }
        Some(p) if p.as_os_str() == "-" => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf
        }
        Some(p) => std::fs::read_to_string(p)
            .with_context(|| format!("read events file {}", p.display()))?,
    };
    let events: Vec<UsageEvent> =
        serde_json::from_str(&text).context("parse UsageEvent JSON array")?;
    Ok(events)
}

fn build_fx(currency: &CurrencyArg, fx_rate: Option<f64>, fx_source: String) -> Result<Option<FxSnapshot>> {
    match (currency, fx_rate) {
        (CurrencyArg::Twd, Some(rate)) => Ok(Some(FxSnapshot {
            pair: "USD/TWD".into(),
            rate,
            as_of: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            source: fx_source,
        })),
        (CurrencyArg::Twd, None) => {
            bail!("--currency TWD requires --fx-rate <TWD per 1 USD>")
        }
        _ => Ok(None),
    }
}

fn run_total(
    currency: CurrencyArg,
    events: Option<PathBuf>,
    range: RangeArg,
    day_boundary: DayBoundaryArg,
    timezone: String,
    fx_rate: Option<f64>,
    fx_source: String,
    print_by_model_table: bool,
) -> Result<()> {
    let table = PriceTable::embedded();
    let all_events = read_events(events.as_ref())?;
    let fx = build_fx(&currency, fx_rate, fx_source)?;
    let opts = RangeFilterOpts {
        kind: range.to_kind(),
        day_boundary: day_boundary.to_boundary(),
        timezone: Some(timezone.as_str()),
        now: Utc::now(),
        custom_from: None,
        custom_to: None,
    };
    let (filtered, window) = filter_events_by_range(&all_events, &opts)?;
    let mut summary = price_with_range(&filtered, &table, fx.as_ref(), Some(&window));
    if matches!(currency, CurrencyArg::Usd) {
        summary.currency = Currency::Usd;
    }
    println!("{}", serde_json::to_string_pretty(&summary)?);
    if print_by_model_table {
        eprintln!("--- by_model ---");
        eprintln!(
            "{:<36} {:<14} {:>12} {:>8} {:>8} {}",
            "model", "usage_pool", "amount", "in_tok", "out_tok", "priced"
        );
        for row in &summary.by_model {
            let pool = row
                .usage_pool
                .map(|p| p.as_str().to_string())
                .unwrap_or_else(|| "-".into());
            let amt = row
                .amount
                .map(|a| format!("{a:.4}"))
                .unwrap_or_else(|| "unpriced".into());
            eprintln!(
                "{:<36} {:<14} {:>12} {:>8} {:>8} {}",
                row.model, pool, amt, row.input_tokens, row.output_tokens, row.priced
            );
        }
    }
    Ok(())
}

fn run_by_pool(
    currency: CurrencyArg,
    events: Option<PathBuf>,
    range: RangeArg,
    day_boundary: DayBoundaryArg,
    timezone: String,
    fx_rate: Option<f64>,
    fx_source: String,
) -> Result<()> {
    // Full SpendSummary JSON (includes by_usage_pool); same pricing path as total.
    run_total(
        currency,
        events,
        range,
        day_boundary,
        timezone,
        fx_rate,
        fx_source,
        false,
    )
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Total {
            currency,
            events,
            range,
            day_boundary,
            timezone,
            fx_rate,
            fx_source,
            by_model,
        } => run_total(
            currency,
            events,
            range,
            day_boundary,
            timezone,
            fx_rate,
            fx_source,
            by_model,
        )?,
        Commands::ByModel {
            currency,
            events,
            range,
            day_boundary,
            timezone,
            fx_rate,
            fx_source,
        } => run_total(
            currency,
            events,
            range,
            day_boundary,
            timezone,
            fx_rate,
            fx_source,
            true,
        )?,
        Commands::ByPool {
            currency,
            events,
            range,
            day_boundary,
            timezone,
            fx_rate,
            fx_source,
        } => run_by_pool(
            currency,
            events,
            range,
            day_boundary,
            timezone,
            fx_rate,
            fx_source,
        )?,
        Commands::Today {
            currency,
            events,
            day_boundary,
            timezone,
            fx_rate,
            fx_source,
        } => run_total(
            currency,
            events,
            RangeArg::Today,
            day_boundary,
            timezone,
            fx_rate,
            fx_source,
            false,
        )?,
        Commands::Series {
            grain,
            currency,
            events,
            range,
            from,
            to,
            day_boundary,
            timezone,
            fx_rate,
            fx_source,
        } => {
            if grain != "day" {
                bail!("v0 only supports --grain day");
            }
            let table = PriceTable::embedded();
            let all_events = read_events(events.as_ref())?;
            let fx = build_fx(&currency, fx_rate, fx_source)?;
            let series = daily_spend_series(
                &all_events,
                &table,
                fx.as_ref(),
                &SeriesOpts {
                    day_boundary: day_boundary.to_boundary(),
                    timezone: Some(timezone.as_str()),
                    now: Utc::now(),
                    range_kind: range.to_kind(),
                    from: from.as_deref(),
                    to: to.as_deref(),
                },
            )?;
            println!("{}", serde_json::to_string_pretty(&series)?);
        }
        Commands::Parse { agent, file } => {
            let text = std::fs::read_to_string(&file)
                .with_context(|| format!("read {}", file.display()))?;
            let events = match agent {
                ParseAgent::ClaudeCode => parse_claude_code_jsonl_with_source(
                    &text,
                    &format!("EC-claude-code-v1:{}", file.display()),
                )
                .map_err(|e| anyhow::anyhow!(e))?,
                ParseAgent::Codex => parse_codex_rollout_jsonl_with_source(
                    &text,
                    &format!("EC-codex-v1:{}", file.display()),
                )
                .map_err(|e| anyhow::anyhow!(e))?,
                ParseAgent::Cursor => {
                    let _ = AgentId::Cursor;
                    parse_cursor_local(&text).map_err(|e| anyhow::anyhow!(e))?
                }
            };
            println!("{}", serde_json::to_string_pretty(&events)?);
        }
        Commands::Import { cmd } => match cmd {
            ImportCmd::Status { state, json: _ } => {
                let meta = read_import_meta(&state)?;
                println!("{}", serde_json::to_string_pretty(&meta)?);
            }
            ImportCmd::Run {
                state,
                events_upserted,
                parse_error_count,
                source_id,
                json: _,
            } => {
                let meta = record_import(
                    &state,
                    source_id,
                    events_upserted,
                    parse_error_count,
                    Some(state.display().to_string()),
                )?;
                println!("{}", serde_json::to_string_pretty(&meta)?);
            }
        },
    }
    Ok(())
}
