use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::{Date, Datelike, Local};
use clap::Parser;
use glob::glob;
use raf::option::RafConfig;
use rustyline::{error::ReadlineError, Editor};

#[derive(Debug, Parser)]
/// Draw your rough draft freely.
enum Args {
    /// Create a new draft directory and output the path of it.
    New {
        /// project type.
        #[clap(short, long)]
        kind: Option<String>,
        /// project slug.
        #[clap(short, long)]
        slug: Option<String>,
    },
    /// List up all draft directories under the root raf directory.
    Ls,
}

fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let raf_config_root = format!("{}/.config/raf/config.toml", std::env::var("HOME").unwrap());
    let raf_config_root = Path::new(&raf_config_root);
    let raf_config_root = std::fs::read_to_string(raf_config_root)
        .with_context(|| anyhow!("Failed to load {:?}", raf_config_root))?;
    let raf_config: RafConfig = toml::from_str(&raf_config_root)?;
    let raf_root = raf_config.path.root;

    let args = Args::parse();
    let today = Local::today();

    match args {
        Args::New { kind, slug } => {
            let config = rustyline::config::Builder::new()
                .output_stream(rustyline::OutputStreamType::Stderr)
                .build();
            let mut rl = Editor::<()>::with_config(config);
            let kind = if let Some(kind) = kind {
                kind
            } else {
                let readline = rl.readline("Project kind: ");
                match readline {
                    Ok(kind) => kind,
                    Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                        return Err(anyhow!("Interrupted!"))
                    }
                    _ => return Err(anyhow!("Unknown error.")),
                }
            };
            let slug = if let Some(slug) = slug {
                slug
            } else {
                let readline = rl.readline("Project slug: ");
                match readline {
                    Ok(slug) => slug,
                    Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                        return Err(anyhow!("Interrupted!"))
                    }
                    _ => return Err(anyhow!("Unknown error.")),
                }
            };
            let new_dir = create_new_dir(&raf_root, &kind, &today, &slug)?;
            println!("{}", new_dir.to_str().unwrap());
        }
        Args::Ls => {
            let str_root = raf_root.to_str().unwrap();
            let g = glob(&format!("{}/*/*/*/*/*", str_root))?;
            let print_func = |p: PathBuf| {
                let fname = p.file_name()?.to_str()?;
                if !fname.starts_with('.') {
                    println!("{}", p.to_str()?);
                }
                Some(())
            };
            for p in g.flatten() {
                print_func(p);
            }
        }
    }

    Ok(())
}

fn create_new_dir(root: &Path, ftype: &str, date: &Date<Local>, fname: &str) -> Result<PathBuf> {
    let fpath = format!(
        "{ftype:}/{year:04}/{month:02}/{day:02}/{fname:}",
        ftype = ftype,
        year = date.year(),
        month = date.month(),
        day = date.day(),
        fname = fname
    );
    let fpath = Path::new(&fpath);
    let fpath = root.join(fpath);

    if !fpath.exists() {
        std::fs::create_dir_all(&fpath)?;
    };
    // std::fs::File::create(&fpath).expect("Cannot create.");

    Ok(fpath)
}
