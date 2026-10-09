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

struct State {
    store: HashMap<String, String>,
    expiry: HashMap<String, i64>, // key -> absolute ms timestamp
    clock: i64,                   // simulated clock in ms
}

impl State {
    fn new() -> Self {
        State {
            store: HashMap::new(),
            expiry: HashMap::new(),
            clock: 0,
        }
    }
    fn check_expiry(&mut self, key: &str) {
        if let Some(&exp) = self.expiry.get(key) {
            if self.clock >= exp {
                self.store.remove(key);
                self.expiry.remove(key);
            }
        }
    }
}

fn incr_by(store: &mut HashMap<String, String>, key: &str, delta: i64) -> String {
    let cur = store.get(key).map(|s| s.as_str()).unwrap_or("0");
    match cur.parse::<i64>() {
        Ok(v) => {
            let n = v + delta;
            store.insert(key.into(), n.to_string());
            ei(n)
        }
        Err(_) => ee("ERR value is not an integer or out of range"),
    }
}

fn handle(args: &[String], st: &mut State) -> String {
    let cmd = args[0].to_uppercase();
    match cmd.as_str() {
        "WAIT" => {
            st.clock += args[1].parse::<i64>().unwrap_or(0);
            es("OK")
        }
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
            let (key, val) = (&args[1], &args[2]);
            let flags: Vec<String> = args[3..].iter().map(|a| a.to_uppercase()).collect();
            if flags.contains(&"NX".into()) && st.store.contains_key(key) {
                return "$-1\r\n".into();
            }
            if flags.contains(&"XX".into()) && !st.store.contains_key(key) {
                return "$-1\r\n".into();
            }
            st.store.insert(key.clone(), val.clone());
            es("OK")
        }
        "GET" => {
            st.check_expiry(&args[1]);
            eb(st.store.get(&args[1]).map(|s| s.as_str()))
        }
        "DBSIZE" => ei(st.store.len() as i64),
        "INCR" => incr_by(&mut st.store, &args[1], 1),
        "DECR" => incr_by(&mut st.store, &args[1], -1),
        "INCRBY" => {
            let d = args[2].parse::<i64>().unwrap_or(0);
            incr_by(&mut st.store, &args[1], d)
        }
        "DECRBY" => {
            let d = args[2].parse::<i64>().unwrap_or(0);
            incr_by(&mut st.store, &args[1], -d)
        }
        "EXPIRE" => {
            let (key, val) = (&args[1], &args[2]);
            st.check_expiry(&args[1]);
            if st.store.contains_key(key) {
                let d = val.parse::<i64>().unwrap_or(0);
                st.expiry.insert(key.clone(), d * 1000);
                return ei(1);
            }
            ei(0)
        }
        "TTL" => {
            let key = &args[1];
            st.check_expiry(key);
            if !st.store.contains_key(key) {
                return ei(-2);
            }
            match st.expiry.get(key) {
                Some(exp) => ei((exp - st.clock) / 1000),
                None => ei(-1),
            }
        }
        "PERSIST" => {
            let key = &args[1];
            st.check_expiry(key);
            if st.expiry.contains_key(key) {
                st.expiry.remove(key);
                return ei(1);
            }
            ei(0)
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
