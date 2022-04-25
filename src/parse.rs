use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::io::{self, BufRead};
use yansi::{Color, Paint, Style}; // 0.6.5

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
    time_format: Option<&str>,
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
            crate::utils::convert_str_to_ts(p.timestamp.as_str(), time_format.unwrap()),
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
                    crate::utils::convert_unix_ts(
                        ts.as_f64().unwrap() as i64,
                        time_format.unwrap(),
                    ),
                );
            } else if ts.as_str().is_some() {
                msg.insert(
                    "ts".to_string(),
                    crate::utils::convert_str_to_ts(ts.as_str().unwrap(), time_format.unwrap()),
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_line() {
        let line = r#"{"severity":"INFO","timestamp":"2022-04-25T10:24:30.155404234Z","logger":"pipelinesascode","caller":"kubeinteraction/secrets.go:114","message":"hello moto"}"#;
        let msg = super::getinfo(line, false, Some("%H:%M")).unwrap();
        assert_eq!(msg["msg"], "hello moto");
    }
    #[test]
    fn test_kail_prefix() {
        let line = r#"ns/pod[container]: {"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":"updated","provider":"github","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#;
        let msg = super::getinfo(line, false, Some("%H:%M")).unwrap();
        assert!(msg["msg"].contains("ns/pod[container]"));
        assert!(msg["msg"].contains("updated"));
    }
    #[test]
    fn test_kail_no_prefix() {
        let line = r#"ns/pod[container]: {"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":" updated","provider":"github","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#;
        let msg = super::getinfo(line, true, Some("%H:%M")).unwrap();
        assert_eq!(msg["msg"], "updated");
    }
    #[test]
    fn test_pac_provider_icon() {
        let line = r#"ns/pod[container]: {"severity":"INFO","timestamp":"2022-04-25T14:20:32.505637358Z","logger":"pipelinesascode","caller":"pipelineascode/status.go:59","message":" github","provider":"github","event":"8b400490-c4a1-11ec-9219-63bc5bbc8228"}"#;
        let msg = super::getinfo(line, true, Some("%H:%M")).unwrap();
        assert!(msg.contains_key("others"));
        assert!(msg["others"].contains(" "));
    }
}

pub fn read_from_stdin(matches: &clap::ArgMatches) {
    let stdin = io::stdin();
    // check if filter-levels is specified
    let mut filter_levels = Vec::new();
    let mut regexp_colours = HashMap::new();

    if matches.is_present("filter-levels") {
        filter_levels = matches
            .value_of("filter-levels")
            .unwrap()
            .split(',')
            .map(|s| s.to_string())
            .collect();
    }
    if matches.occurrences_of("regexp") > 0 {
        let colours = vec![
            Color::Yellow,
            Color::Magenta,
            Color::Cyan,
            Color::Red,
            Color::Blue,
        ];
        let regexps: Vec<&str> = matches.values_of("regexp").unwrap().collect();
        // assign a colour to each regexp
        for (i, regexp) in regexps.iter().enumerate() {
            regexp_colours.insert(regexp.to_string(), colours[i % colours.len()]);
        }
    }
    for line in stdin.lock().lines() {
        let parseline = &line.unwrap();
        // exclude lines with only space or empty
        if parseline.trim().is_empty() {
            continue;
        }

        if let Some(msg) = crate::parse::getinfo(
            parseline,
            matches.is_present("kail-no-prefix"),
            matches.value_of("time_format"),
        ) {
            let unwrapped = serde_json::to_string(&msg).unwrap();
            //check if unwrapped is not an empty hashmap
            if unwrapped == "{}" {
                println!("{}", parseline);
                continue;
            }

            if !filter_levels.is_empty() && !filter_levels.contains(&msg["level"].to_lowercase()) {
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

            if matches.occurrences_of("regexp") > 0 {
                let regexps: Vec<&str> = matches.values_of("regexp").unwrap().collect();
                for r in regexps {
                    let re = Regex::new(format!(r"(?P<r>{})", r).as_str()).unwrap();
                    let style = Style::new(regexp_colours[r]);
                    let _result = re
                        .replace_all(&themsg, style.paint("$r").to_string())
                        .to_string();
                    themsg = _result;
                }
            }

            println!("{} {} {}{}", Paint::wrapping(level), ts, other, themsg);
        }
    }
}
