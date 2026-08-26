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
        Format::Flatbuffers => {
            tpt_uir_flatbuffers::from_flatbuffer(&bytes).map_err(|e| e.to_string())
        }
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

fn cmd_fmt(args: &[String]) -> Result<(), String> {
    let mut file: Option<String> = None;
    let mut check = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--check" => check = true,
            other if file.is_none() => file = Some(other.to_string()),
            _ => return Err(format!("unexpected argument '{}'", args[i])),
        }
        i += 1;
    }
    let file = file.ok_or("fmt requires a <file> argument")?;
    if format_from_ext(&file) != Format::Text && !file.ends_with(".txt") && !file.ends_with(".uir")
    {
        return Err(format!(
            "fmt operates on textual files (*.txt / *.uir); use 'convert' for {}",
            file
        ));
    }
    let region = load_region(&file, Some(Format::Text))?;
    let canonical = tpt_uir_text::to_text(&region);
    let original =
        std::fs::read_to_string(&file).map_err(|e| format!("cannot read {}: {}", file, e))?;

    if original == canonical {
        println!("{}: already formatted", file);
        return Ok(());
    }

    if check {
        eprintln!("{}: not canonically formatted", file);
        exit(1);
    }

    std::fs::write(&file, canonical).map_err(|e| format!("cannot write {}: {}", file, e))?;
    println!("{}: reformatted", file);
    Ok(())
}

/// Describe a region structurally for diff output.
fn describe(region: &Region) -> Vec<String> {
    let mut lines = Vec::new();
    for (bi, block) in region.blocks.iter().enumerate() {
        lines.push(format!("block {} ({} ops)", bi, block.operations.len()));
        for op in &block.operations {
            lines.push(format!(
                "  op {} {} operands=[{}] results=[{}] attrs={}",
                op.id,
                op.op_name,
                op.operands
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
                op.results
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
                op.attributes.len()
            ));
            for (ri, r) in op.regions.iter().enumerate() {
                lines.push(format!(
                    "    nested region {} ({} blocks)",
                    ri,
                    r.blocks.len()
                ));
                for line in describe(r) {
                    lines.push(format!("      {}", line));
                }
            }
        }
    }
    lines
}

fn cmd_diff(args: &[String]) -> Result<(), String> {
    let mut files: [Option<String>; 2] = [None, None];
    let mut formats: HashMap<String, Format> = HashMap::new();
    let mut slot = 0usize;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format-a" | "--format-b" => {
                let key = args[i].clone();
                i += 1;
                let val = args
                    .get(i)
                    .ok_or_else(|| format!("{} needs a value", key))?
                    .clone();
                formats.insert(key, parse_format(&val)?);
            }
            other => {
                if slot >= 2 {
                    return Err(format!("unexpected argument '{}'", other));
                }
                files[slot] = Some(other.to_string());
                slot += 1;
            }
        }
        i += 1;
    }
    let a = files[0]
        .clone()
        .ok_or("diff requires <file-a> and <file-b> arguments")?;
    let b = files[1]
        .clone()
        .ok_or("diff requires <file-a> and <file-b> arguments")?;

    let region_a = load_region(&a, formats.get("--format-a").copied())?;
    let region_b = load_region(&b, formats.get("--format-b").copied())?;

    if region_a == region_b {
        println!("regions identical");
        return Ok(());
    }

    println!("regions differ:");
    let da = describe(&region_a);
    let db = describe(&region_b);
    let mut removed: Vec<&String> = da.iter().filter(|l| !db.contains(l)).collect();
    let mut added: Vec<&String> = db.iter().filter(|l| !da.contains(l)).collect();
    // Only show structural lines; skip the generic block header duplicates.
    removed.retain(|l| !l.starts_with("block "));
    added.retain(|l| !l.starts_with("block "));
    for l in removed {
        println!("- {}", l.trim_start());
    }
    for l in added {
        println!("+ {}", l.trim_start());
    }
    exit(1);
}

fn usage() -> String {
    "usage:\n  tpt-uir-cli validate <file> [--dialect gpu|crucible]\n  tpt-uir-cli print <file>\n  tpt-uir-cli convert --in <file> --out <file> [--from-format postcard|text|flatbuffers] [--to-format postcard|text|flatbuffers]\n  tpt-uir-cli fmt <file.txt|file.uir> [--check]\n  tpt-uir-cli diff <file-a> <file-b> [--format-a postcard|text|flatbuffers] [--format-b postcard|text|flatbuffers]".to_string()
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
        "fmt" => cmd_fmt(rest),
        "diff" => cmd_diff(rest),
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
