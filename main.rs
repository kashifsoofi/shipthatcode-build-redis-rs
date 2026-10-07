use std::collections::HashMap;
use std::io::{self, BufRead, Write};

fn eb(s: Option<&str>) -> String {
    match s {
        None => "$-1\r\n".into(),
        Some(s) => format!("${}\r\n{}\r\n", s.len(), s),
    }
}
fn es(s: &str) -> String {
    format!("+{}\r\n", s)
}
fn ee(m: &str) -> String {
    format!("-{}\r\n", m)
}
fn ei(n: i64) -> String {
    format!(":{}\r\n", n)
}

fn parse_args(line: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut q = false;
    for ch in line.chars() {
        match ch {
            '"' => q = !q,
            ' ' if !q => {
                if !cur.is_empty() {
                    args.push(cur.clone());
                    cur.clear();
                }
            }
            _ => cur.push(ch),
        }
    }
    if !cur.is_empty() {
        args.push(cur);
    }
    args
}

fn handle(args: &[String], store: &mut HashMap<String, String>) -> String {
    let cmd = args[0].to_uppercase();
    match cmd.as_str() {
        "PING" => {
            if args.len() == 1 {
                es("PONG")
            } else {
                eb(Some(&args[1]))
            }
        }
        "ECHO" => eb(Some(&args[1])),
        "COMMAND" => es("OK"),
        "SET" => {
            let key = &args[1];
            let val = &args[2];
            let flags: Vec<String> = args[3..].iter().map(|a| a.to_uppercase()).collect();
            if flags.contains(&"NX".to_string()) {
                if store.contains_key(&key.to_string()) {
                    return eb(None);
                }
            }

            if flags.contains(&"XX".to_string()) {
                if !store.contains_key(&key.to_string()) {
                    return eb(None);
                }
            }
            // TODO: If NX flag present and key already exists, return "$-1\r\n"
            // TODO: If XX flag present and key does NOT exist, return "$-1\r\n"
            store.insert(key.clone(), val.clone());
            es("OK")
        }
        "GET" => eb(store.get(&args[1]).map(|s| s.as_str())),
        "DBSIZE" => ei(store.len() as i64),
        _ => ee(&format!("ERR unknown command '{}'", args[0])),
    }
}

fn main() {
    let sin = io::stdin();
    let sout = io::stdout();
    let mut out = sout.lock();
    let mut store = HashMap::new();
    for line in sin.lock().lines() {
        let line = line.unwrap();
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let args = parse_args(&line);
        write!(out, "{}", handle(&args, &mut store)).unwrap();
        out.flush().unwrap();
    }
}
