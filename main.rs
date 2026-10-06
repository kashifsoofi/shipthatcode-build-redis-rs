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

fn handle(args: &[String]) -> String {
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
        _ => {
            // TODO: Return error for unknown commands using ee()
            // Format: "ERR unknown command '<cmd>'"
            ee(&format!("ERR unknown command '{}'", cmd).to_string())
        }
    }
}

fn main() {
    let sin = io::stdin();
    let sout = io::stdout();
    let mut out = sout.lock();
    for line in sin.lock().lines() {
        let line = line.unwrap();
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let args = parse_args(&line);
        write!(out, "{}", handle(&args)).unwrap();
        out.flush().unwrap();
    }
}
