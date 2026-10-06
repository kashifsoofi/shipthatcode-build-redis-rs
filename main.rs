use std::collections::HashMap;
use std::io::{self, Read, Write};

fn encode_bulk(s: Option<&str>) -> Vec<u8> {
    match s {
        None => b"$-1\r\n".to_vec(),
        Some(v) => format!("${}\r\n{}\r\n", v.len(), v).into_bytes(),
    }
}
fn encode_simple(s: &str) -> Vec<u8> {
    format!("+{}\r\n", s).into_bytes()
}
fn encode_error(s: &str) -> Vec<u8> {
    format!("-{}\r\n", s).into_bytes()
}

fn handle(store: &mut HashMap<String, String>, args: &[String]) -> Vec<u8> {
    let cmd = args[0].to_uppercase();
    // println!("cmd: {}", cmd.clone());
    match cmd.as_str() {
        "PING" => {
            if args.len() == 1 {
                encode_simple("PONG")
            } else {
                encode_bulk(Some(&args[1]))
            }
        }
        "ECHO" => encode_bulk(Some(&args[1])),
        "SET" => {
            store.insert(args[1].clone(), args[2].clone());
            encode_simple("OK")
        }
        "GET" => encode_bulk(store.get(&args[1]).map(|s| s.as_str())),
        _ => encode_error(&format!("ERR unknown command '{}'", cmd)),
    }
}

fn extract_line(buf: &[u8], pos: usize) -> Option<(Vec<u8>, usize)> {
    let mut new_pos = pos;
    if new_pos >= buf.len() {
        return None;
    }

    let mut separator_found = false;

    let mut previous: u8 = buf[new_pos];
    new_pos += 1;
    for &current in buf[new_pos..].iter() {
        new_pos += 1;

        if previous == b'\r' && current == b'\n' {
            separator_found = true;
            break;
        }

        previous = current;
    }

    if !separator_found {
        return None;
    }

    Some((buf[pos..(new_pos-2)].to_vec(), new_pos))
}

// Parse one RESP array starting at byte offset `pos`. Return Some((args, new_pos)) or None.
fn parse_request(buf: &[u8], pos: usize) -> Option<(Vec<String>, usize)> {

    let (header, new_pos) = extract_line(buf, pos)?;

    let mut args = Vec::new();
    let mut cur_pos = new_pos;

    if header[0] == b'*' {
        let n = String::from_utf8(header[1..].to_vec()).unwrap();
        let n = n.parse::<i32>().unwrap();
        for _ in 0..n {
            let (_arg, new_pos) = extract_line(buf, cur_pos)?;

            let (arg, new_pos) = extract_line(buf, new_pos)?;
            let arg = String::from_utf8(arg).unwrap();
            args.push(arg);
            cur_pos = new_pos;
        }
    }
    Some((args, cur_pos))
}

fn main() {
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf).unwrap();
    let mut store: HashMap<String, String> = HashMap::new();
    let mut pos = 0;
    let stdout = io::stdout();
    let mut out = stdout.lock();
    while let Some((args, new_pos)) = parse_request(&buf, pos) {
        out.write_all(&handle(&mut store, &args)).unwrap();
        pos = new_pos;
    }
}
