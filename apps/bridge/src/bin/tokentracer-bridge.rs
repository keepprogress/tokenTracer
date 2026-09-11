use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tokentracer_bridge::{
    config_from_fixture_root, discover_paths, list_files_for_source, resolve_path, DiscoverConfig,
    HostOs, FILES_EXPAND_DEFAULT_LIMIT,
};

#[derive(Parser, Debug)]
#[command(name = "tokentracer-bridge")]
#[command(about = "tokenTracer Windows/WSL discovery bridge (path probe POC)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Discover agent data paths (Win + WSL dual-scan). Default: no files[].
    Discover {
        /// Emit DiscoverResult JSON on stdout.
        #[arg(long, default_value_t = true)]
        json: bool,
        /// Include capped files[] in each source (discover incidental cap = 32).
        #[arg(long)]
        list_files: bool,
        /// Use fixture roots under this directory (Linux POC / CI).
        #[arg(long)]
        fixture: Option<PathBuf>,
        #[arg(long)]
        win_user_profile: Option<PathBuf>,
        #[arg(long)]
        win_appdata: Option<PathBuf>,
        #[arg(long)]
        macos_home: Option<PathBuf>,
    },
    /// Expand concrete file list for one source id (import expand path).
    /// Default --limit 500; --limit 0 = unlimited (warns if count > 5000).
    Files {
        #[arg(long)]
        source_id: String,
        /// Max files to return. Default 500. `0` = unlimited.
        #[arg(long, default_value_t = FILES_EXPAND_DEFAULT_LIMIT)]
        limit: usize,
        #[arg(long)]
        fixture: Option<PathBuf>,
        #[arg(long)]
        win_user_profile: Option<PathBuf>,
        #[arg(long)]
        win_appdata: Option<PathBuf>,
    },
}

fn build_config(
    fixture: Option<PathBuf>,
    win_user_profile: Option<PathBuf>,
    win_appdata: Option<PathBuf>,
    macos_home: Option<PathBuf>,
    list_files: bool,
) -> DiscoverConfig {
    if let Some(fx) = fixture {
        let mut cfg = config_from_fixture_root(&resolve_path(fx));
        cfg.list_files = list_files;
        return cfg;
    }
    let mut cfg = DiscoverConfig {
        host_os: if cfg!(target_os = "windows") {
            HostOs::Windows
        } else if cfg!(target_os = "macos") {
            HostOs::Macos
        } else {
            HostOs::Linux
        },
        list_files,
        ..DiscoverConfig::default()
    };
    if let Some(p) = win_user_profile {
        cfg.win_user_profile = Some(resolve_path(p));
    }
    if let Some(p) = win_appdata {
        cfg.win_appdata = Some(resolve_path(p));
    }
    if let Some(p) = macos_home {
        cfg.macos_home = Some(resolve_path(p));
    }
    if cfg.host_os == HostOs::Linux
        && cfg.win_user_profile.is_none()
        && cfg.wsl_distros.is_empty()
    {
        eprintln!("hint: on this Linux box use --fixture tests/fixtures for dual-scan POC");
    }
    cfg
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Discover {
            json: _,
            list_files,
            fixture,
            win_user_profile,
            win_appdata,
            macos_home,
        } => {
            let config = build_config(fixture, win_user_profile, win_appdata, macos_home, list_files);
            let result = discover_paths(&config);
            match serde_json::to_string_pretty(&result) {
                Ok(s) => println!("{s}"),
                Err(e) => {
                    eprintln!("serialize error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Files {
            source_id,
            limit,
            fixture,
            win_user_profile,
            win_appdata,
        } => {
            let config = build_config(fixture, win_user_profile, win_appdata, None, true);
            match list_files_for_source(&config, &source_id, limit) {
                Ok(result) => {
                    if let Some(ref w) = result.warning {
                        eprintln!("warn: {w}");
                    }
                    println!("{}", serde_json::to_string_pretty(&result).unwrap());
                }
                Err(e) => {
                    eprintln!("{}", serde_json::to_string_pretty(&e).unwrap());
                    std::process::exit(2);
                }
            }
        }
    }
}
