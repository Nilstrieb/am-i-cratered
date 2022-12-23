use color_eyre::{
    eyre::{ContextCompat, WrapErr},
    Result,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

#[derive(Serialize)]
enum BuildResult {
    Success,
    TestFail { output: String },
    TestSkipped { output: String },
    BuildFail { output: String },
}

fn builds(type_name: &str, crate_path: &Path, report: &str) -> Result<BuildResult> {
    match type_name {
        "fixed" | "test-pass" | "spurious-fixed" => return Ok(BuildResult::Success),
        _ => {}
    }

    let version_dir = fs::read_dir(crate_path)?
        .next()
        .wrap_err("no version found")??;

    let output = fs::read_to_string(crate_path.join(version_dir.file_name()).join(report))
        .wrap_err_with(|| format!("reading content for crate {crate_path:?}"))?;

    Ok(match type_name {
        "broken" => BuildResult::BuildFail { output },
        "build-fail" => BuildResult::BuildFail { output },
        "error" => BuildResult::BuildFail { output },
        "fixed" => unreachable!(),
        "regressed" => BuildResult::BuildFail { output },
        "spurious-fixed" => unreachable!(),
        "spurious-regressed" => BuildResult::BuildFail { output },
        "test-fail" => BuildResult::TestFail { output },
        "test-pass" => unreachable!(),
        "test-skipped" => BuildResult::TestSkipped { output },
        _ => panic!("invalid directory type: {type_name}"),
    })
}

fn main() -> Result<()> {
    println!("Baking JSON 🥐");

    let report = std::env::args()
        .nth(1)
        .wrap_err("first argument must be report name")?;

    let report_dirs = fs::read_dir("report").wrap_err("./report not found")?;

    let mut crate_results = HashMap::<String, BuildResult>::new();

    for dir in report_dirs {
        let dir = dir?;
        let type_name = dir.file_name();
        let type_name = type_name.to_str().unwrap();

        println!("Checking {type_name}");

        let reg_path = Path::new("report").join(&*type_name).join("reg");

        let crates = fs::read_dir(&reg_path).wrap_err("regression type is not a directory")?;

        print!("Starting checking");
        std::io::stdout().flush().unwrap();

        for krate in crates {
            let krate = krate?;
            let crate_name = krate.file_name();

            print!("\rChecking {crate_name:?}");
            std::io::stdout().flush().unwrap();

            crate_results.insert(
                crate_name
                    .to_str()
                    .wrap_err("crate_name is invalid utf8")?
                    .to_owned(),
                builds(type_name, &reg_path.join(crate_name), &report)?,
            );
        }
    }

    let mut output = BufWriter::new(File::create("output.json")?);

    serde_json::to_writer(&mut output, &crate_results)?;

    println!();

    Ok(())
}
