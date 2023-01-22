use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::{Date, Datelike, Local};
use clap::Parser;
use dialoguer::console::Term;
use fs_extra::dir::CopyOptions;
use glob::glob;
use itertools::Itertools;
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
        /// template path.
        #[clap(short, long)]
        template: Option<String>,
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
        Args::New {
            kind,
            slug,
            template,
        } => {
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

            let template_dir = if let Some(template) = template {
                let template_dir = raf_config.path.template.join(template);
                if template_dir.is_dir() {
                    Some(template_dir)
                } else {
                    eprintln!("[WARN] template directory '{template_dir:?}' not found. createing empty directory.");
                    None
                }
            } else {
                let g = glob(&format!(
                    "{}/{kind}/*",
                    raf_config.path.template.to_str().unwrap()
                ))?;
                let items = g.flatten().map(|p| p.to_str().unwrap().to_owned());
                let mut items = Some("Do not use template".to_string())
                    .into_iter()
                    .chain(items)
                    .collect_vec();
                if items.len() > 1 {
                    eprintln!("Select template to use:");
                    let selection = dialoguer::Select::new()
                        .items(&items)
                        .default(0)
                        .interact_on_opt(&Term::stderr())?;
                    match selection {
                        Some(selection) if selection > 0 => {
                            let path = items.swap_remove(selection);
                            eprintln!("Uses template {path}");
                            Some(PathBuf::from(path))
                        }
                        _ => None,
                    }
                } else {
                    None
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

            if let Some(template_dir) = template_dir {
                let new_dir = copy_template(&raf_root, &kind, &template_dir, &today, &slug)?;
                println!("{}", new_dir.to_str().unwrap());
            } else {
                let new_dir = create_new_dir(&raf_root, &kind, &today, &slug)?;
                println!("{}", new_dir.to_str().unwrap());
            }
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

fn copy_template(
    root: &Path,
    ftype: &str,
    template_dir: &Path,
    date: &Date<Local>,
    fname: &str,
) -> Result<PathBuf> {
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

    let parent_dir = fpath.parent().expect("cannot find parent directory");

    if !parent_dir.exists() {
        std::fs::create_dir_all(parent_dir)?;
    };
    // std::fs::File::create(&fpath).expect("Cannot create.");
    fs_extra::dir::copy(
        template_dir,
        &fpath,
        &CopyOptions {
            overwrite: true,
            skip_exist: true,
            copy_inside: true,
            ..Default::default()
        },
    )?;

    Ok(fpath)
}
