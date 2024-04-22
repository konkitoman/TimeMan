use std::{collections::BTreeMap, fmt::Write, str::FromStr};

use chrono::{Offset, TimeZone};
use clap::{Parser, Subcommand};

/// A simple date and time manipulator
/// You can get the time, get how much time elapsed sice,
/// subtract time to get the duration and add/substrat duration from date and time
///
/// Most of the work is done by clap and chrono
#[derive(Parser)]
#[command(version, author)]
pub struct TimeMan {
    /// The date format
    #[arg(short = 'f', default_value = "%a, %d %b %Y %T %z")]
    format: String,

    /// UTC/Universal
    #[arg(short = 'o')]
    utc_offset: Option<String>,

    #[command(subcommand)]
    command: TimeManCommand,
}

#[derive(Subcommand)]
pub enum TimeManCommand {
    /// get the current time you can use `-o` before to set the utc offset
    Now,

    /// alias: s
    #[command(alias = "s")]
    Since {
        date: String,

        duration_flags: Option<String>,

        #[arg(short)]
        preety: bool,
    },

    /// alias: -
    #[command(alias = "-")]
    Sub {
        from_date: String,
        date: String,

        duration_flags: Option<String>,

        #[arg(short)]
        preety: bool,
    },

    /// alias: -d
    #[command(alias = "-d")]
    SubDuration {
        from_date: String,
        duration: String,
    },

    /// alias: +d
    #[command(alias = "+d")]
    AddDuration {
        from_date: String,
        duration: String,
    },

    /// alias: t
    #[command(alias = "t")]
    Translate {
        date: String,
        #[arg(short = 'F')]
        to_format: Option<String>,
        #[arg(short = 'O')]
        offset: Option<String>,
    },
    HelpFormat {
        get_or_search: Option<String>,
    },
    HelpDuration,
}

