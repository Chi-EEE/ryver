mod sheet;
mod generate;

use std::{fs, path::PathBuf, process};

use calamine::{open_workbook_auto, Reader};
use clap::{arg, command, Parser};
use color_eyre::eyre::Result;
use tracing::{error, info};

use crate::sheet::Sheet;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input file
    #[arg(short, long)]
    file: PathBuf,

    /// Output path
    #[arg(short, long)]
    out: PathBuf,

    /// Sheet names
    #[arg(short, long)]
    #[clap(default_values_t = ["*".to_owned()])]
    sheet: Vec<String>,

    /// Column to be used as the table/object name
    #[arg(long)]
    #[clap(default_value_t = 0)]
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
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let mut sheets = vec![];

    match args.file.extension().and_then(|f| f.to_str()) {
        Some("xlsx") | Some("xlsm") | Some("xlsb") | Some("xls") => {
            let mut xl = open_workbook_auto(args.file)?;

            if args.sheet.len() == 1 && args.sheet[0] == "*" {
                for (name, range) in xl.worksheets() {
                    sheets.push(Sheet::excel(name, range));
                }
            } else {
                for name in args.sheet {
                    let range = xl.worksheet_range(&name)?;
                    sheets.push(Sheet::excel(name, range));
                }
            }
        }
        Some("csv") => {
            let content = fs::read_to_string(&args.file)?;

            sheets.push(Sheet::csv(false, args.file.file_name().unwrap().to_str().unwrap().to_owned(), content));
        },
        Some("tsv") => {
            let content = fs::read_to_string(&args.file)?;

            sheets.push(Sheet::csv(true, args.file.file_name().unwrap().to_str().unwrap().to_owned(), content));
        },
        _ => {
            error!("unsupported file type");
            process::exit(1);
        }
    };

    let mut files = vec![];

    if args.typescript {
        for sheet in sheets {
            let (name, content) = generate::typescript::code(Config { sheet, table_name: args.table_name, no_type: args.no_type });
            files.push((name, content));
        }
    } else {
        for sheet in sheets {
            let (name, content) = generate::luau::code(Config { sheet, table_name: args.table_name, no_type: args.no_type });
            files.push((name, content));
        }
    }

    if args.out.exists() {
        fs::remove_dir_all(&args.out)?;
    }

    fs::create_dir_all(&args.out)?;

    for (name, content) in files {
        info!("creating file: {:?}", name);
        fs::write(args.out.join(name), content)?;
    }

    Ok(())
}
