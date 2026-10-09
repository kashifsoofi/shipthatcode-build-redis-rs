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
    expiry: HashMap<String, i64>,
    clock: i64,
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
            let (key, val) = (args[1].clone(), args[2].clone());
            let mut ex_ms: Option<i64> = None;
            let mut i = 3;
            while i < args.len() {
                match args[i].to_uppercase().as_str() {
                    "EX" => {
                        ex_ms = Some(args[i + 1].parse::<i64>().unwrap_or(0) * 1000);
                        i += 2;
                    }
                    "PX" => {
                        ex_ms = Some(args[i + 1].parse::<i64>().unwrap_or(0));
                        i += 2;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            st.store.insert(key.clone(), val);
            if let Some(ms) = ex_ms {
                st.expiry.insert(key, st.clock + ms);
            }
            es("OK")
        }
        "GET" => {
            // TODO: Call check_expiry before accessing
            st.check_expiry(&args[1]);
            eb(st.store.get(&args[1]).map(|s| s.as_str()))
        }
        "EXISTS" => {
            let mut i = 3;
            let mut count = 0;
            while i < args.len() {
                st.check_expiry(&args[1]);
                if st.store.contains_key(&args[1]) {
                    count += 1
                }
                i += 1;
            }
            ei(count)
        }
        "TTL" => {
            st.check_expiry(&args[1]);
            if !st.store.contains_key(&args[1]) {
                return ei(-2);
            }
            match st.expiry.get(&args[1]) {
                None => ei(-1),
                Some(&exp) => ei(std::cmp::max(0, (exp - st.clock) / 1000)),
            }
        }
        "PTTL" => {
            st.check_expiry(&args[1]);
            if !st.store.contains_key(&args[1]) {
                return ei(-2);
            }
            match st.expiry.get(&args[1]) {
                None => ei(-1),
                Some(&exp) => ei(std::cmp::max(0, exp - st.clock)),
            }
        }
        "EXPIRE" => {
            st.check_expiry(&args[1]);
            if !st.store.contains_key(&args[1]) {
                return ei(0);
            }
            let secs = args[2].parse::<i64>().unwrap_or(0);
            st.expiry.insert(args[1].clone(), st.clock + secs * 1000);
            ei(1)
        }
        "PERSIST" => {
            if st.expiry.remove(&args[1]).is_some() {
                ei(1)
            } else {
                ei(0)
            }
        }
        "DBSIZE" => {
            // TODO: Count only non-expired keys (check expiry for each)
            for (key, _) in st.expiry.clone() {
                st.check_expiry(&key);
            }
            ei(st.store.len() as i64)
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
