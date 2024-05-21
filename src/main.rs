mod generate;
mod sheet;

use std::{fs, path::PathBuf, process};

use calamine::{open_workbook_auto, Reader};
use clap::{arg, command, Parser};
use color_eyre::eyre::Result;
use tracing::{error, info, trace, Level};
use wildmatch::WildMatch;

use crate::sheet::Sheet;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Config file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Input file
    #[arg(short, long)]
    file: PathBuf,

    /// Output path
    #[arg(short, long)]
    out: PathBuf,

    /// Sheet names
    #[arg(short, long, default_values_t = ["*".to_owned()])]
    sheet: Vec<String>,

    /// Ignore sheet names
    #[arg(short, long)]
    ignore_sheet: Vec<String>,

    /// Column to be used as the table/object name
    #[arg(long, default_value_t = 0)]
    table_name: i32,

    /// Dont generate type
    #[arg(short, long)]
    no_type: bool,

    /// Generate ts file
    #[arg(short, long)]
    typescript: bool,

    /// Verbose mode
    #[arg(short, long)]
    verbose: bool,
}

pub struct Config {
    pub sheet: Sheet,
    pub table_name: i32,
    pub no_type: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    if args.verbose {
        tracing_subscriber::fmt().pretty().with_max_level(Level::TRACE).init();
    } else {
        tracing_subscriber::fmt()
            .without_time()
            .with_target(false)
            .with_max_level(Level::INFO)
            .init();
    }

    let mut sheets = vec![];

    match args.file.extension().and_then(|f| f.to_str()) {
        Some("xlsx") | Some("xlsm") | Some("xlsb") | Some("xls") => {
            let mut xl = open_workbook_auto(args.file)?;

            for (name, range) in xl.worksheets() {
                if args.ignore_sheet.iter().any(|s| WildMatch::new(s).matches(&name)) {
                    continue;
                }

                if args.sheet.iter().any(|s| WildMatch::new(s).matches(&name)) {
                    sheets.push(Sheet::excel(name, range));
                }
            }
        }
        Some("csv") => {
            let content = fs::read_to_string(&args.file)?;

            sheets.push(Sheet::csv(
                false,
                args.file.file_name().unwrap().to_str().unwrap().to_owned().strip_suffix(".csv").unwrap().to_owned(),
                content,
            ));
        }
        Some("tsv") => {
            let content = fs::read_to_string(&args.file)?;

            sheets.push(Sheet::csv(
                true,
                args.file.file_name().unwrap().to_str().unwrap().to_owned().strip_suffix(".tsv").unwrap().to_owned(),
                content,
            ));
        }
        _ => {
            error!("unsupported file type");
            process::exit(1);
        }
    };

    let mut files = vec![];

    if args.typescript {
        for sheet in sheets {
            let (name, content) = generate::typescript::code(Config {
                sheet,
                table_name: args.table_name,
                no_type: args.no_type,
            });
            files.push((name, content));
        }
    } else {
        for sheet in sheets {
            let (name, content) = generate::luau::code(Config {
                sheet,
                table_name: args.table_name,
                no_type: args.no_type,
            });
            files.push((name, content));
        }
    }

    if !args.out.exists() {
        fs::create_dir_all(&args.out)?;
        trace!("created output directory")
    }

    for (name, content) in files {
        if args.out.join(&name).exists() {
            fs::remove_file(args.out.join(&name))?;
            trace!("removed existing file: {:?}", name)
        }

        info!("creating file: {:?}", name);
        fs::write(args.out.join(&name), content)?;
    }

    Ok(())
}
