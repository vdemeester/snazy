use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::io::{self, BufRead};
use yansi::Paint;

#[derive(Serialize, Deserialize, Debug)]
struct Pac {
    severity: String,
    timestamp: String,
    caller: String,
    message: String,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Knative {
    level: String,
    msg: String,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

pub fn getinfo(
    rawline: &str,
    kail_no_prefix: bool,
    time_format: &str,
) -> Option<HashMap<String, String>> {
    let mut msg = HashMap::new();
    let mut sample = rawline.to_string();
    let kali_re =
        Regex::new(r"^(?P<namespace>[^/]*)/(?P<pod>[^\[]*)\[(?P<container>[^]]*)]: (?P<line>.*)")
            .unwrap();
    let mut kali_msg_prefix = String::new();
    if kali_re.is_match(rawline) {
        let _result = kali_re.replace_all(rawline, "$line").to_string();
        sample = _result;
        kali_msg_prefix = kali_re
            .replace_all(rawline, "$namespace/$pod[$container]")
            .to_string();
    }
    if let Ok(p) = serde_json::from_str::<Pac>(sample.as_str()) {
        msg.insert("msg".to_string(), p.message.trim().to_string());
        msg.insert("level".to_string(), p.severity.to_uppercase());
        // parse timestamp to a unix timestamp
        msg.insert(
            "ts".to_string(),
            crate::utils::convert_str_to_ts(p.timestamp.as_str(), time_format),
        );
        let mut others = String::new();
        if p.other.contains_key("provider") {
            // append provider icon to others
            others.push_str(
                &(match p.other["provider"].as_str() {
                    Some("github") => " ".to_string(),
                    Some("gitlab") => " ".to_string(),
                    Some("bitbucket-cloud") => " ".to_string(),
                    Some("bitbucket-server") => " Server".to_string(),
                    _ => p.other["provider"].to_string(),
                }),
            );
            msg.insert("others".to_string(), format!("{} ", others));
        }
    }

    if let Ok(p) = serde_json::from_str::<Knative>(sample.as_str()) {
        msg.insert("msg".to_string(), p.msg.trim().to_string());
        msg.insert("level".to_string(), p.level.to_uppercase());
        if let Some(ts) = p.other.get("ts") {
            if ts.is_f64() {
                msg.insert(
                    "ts".to_string(),
                    crate::utils::convert_unix_ts(ts.as_f64().unwrap() as i64, time_format),
                );
            } else if ts.as_str().is_some() {
                msg.insert(
                    "ts".to_string(),
                    crate::utils::convert_str_to_ts(ts.as_str().unwrap(), time_format),
                );
            }
        };
    }
    // TODO: no prefix
    if !kail_no_prefix && !kali_msg_prefix.is_empty() && msg.contains_key("msg") {
        *msg.get_mut("msg").unwrap() = format!("{} {}", Paint::blue(kali_msg_prefix), msg["msg"])
    }
    Some(msg)
}

pub fn read_from_stdin(cli: crate::cli::Cli) {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let parseline = &line.unwrap();
        // exclude lines with only space or empty
        if parseline.trim().is_empty() {
            continue;
        }

        if let Some(msg) =
            crate::parse::getinfo(parseline, cli.kail_no_prefix, cli.time_format.as_str())
        {
            let unwrapped = serde_json::to_string(&msg).unwrap();
            //check if unwrapped is not an empty hashmap
            if unwrapped == "{}" {
                println!("{}", parseline);
                continue;
            }
            if !cli.filter_levels.is_empty()
                && !cli.filter_levels.contains(&msg["level"].to_lowercase())
            {
                continue;
            }

            let level = crate::utils::color_by_level(msg.get("level").unwrap());
            let mut ts = String::new();
            if msg.contains_key("ts") {
                ts = Paint::fixed(13, msg.get("ts").unwrap()).to_string();
            }
            let other = if msg.contains_key("others") {
                format!(" {}", Paint::cyan(msg.get("others").unwrap()).italic())
            } else {
                "".to_string()
            };
            let mut themsg = msg.get("msg").unwrap().to_string();
            if !cli.regexp.is_empty() {
                let re = Regex::new(format!(r"(?P<r>{})", cli.regexp).as_str()).unwrap();
                let _result = re
                    .replace_all(&themsg, Paint::yellow("$r").to_string())
                    .to_string();
                themsg = _result;
            }

            println!("{} {} {}{}", Paint::wrapping(level), ts, other, themsg);
        }
    }
}
