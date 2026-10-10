use std::collections::{HashMap, VecDeque};
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
fn ea(items: &[String]) -> String {
    let mut r = format!("*{}\r\n", items.len());
    for i in items {
        r.push_str(i);
    }
    r
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

struct State {
    lists: HashMap<String, VecDeque<String>>,
    key_types: HashMap<String, String>,
}
impl State {
    fn new() -> Self {
        State {
            lists: HashMap::new(),
            key_types: HashMap::new(),
        }
    }
}

fn handle(args: &[String], st: &mut State) -> String {
    let cmd = args[0].to_uppercase();
    match cmd.as_str() {
        "PING" => {
            if args.len() == 1 {
                es("PONG")
            } else {
                eb(Some(&args[1]))
            }
        }
        "RPUSH" => {
            let key = &args[1];
            let lst = st.lists.entry(key.clone()).or_insert_with(VecDeque::new);
            st.key_types
                .entry(key.clone())
                .or_insert_with(|| "list".into());
            for v in &args[2..] {
                lst.push_back(v.clone());
            }
            ei(lst.len() as i64)
        }
        "LPUSH" => {
            let key = &args[1];
            let lst = st.lists.entry(key.clone()).or_insert_with(VecDeque::new);
            st.key_types
                .entry(key.clone())
                .or_insert_with(|| "list".into());
            for v in &args[2..] {
                lst.push_front(v.clone());
            }
            ei(lst.len() as i64)
        }
        "LRANGE" => {
            let key = &args[1];
            if !st.lists.contains_key(key) {
                return ea(&Vec::new());
            }
            let lst: Vec<String> = st.lists[key].iter().cloned().collect();
            let len = lst.len() as i64;
            let mut s = args[2].parse::<i64>().unwrap_or(0);
            let mut e = args[3].parse::<i64>().unwrap_or(0);
            if s < 0 {
                s = std::cmp::max(0, len + s);
            }
            if e < 0 {
                e = len + e;
            }
            let s = s as usize;
            let e = std::cmp::min(e as usize + 1, lst.len());
            if s >= e {
                return ea(&Vec::new());
            }
            let items: Vec<String> = lst[s..e].iter().map(|x| eb(Some(x))).collect();
            ea(&items)
        }
        _ => ee(&format!("ERR unknown command '{}'", args[0])),
    }
}

fn main() {
    let sin = io::stdin();
    let sout = io::stdout();
    let mut out = sout.lock();
    let mut st = State::new();
    for line in sin.lock().lines() {
        let line = line.unwrap();
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let args = parse_args(&line);
        write!(out, "{}", handle(&args, &mut st)).unwrap();
        out.flush().unwrap();
    }
}
