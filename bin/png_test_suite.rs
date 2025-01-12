use anyhow::{anyhow, Result};
use comfy_table::{Attribute, Cell, Color, Table};
use png::test_file_parser::{parse_test_file, PNGSuiteTestCase};
use png::Decoder;
use std::ffi::OsStr;
use std::{fmt, fs, panic};

#[derive(Debug)]
enum TestStatus<'a> {
    Passed,
    Incorrect,
    Panic(&'a str),
    Error(anyhow::Error),
}

impl<'a> TestStatus<'a> {
    fn color(&self) -> Color {
        match self {
            TestStatus::Passed => Color::Green,
            TestStatus::Incorrect => Color::Red,
            TestStatus::Panic(_) => Color::Red,
            TestStatus::Error(_) => Color::DarkRed,
        }
    }
}

impl<'a> fmt::Display for TestStatus<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestStatus::Passed => write!(f, "Passed"),
            TestStatus::Incorrect => write!(f, "Incorrect"),
            TestStatus::Panic(msg) => write!(f, "Panic: {:?}", msg),
            TestStatus::Error(error) => write!(f, "Error: {:?}", error),
        }
    }
}

fn bold_cell(s: &str) -> Cell {
    Cell::new(s).add_attribute(Attribute::Bold)
}

fn main() -> Result<()> {
    let mut table = Table::new();
    table.set_header(vec![
        bold_cell("File"),
        bold_cell("Description"),
        bold_cell("Status"),
    ]);

    for entry in fs::read_dir("./tests")? {
        let path = entry?.path();

        if let Some(true) = path
            .extension()
            .and_then(OsStr::to_str)
            .map(|ext| ext.eq_ignore_ascii_case("png"))
        {
            let file_name = path.file_name().unwrap();
            let PNGSuiteTestCase {
                test_desc,
                should_fail,
            } = parse_test_file(&path)?;

            let content = fs::read(&path)?;

            let png_res = panic::catch_unwind(|| Decoder::new(&content).decode());

            let status = match png_res {
                Err(panic_info) => {
                    let msg = if let Some(msg) = panic_info.downcast_ref::<&str>() {
                        msg
                    } else {
                        ""
                    };
                    TestStatus::Panic(msg)
                }
                Ok(Ok(_png)) => {
                    if should_fail {
                        TestStatus::Error(anyhow!("Failed to raise error for corrupt file"))
                    } else {
                        // todo! compare pngs with the expected binary blob
                        TestStatus::Passed
                    }
                }
                Ok(Err(msg)) => {
                    if should_fail {
                        TestStatus::Passed
                    } else {
                        TestStatus::Error(msg)
                    }
                }
            };

            table.add_row(vec![
                Cell::new(file_name.to_str().unwrap()),
                Cell::new(test_desc),
                Cell::new(&status).fg(status.color()),
            ]);
        }
    }

    println!("{table}");

    Ok(())
}
