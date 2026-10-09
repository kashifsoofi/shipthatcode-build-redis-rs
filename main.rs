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
    store: HashMap<String, String>,
    lists: HashMap<String, VecDeque<String>>,
    key_types: HashMap<String, String>,
}
impl State {
    fn new() -> Self {
        State {
            store: HashMap::new(),
            lists: HashMap::new(),
            key_types: HashMap::new(),
        }
    }
    fn check_type(&self, key: &str, expected: &str) -> Option<String> {
        if let Some(t) = self.key_types.get(key) {
            if t != expected {
                return Some(ee(
                    "WRONGTYPE Operation against a key holding the wrong kind of value",
                ));
            }
        }
        None
    }
    fn auto_delete_list(&mut self, key: &str) {
        if let Some(lst) = self.lists.get(key) {
            if lst.is_empty() {
                self.lists.remove(key);
                self.key_types.remove(key);
            }
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
        "ECHO" => eb(Some(&args[1])),
        "COMMAND" => es("OK"),
        "SET" => {
            st.store.insert(args[1].clone(), args[2].clone());
            st.key_types.insert(args[1].clone(), "string".into());
            es("OK")
        }
        "GET" => eb(st.store.get(&args[1]).map(|s| s.as_str())),
        "LPUSH" => {
            let key = &args[1];
            if let Some(e) = st.check_type(key, "list") {
                return e;
            }
            let lst = st.lists.entry(key.clone()).or_insert_with(VecDeque::new);
            st.key_types
                .entry(key.clone())
                .or_insert_with(|| "list".into());
            for v in &args[2..] {
                lst.push_front(v.clone());
            }
            ei(lst.len() as i64)
        }
        "RPUSH" => {
            let key = &args[1];
            if let Some(e) = st.check_type(key, "list") {
                return e;
            }
            let lst = st.lists.entry(key.clone()).or_insert_with(VecDeque::new);
            st.key_types
                .entry(key.clone())
                .or_insert_with(|| "list".into());
            for v in &args[2..] {
                lst.push_back(v.clone());
            }
            ei(lst.len() as i64)
        }
        "LPOP" => {
            let key = &args[1];
            if !st.lists.contains_key(key) {
                return eb(None);
            }
            let lst = st.lists.entry(key.clone()).or_insert_with(VecDeque::new);
            let v = lst.pop_front();
            st.auto_delete_list(&key.clone());
            eb(v.as_deref())
        }
        "RPOP" => {
            let key = &args[1];
            if !st.lists.contains_key(key) {
                return eb(None);
            }
            let lst = st.lists.entry(key.clone()).or_insert_with(VecDeque::new);
            let v = lst.pop_back();
            st.auto_delete_list(&key.clone());
            eb(v.as_deref())
        }
        "LLEN" => {
            let key = &args[1];
            let lst = st.lists.entry(key.clone()).or_insert_with(VecDeque::new);
            ei(lst.len() as i64)
        }
        "LRANGE" => {
            let key = &args[1];
            if !st.lists.contains_key(key) {
                return "*0\r\n".into();
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
                return "*0\r\n".into();
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
