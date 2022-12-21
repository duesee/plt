use std::env::args;

use plt::{dependencies::Dependencies, parse, Definition};
use regex::Regex;

fn main() -> std::io::Result<()> {
    let data = {
        let raw =
            std::fs::read_to_string(args().nth(1).expect("USAGE: mls_pl_graph file [gml|gv]"))?;

        let comment_single_line = Regex::new("//.*\\n").unwrap();
        let raw = comment_single_line.replace_all(&raw, "");

        let comment_multi_line = Regex::new("/\\*(.|\\n)*?\\*/").unwrap();
        comment_multi_line.replace_all(&raw, "").to_string()
    };

    let definitions = parse(&data).unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(1);
    });

    let format = args().nth(2).unwrap_or(String::from("gml"));

    match format.as_ref() {
        "gml" => print_gml(definitions),
        "gv" => print_gv(definitions),
        other => {
            eprintln!("Unknown format \"{}\". Try \"gml\" or \"gv\".", other);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_gml(definitions: Vec<Definition>) {
    println!("graph [");

    for definition in definitions.iter() {
        println!(
            "\tnode [id \"{}\" label \"{}\"]",
            definition.name(),
            definition.name()
        );
    }

    for definition in definitions.iter() {
        for dep in definition.dependencies() {
            println!(
                "\tedge [source \"{}\" target \"{}\"]",
                definition.name(),
                dep
            );
        }
    }

    println!("]");
}

fn print_gv(definitions: Vec<Definition>) {
    println!("digraph {{");
    println!("\tcompound=true;");
    println!("\toverlap=scalexy;");
    println!("\tsplines=true;");
    println!("\tlayout=neato;\n");
    for definition in definitions.iter() {
        let deps = definition
            .dependencies()
            .iter()
            .map(|name| name.to_string())
            .collect::<Vec<_>>();

        println!("\t{} -> {{{}}}", definition.name(), deps.join(" "));
    }
    println!("}}");
}
