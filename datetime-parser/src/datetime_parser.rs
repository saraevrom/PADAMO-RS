use std::collections::HashMap;

use chrono::{offset::LocalResult, Datelike, TimeZone, Timelike};
use once_cell::sync::Lazy;
use regex::Regex;

use chrono::prelude::*;

static DATETIME_REGEXES:Lazy<Vec<Regex>> = Lazy::new(||{
    let mut regexes = Vec::new();
    regexes.push(Regex::new(r"^\s+").unwrap());
    regexes.push(Regex::new(r"^(?<Y>\d{4})-(?<m>\d{2})-(?<d>\d{2})").unwrap());
    regexes.push(Regex::new(r"^(?<H>\d{2}):(?<M>\d{2}):(?<S>\d{2})\.(?<ms>\d+)").unwrap());
    regexes.push(Regex::new(r"^(?<H>\d{2}):(?<M>\d{2}):(?<S>\d{2})").unwrap());
    regexes.push(Regex::new(r"^h\s*=\s*(?<H>\d+)").unwrap());
    regexes.push(Regex::new(r"^m\s*=\s*(?<M>\d+)").unwrap());
    regexes.push(Regex::new(r"^s\s*=\s*(?<S>\d+)").unwrap());
    regexes.push(Regex::new(r"^ms\s*=\s*(?<ms>\d+)").unwrap());
    regexes.push(Regex::new(r"^Y\s*=\s*(?<Y>\d+)").unwrap());
    regexes.push(Regex::new(r"^M\s*=\s*(?<m>\d+)").unwrap());
    regexes.push(Regex::new(r"^D\s*=\s*(?<d>\d+)").unwrap());
    regexes
});


pub fn parse_datetimes(input:&str,init_datetime:chrono::DateTime<chrono::Utc>)->Option<chrono::DateTime<chrono::Utc>>{
    let mut i = 0;
    let mut res:HashMap<String, String> = HashMap::new();
    while i<input.len(){
        let mut found = false;
        for re in DATETIME_REGEXES.iter(){
            if let Some(cap) = re.captures(&input[i..]){
                println!("Found {:?}", cap);
                let end = i+cap.get(0).unwrap().len();
                println!("i: {}->{}",i,end);

                found = true;
                i = end;
                for part in "Y m d H M S ms dd dH dM dS".split_whitespace(){
                    if let Some(grp) = cap.name(part){
                        res.insert(part.to_string(),grp.as_str().into());
                    }
                }
                break;
            }
        }
        if !found{
            println!("Got invalid token at {}",i);
            return None;
        }
    }
    println!("Got data {:?}", res);
    let year = res.get("Y").map(|x| x.clone()).unwrap_or(init_datetime.year().to_string());
    let month = res.get("m").map(|x| x.clone()).unwrap_or(init_datetime.month().to_string());
    let day = res.get("d").map(|x| x.clone()).unwrap_or(init_datetime.day().to_string());

    let hour = res.get("H").map(|x| x.clone()).unwrap_or(init_datetime.hour().to_string());
    let minute = res.get("M").map(|x| x.clone()).unwrap_or(init_datetime.minute().to_string());
    let second = res.get("S").map(|x| x.clone()).unwrap_or(init_datetime.second().to_string());
    let millisecond = res.get("ms").map(|x| x.clone()).unwrap_or("0".into());

    let year:i32 = if let Ok(v) = year.parse() {v} else {println!("Cannot parse year"); return None;};
    let month:u32 = if let Ok(v) = month.parse() {v} else {println!("Cannot parse month"); return None;};
    let day:u32 = if let Ok(v) = day.parse() {v} else {println!("Cannot parse day"); return None;};

    let hour:u32 = if let Ok(v) = hour.parse() {v} else {println!("Cannot parse hour"); return None;};
    let minute:u32 = if let Ok(v) = minute.parse() {v} else {println!("Cannot parse minute"); return None;};
    let second:u32 = if let Ok(v) = second.parse() {v} else {println!("Cannot parse second"); return None;};
    let millisecond:u32 = if let Ok(v) = millisecond.parse() {v} else {println!("Cannot parse millisecond"); return None;};


    //let end_date = Utc.ymd_opt(year, month, day).and_hms_milli_opt(hour, minute, second, millisecond);
    let end_date = Utc.with_ymd_and_hms(year, month, day, hour, minute, second);
    let end_date = if let LocalResult::Single(v) = end_date {v} else{println!("Cannot make UTC time"); return None;};
    let end_date = end_date.with_nanosecond(millisecond*1000_000);
    //    .map(|x| x.with_nanosecond(millisecond*1000_000));
    // let end_date = chrono::Utc::now().with_year(year.parse().unwrap());
    // let end_date = if let Some(d) = end_date {d} else{return None;};
    //
    // let end_date = end_date.with_month(month.parse().unwrap());
    // let end_date = if let Some(d) = end_date {d} else{return None;};
    //
    // let end_date = end_date.with_day(day.parse().unwrap());
    // let end_date = if let Some(d) = end_date {d} else{return None;};
    //
    //
    // let end_date = end_date.with_hour(hour.parse().unwrap());
    // let end_date = if let Some(d) = end_date {d} else{return None;};
    //
    // let end_date = end_date.with_minute(minute.parse().unwrap());
    // let end_date = if let Some(d) = end_date {d} else{return None;};
    //
    // let end_date = end_date.with_second(second.parse().unwrap());
    // let end_date = if let Some(d) = end_date {d} else{return None;};
    //
    // let nanosec:u32 = millisecond.parse::<u32>().unwrap()*1000_000;
    //
    // let end_date = end_date.with_nanosecond(nanosec);
    // let end_date = if let Some(d) = end_date {d} else{return None;};
    println!("DATETIME SET {:?}", end_date);
    end_date

}