fn main() {
    let time_man = TimeMan::parse();

    let utc_offset = time_man
        .utc_offset
        .map(|offset| {
            if let Ok(offset) = chrono::FixedOffset::from_str(&offset) {
                offset
            } else {
                eprintln!("The offset should look like \"+00:00\"");
                std::process::exit(1)
            }
        })
        .unwrap_or(chrono::Local::now().offset().fix());

    let format = if let Ok(format) =
        chrono::format::strftime::StrftimeItems::new(&time_man.format).parse()
    {
        format
    } else {
        eprintln!("Invalid format, run command `help-format`");
        std::process::exit(1);
    };

    let offset = chrono::FixedOffset::from_offset(&utc_offset);

    match time_man.command {
        TimeManCommand::Now => {
            let now = offset.from_utc_datetime(&chrono::Utc::now().naive_utc());
            let date = now.format_with_items(format.iter());
            println!("{date}");
        }
        TimeManCommand::Since {
            date,
            preety,
            duration_flags,
        } => {
            let date = parse_date(&format, &time_man.format, &date, "date");
            let now = offset.from_utc_datetime(&chrono::Utc::now().naive_utc());

            let since = now - date;
            let buf = timedelta_to_str(
                since,
                duration_flags
                    .map(|str| TimedeltaFlags::new(&str))
                    .unwrap_or(TimedeltaFlags::all()),
            );

            if preety {
                println!("{}", timedelta_str_to_preety(&buf));
            } else {
                println!("{}", buf);
            }
        }
        TimeManCommand::Sub {
            from_date,
            date,
            preety,
            duration_flags,
        } => {
            let from_date = parse_date(&format, &time_man.format, &from_date, "from_date");
            let date = parse_date(&format, &time_man.format, &date, "date");

            let res = from_date - date;
            let buf = timedelta_to_str(
                res,
                duration_flags
                    .map(|str| TimedeltaFlags::new(&str))
                    .unwrap_or(TimedeltaFlags::all()),
            );

            if preety {
                println!("{}", timedelta_str_to_preety(&buf));
            } else {
                println!("{}", buf);
            }
        }
        TimeManCommand::SubDuration {
            from_date,
            duration,
        } => {
            let from_date = parse_date(&format, &time_man.format, &from_date, "from_date");
            let Some(duration) = timedelta_from_str(&duration) else {
                eprintln!("Invalid duration!");
                std::process::exit(10)
            };

            let res = (from_date - duration).format_with_items(format.iter());
            println!("{res}");
        }
        TimeManCommand::AddDuration {
            from_date,
            duration,
        } => {
            let from_date = parse_date(&format, &time_man.format, &from_date, "from_date");
            let Some(duration) = timedelta_from_str(&duration) else {
                eprintln!("Invalid duration!");
                std::process::exit(10)
            };

            let res = (from_date + duration).format_with_items(format.iter());
            println!("{res}");
        }
        TimeManCommand::Translate {
            date,
            to_format,
            offset,
        } => {
            let date = parse_date(&format, &time_man.format, &date, "date");
            let mut format = format;

            if let Some(to_format) = &to_format {
                let Ok(f) = chrono::format::strftime::StrftimeItems::new(to_format).parse() else {
                    eprintln!("Invalid to_format, look at `format-help`");
                    std::process::exit(11);
                };
                format = f;
            }

            if let Some(offset) = offset {
                let offset = if let Ok(offset) = chrono::FixedOffset::from_str(&offset) {
                    offset
                } else {
                    eprintln!("The offset should look like \"+00:00\"");
                    std::process::exit(1)
                };

                let t = offset.from_utc_datetime(&date.naive_utc());
                println!("{}", t.format_with_items(format.iter()));
                return;
            }

            println!("{}", date.format_with_items(format.iter()));
        }
        TimeManCommand::HelpFormat { get_or_search } => {
            let mut items = BTreeMap::new();

            items.insert(
                "%A",
                r#"Full day of the week names.

Prints a full name in the title case, reads either a short or full name in any case."#,
            );

            items.insert(
                "%B",
                r#"Full month names.

Prints a full name in the title case, reads either a short or full name in any case."#,
            );

            items.insert(
                "%C",
                r#"Gregorian year divided by 100. Implies the non-negative year."#,
            );

            items.insert(
                "%D",
                r#"Date in format: 4/22/2024

Same as format: "%m/%d/%Y""#,
            );
            items.insert(
                "%F",
                r#"Date in format: 2024-4-22

Same as format: "%Y-%m-%d""#,
            );
            items.insert("%G", r#"IsoYear 2024 or 100BCE"#);
            items.insert("%H", r#"Hour 0-24"#);
            items.insert("%I", r#"Hour 0-12 zero pad like: 06"#);
            items.insert("%M", r#"Minute 0-59"#);
            items.insert("%P", r#"pm/am"#);
            items.insert(
                "%R",
                r#"Hour 0-24 and minute like: 17:00

Same as format: "%H:%M""#,
            );
            items.insert("%S", r#"Second 0-59 zero pad like: 06"#);
            items.insert(
                "%T",
                r#"Time like: 17:46:05

Same as format: "%H:%M:%S""#,
            );
            items.insert("%U", r#"Week of the year like: 16"#);
            items.insert("%V", r#"ISO Week of the year like: 17"#);
            items.insert("%Y", r#"Year like: 2024"#);
            items.insert(
                "%Z",
                r#"Time zone name like: +03:00, don't use!

This cannot be parsed use "%:z""#,
            );
            items.insert("%a", r#"Short name of the day of the week, is 3 letters"#);
            items.insert("%b", r#"Short name of month"#);
            items.insert("%h", r#"Short name of month"#);
            items.insert("%d", r#"Day of the month zero pad like: 07"#);
            items.insert("%e", r#"Day of the month space pad like:  7"#);
            items.insert("%f", r#"Nanoseconds zero pad like: 000000007"#);
            items.insert("%g", r#"Year mod 100 like: 24"#);
            items.insert("%j", r#"Day of the year zero pad like: 013"#);
            items.insert("%k", r#"Hour 24 space pad"#);
            items.insert("%l", r#"Hour 12 space pad"#);
            items.insert("%m", r#"Month space pad"#);
            items.insert("%n", r#"New line like "\n""#);
            items.insert("%p", r#"AM/PM"#);
            items.insert("%r", r#"Time like: 07:08:29 PM and fallback to 18:08:29"#);
            items.insert(
                "%s",
                r#"Timestamp

The number of non-leap seconds since the midnight UTC on January 1, 1970.
For formatting, it assumes UTC upon the absence of time zone offset."#,
            );
            items.insert("%t", r#"tab like: \t"#);
            items.insert(
                "%u",
                r#"Day of the week, where Monday = 1 as Sunday = 7 like: 1"#,
            );
            items.insert(
                "%v",
                r#"Date like: 22-Apr-2024

Same as format: "%d-%b-%Y""#,
            );
            items.insert(
                "%w",
                r#"Day of the week, where Sunday = 0 and Saturday = 6 like: 1"#,
            );
            items.insert("%y", r#"Year mod 100"#);
            items.insert(
                "%+",
                r#"Date and time like: 2024-04-22T18:20:29.306665267+03:00

Is from RFC3339
Same as format: "%FT%T%.9f%:z""#,
            );
            items.insert("%:z", r#"Timezone offset like: +03:00"#);
            items.insert("%::z", r#"Timezone offset like: +03:00:00"#);
            items.insert("%:::z", r#"Timezone offset like: +03"#);
            items.insert("%.3f", r#"Nanoseconds 3 digits like: .467"#);
            items.insert("%.6f", r#"Nanoseconds 6 digits like: .467312"#);
            items.insert("%.9f", r#"Nanoseconds 9 digits like: .432467312"#);
            items.insert("%3f", r#"Nanoseconds 3 digits like: 467"#);
            items.insert("%6f", r#"Nanoseconds 6 digits like: 467312"#);
            items.insert("%9f", r#"Nanoseconds 9 digits like: 432467312"#);
            items.insert("%%", r#"% like: %"#);

            if let Some(get_or_search) = get_or_search {
                let get_or_search = get_or_search.trim();
                if let Some(item) = items.get(get_or_search) {
                    println!("{get_or_search} : {item}");
                } else {
                    let items = items
                        .iter()
                        .filter(|(_, item)| item.to_lowercase().contains(get_or_search))
                        .collect::<Vec<_>>();
                    let pad = items.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
                    let mut pad_str = String::default();
                    for (item, description) in items.iter() {
                        pad_str.clear();
                        for _ in item.len()..pad {
                            pad_str.push(' ')
                        }
                        println!(
                            "{item}{pad_str} : {}",
                            description.lines().next().unwrap_or("")
                        );
                    }
                }
            } else {
                let pad = items.keys().map(|k| k.len()).max().unwrap_or(0);
                let mut pad_str = String::default();
                for (item, description) in items.iter() {
                    pad_str.clear();
                    for _ in item.len()..pad {
                        pad_str.push(' ')
                    }
                    println!(
                        "{item}{pad_str} : {}",
                        description.lines().next().unwrap_or("")
                    );
                }
            }
        }
        TimeManCommand::HelpDuration => {
            println!(
                r#"This is only for the content of the duration
Valid flags are:
Y : Year
M : Month
D : Day
h : Hour
m : minute
s : second
n : nanosecond

They are used like:
"YMDhmsn" this means that everything is included in duration
"sn" this means only the seconds and nanoseconds are included but everything is stored in seconds and nanoseconds

The recommended duration flags are "sn"
                "#
            );
        }
    }
}

pub fn parse_date(
    format: &[chrono::format::Item],
    format_str: &str,
    date: &str,
    field: &str,
) -> chrono::DateTime<chrono::FixedOffset> {
    let mut parsed = chrono::format::Parsed::new();
    let Ok(_) = chrono::format::parse(&mut parsed, date, format.iter()) else {
        eprintln!("Cannot parse `{field}` the date should be in this format: `{format_str}` ");
        std::process::exit(5)
    };
    let Ok(offset) = parsed.to_fixed_offset() else {
        eprintln!("Cannot parse the timeoffset for `{field}` or you don't have a format with `%:z` in it!");
        std::process::exit(6)
    };

    let Ok(time) = parsed.to_naive_datetime_with_offset(0) else {
        eprintln!("`{field}` has a invalid date!");
        std::process::exit(7);
    };

    let chrono::LocalResult::Single(time) = offset.from_local_datetime(&time) else {
        eprintln!("`{field}` has a invalid date or ambiguous time!");
        std::process::exit(8);
    };

    time
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimedeltaFlags(u8);

impl core::ops::BitOr<Self> for TimedeltaFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl Default for TimedeltaFlags {
    fn default() -> Self {
        Self::SECOND | Self::NANOS
    }
}

impl TimedeltaFlags {
    const YEAR: Self = Self(1 << 0);
    const MONTH: Self = Self(1 << 1);
    const WEEK: Self = Self(1 << 2);
    const DAY: Self = Self(1 << 3);
    const HOUR: Self = Self(1 << 4);
    const MINUTE: Self = Self(1 << 5);
    const SECOND: Self = Self(1 << 6);
    const NANOS: Self = Self(1 << 7);

    pub fn empty() -> Self {
        Self(0)
    }

    pub fn all() -> Self {
        Self::YEAR
            | Self::MONTH
            | Self::WEEK
            | Self::DAY
            | Self::HOUR
            | Self::MINUTE
            | Self::SECOND
            | Self::NANOS
    }

    pub fn contains(self, rhs: Self) -> bool {
        self.0 & rhs.0 == rhs.0
    }

    pub fn new(str: &str) -> Self {
        let mut out = Self::empty();
        for char in str.chars() {
            match char {
                'Y' => out = out | Self::YEAR,
                'M' => out = out | Self::MONTH,
                'D' => out = out | Self::DAY,
                'h' => out = out | Self::HOUR,
                'm' => out = out | Self::MINUTE,
                's' => out = out | Self::SECOND,
                'n' => out = out | Self::NANOS,
                _ => {}
            }
        }
        out
    }
}

const YEAR_IN_SECONDS: i64 = 31_536_000;
const MONTH_IN_SECONDS: i64 = YEAR_IN_SECONDS / 12;
const WEAK_IN_SECONDS: i64 = 604800;
const DAY_IN_SECONDS: i64 = 86400;
const HOUR_IN_SECONDS: i64 = 3600;
const MINUTE_IN_SECONDS: i64 = 60;

fn timedelta_to_str(timedelta: chrono::TimeDelta, flags: TimedeltaFlags) -> String {
    let mut out = String::default();
    if timedelta.num_seconds().is_negative() {
        out.push('-');
    }
    out.push('P');

    let mut seconds = unsafe { (&timedelta as *const _ as *const i64).offset(0).read() }.abs();

    if flags.contains(TimedeltaFlags::YEAR) {
        let years = seconds / YEAR_IN_SECONDS;
        if years > 0 {
            seconds -= years * YEAR_IN_SECONDS;
            out.write_fmt(format_args!("{years}Y")).unwrap();
        }
    }

    if flags.contains(TimedeltaFlags::MONTH) {
        let months = seconds / MONTH_IN_SECONDS;
        if months > 0 {
            seconds -= months * MONTH_IN_SECONDS;
            out.write_fmt(format_args!("{months}M")).unwrap();
        }
    }

    if flags.contains(TimedeltaFlags::WEEK) {
        let weeks = seconds / WEAK_IN_SECONDS;
        if weeks > 0 {
            seconds -= weeks * WEAK_IN_SECONDS;
            out.write_fmt(format_args!("{weeks}W")).unwrap();
        }
    }

    if flags.contains(TimedeltaFlags::DAY) {
        let days = seconds / DAY_IN_SECONDS;
        if days > 0 {
            seconds -= days * DAY_IN_SECONDS;
            out.write_fmt(format_args!("{days}D")).unwrap();
        }
    }

    if flags.contains(TimedeltaFlags::HOUR)
        || flags.contains(TimedeltaFlags::MINUTE)
        || flags.contains(TimedeltaFlags::SECOND)
    {
        out.push('T');
    }

    if flags.contains(TimedeltaFlags::HOUR) {
        let hours = seconds / HOUR_IN_SECONDS;
        if hours > 0 {
            seconds -= hours * HOUR_IN_SECONDS;
            out.write_fmt(format_args!("{hours}H")).unwrap();
        }
    }

    if flags.contains(TimedeltaFlags::MINUTE) {
        let minutes = seconds / MINUTE_IN_SECONDS;
        if minutes > 0 {
            seconds -= minutes * MINUTE_IN_SECONDS;
            out.write_fmt(format_args!("{minutes}M")).unwrap();
        }
    }

    if flags.contains(TimedeltaFlags::SECOND) {
        let nanos = unsafe { (&timedelta as *const _ as *const i32).offset(2).read() }.abs();
        if nanos != 0 && flags.contains(TimedeltaFlags::NANOS) {
            out.write_fmt(format_args!("{seconds}.{}S", nanos)).unwrap();
        } else {
            out.write_fmt(format_args!("{seconds}S")).unwrap();
        }
    }

    out
}

fn timedelta_str_to_preety(str: &str) -> String {
    let mut out = String::default();

    let mut num1 = 0u64;
    let mut num2 = 0u32;
    let mut dec = false;

    let mut time = false;

    for char in str.chars() {
        let s = if num1 > 1 { "s" } else { "" };
        match char {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                let num = char as u32 - b'0' as u32;
                if dec {
                    num2 = (num2 * 10) + num;
                } else {
                    num1 = (num1 * 10) + num as u64;
                }
            }
            '.' => {
                dec = true;
            }
            '-' => {
                out.push('-');
            }
            'P' => {}
            'T' => time = true,
            'Y' => {
                out.push_str(&format!("{num1} Year{s}, "));
                num1 = 0;
            }
            'M' => {
                if !time {
                    out.push_str(&format!("{num1} Month{s}, "));
                } else {
                    out.push_str(&format!("{num1} Minute{s}, "));
                }
                num1 = 0;
            }
            'W' => {
                out.push_str(&format!("{num1} Weak{s}, "));
                num1 = 0;
            }
            'D' => {
                out.push_str(&format!("{num1} Day{s}, "));
                num1 = 0;
            }
            'H' => {
                out.push_str(&format!("{num1} Hour{s}, "));
                num1 = 0;
            }
            'S' => {
                out.push_str(&format!("{num1} Second{s}, "));
                if num2 > 0 {
                    out.push_str(&format!("{num2} Nanoseconds"));
                }
            }
            _ => {}
        }
    }

    out
}

fn timedelta_from_str(str: &str) -> Option<chrono::TimeDelta> {
    let mut seconds = 0i64;
    let mut nanos = 0u32;

    let mut num1 = 0i64;
    let mut num2 = 0u32;
    let mut dec = false;

    let mut sign = 1;

    let mut chars = str.chars();

    loop {
        match chars.next()? {
            '-' => sign = -1,
            'P' => break,
            _ => return None,
        }
    }

    let mut time = false;

    for char in chars {
        match char {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                let num = char as u32 - b'0' as u32;

                if !dec {
                    num1 = (num1 * 10) + num as i64;
                } else {
                    num2 = (num2 * 10) + num;
                }
            }
            'Y' => {
                seconds += num1 * YEAR_IN_SECONDS;
                num1 = 0;
            }
            'M' => {
                if time {
                    seconds += num1 * MINUTE_IN_SECONDS;
                } else {
                    seconds += num1 * MONTH_IN_SECONDS;
                }
                num1 = 0;
            }
            'W' => {
                seconds += num1 * WEAK_IN_SECONDS;
                num1 = 0;
            }
            'D' => {
                seconds += num1 * DAY_IN_SECONDS;
                num1 = 0;
            }
            'T' => {
                num1 = 0;
                time = true;
            }
            'H' => {
                seconds += num1 * HOUR_IN_SECONDS;
                num1 = 0;
            }
            '.' => dec = true,
            'S' => {
                seconds += num1;
                nanos += num2;
            }

            _ => return None,
        }
    }

    chrono::TimeDelta::new(seconds * sign, nanos)
}

#[cfg(test)]
#[test]
fn timedelta() {
    use chrono::NaiveDateTime;
    use chrono::TimeDelta;
    use chrono::Utc;

    let time_delta = TimeDelta::new(1, 32).unwrap();
    assert_eq!(
        timedelta_to_str(time_delta, TimedeltaFlags::all()),
        "PT1.32S".to_owned()
    );
    assert_eq!(
        time_delta,
        timedelta_from_str(&timedelta_to_str(time_delta, TimedeltaFlags::all())).unwrap()
    );

    let since = Utc::now().naive_utc() - NaiveDateTime::UNIX_EPOCH;
    assert_eq!(
        since,
        timedelta_from_str(&timedelta_to_str(since, TimedeltaFlags::default())).unwrap()
    );
}
