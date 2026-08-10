//! `tpt-uir-cli` — validate, print, and convert TPT-UIR regions.
#![doc = include_str!("../README.md")]

use std::collections::HashMap;
use std::process::exit;

use tpt_uir_core::ir::Region;

#[derive(Clone, Copy, PartialEq)]
enum Format {
    Postcard,
    Text,
    Flatbuffers,
}

fn format_from_ext(path: &str) -> Format {
    if path.ends_with(".txt") || path.ends_with(".uir") {
        Format::Text
    } else if path.ends_with(".fbs") || path.ends_with(".flat") {
        Format::Flatbuffers
    } else {
        Format::Postcard
    }
}

fn parse_format(s: &str) -> Result<Format, String> {
    match s {
        "postcard" => Ok(Format::Postcard),
        "text" => Ok(Format::Text),
        "flatbuffers" => Ok(Format::Flatbuffers),
        other => Err(format!("unknown format '{}'", other)),
    }
}

fn load_region(path: &str, format: Option<Format>) -> Result<Region, String> {
    let fmt = format.unwrap_or_else(|| format_from_ext(path));
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {}: {}", path, e))?;
    match fmt {
        Format::Postcard => tpt_uir_serde::deserialize_region(&bytes)
            .map_err(|e| format!("postcard decode error: {}", e)),
        Format::Text => {
            tpt_uir_text::parse_text(std::str::from_utf8(&bytes).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())
        }
        Format::Flatbuffers => tpt_uir_flatbuffers::from_flatbuffer(&bytes)
            .ok_or_else(|| "invalid flatbuffer region".to_string()),
    }
}

fn save_region(path: &str, format: Option<Format>, region: &Region) -> Result<(), String> {
    let fmt = format.unwrap_or_else(|| format_from_ext(path));
    match fmt {
        Format::Postcard => tpt_uir_serde::write_tptuir(path, region)
            .map_err(|e| format!("postcard encode error: {}", e)),
        Format::Text => {
            let text = tpt_uir_text::to_text(region);
            std::fs::write(path, text).map_err(|e| format!("cannot write {}: {}", path, e))
        }
        Format::Flatbuffers => {
            let bytes = tpt_uir_flatbuffers::to_flatbuffer(region);
            std::fs::write(path, bytes).map_err(|e| format!("cannot write {}: {}", path, e))
        }
    }
}

fn cmd_validate(args: &[String]) -> Result<(), String> {
    let mut file: Option<String> = None;
    let mut dialect: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dialect" => {
                i += 1;
                dialect = Some(args.get(i).ok_or("--dialect needs a value")?.clone());
            }
            other if file.is_none() => file = Some(other.to_string()),
            _ => return Err(format!("unexpected argument '{}'", args[i])),
        }
        i += 1;
    }
    let file = file.ok_or("validate requires a <file> argument")?;
    let region = load_region(&file, None)?;

    // Always run SSA validation.
    if let Err(errs) = tpt_uir_core::validate_region(&region) {
        eprintln!("SSA validation FAILED:");
        for e in &errs {
            eprintln!("  - {}", e);
        }
        return Err("ssa".to_string());
    }

    if let Some(d) = dialect {
        use tpt_uir_dialects::ValidateDialect;
        let result = match d.as_str() {
            "gpu" => tpt_uir_dialects::GpuDialect::validate_all(&region),
            "crucible" => tpt_uir_dialects::CrucibleDialect::validate_all(&region),
            other => return Err(format!("unknown dialect '{}'", other)),
        };
        if let Err(errs) = result {
            eprintln!("{} dialect validation FAILED:", d);
            for e in &errs {
                eprintln!("  - {}", e);
            }
            return Err("dialect".to_string());
        }
    }

    println!("ok: {} valid", file);
    Ok(())
}

fn cmd_print(args: &[String]) -> Result<(), String> {
    let mut file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            other if file.is_none() => file = Some(other.to_string()),
            _ => return Err(format!("unexpected argument '{}'", args[i])),
        }
        i += 1;
    }
    let file = file.ok_or("print requires a <file> argument")?;
    let region = load_region(&file, None)?;
    print!("{}", tpt_uir_text::Pretty(&region));
    Ok(())
}

fn cmd_convert(args: &[String]) -> Result<(), String> {
    let mut opts: HashMap<String, String> = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--in" | "--out" | "--from-format" | "--to-format" => {
                let key = args[i].clone();
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| format!("{} needs a value", key))?
                    .clone();
                opts.insert(key, val);
            }
            other => return Err(format!("unexpected argument '{}'", other)),
        }
        i += 1;
    }
    let input = opts
        .get("--in")
        .ok_or("convert requires --in <file>")?
        .clone();
    let output = opts
        .get("--out")
        .ok_or("convert requires --out <file>")?
        .clone();
    let from = opts
        .get("--from-format")
        .map(|s| parse_format(s))
        .transpose()?;
    let to = opts
        .get("--to-format")
        .map(|s| parse_format(s))
        .transpose()?;

    let region = load_region(&input, from)?;
    save_region(&output, to, &region)?;
    println!("converted {} -> {}", input, output);
    Ok(())
}

fn usage() -> String {
    "usage:\n  tpt-uir-cli validate <file> [--dialect gpu|crucible]\n  tpt-uir-cli print <file>\n  tpt-uir-cli convert --in <file> --out <file> [--from-format postcard|text|flatbuffers] [--to-format postcard|text|flatbuffers]".to_string()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("{}", usage());
        exit(2);
    }
    let sub = args[1].clone();
    let rest = &args[2..];
    let result = match sub.as_str() {
        "validate" => cmd_validate(rest),
        "print" => cmd_print(rest),
        "convert" => cmd_convert(rest),
        "--help" | "-h" => {
            println!("{}", usage());
            Ok(())
        }
        other => Err(format!("unknown subcommand '{}'\n{}", other, usage())),
    };
    if let Err(e) = result {
        eprintln!("error: {}", e);
        exit(1);
    }
}
