use std::{fmt::Write, str::FromStr};

use chrono::{Offset, TimeZone};
use clap::{Parser, Subcommand};

#[derive(Parser)]
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

        #[arg(short)]
        preety: bool,
    },

    /// alias: -
    #[command(alias = "-")]
    Sub {
        from_date: String,
        date: String,

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
    FormatHelp,
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
        TimeManCommand::Since { date, preety } => {
            let date = parse_date(&format, &time_man.format, &date, "date");
            let now = offset.from_utc_datetime(&chrono::Utc::now().naive_utc());

            let since = now - date;
            let buf = timedelta_to_str(since, ISO8601Flags::all());

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
        } => {
            let from_date = parse_date(&format, &time_man.format, &from_date, "from_date");
            let date = parse_date(&format, &time_man.format, &date, "date");

            let res = from_date - date;
            let buf = timedelta_to_str(res, ISO8601Flags::all());

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
        TimeManCommand::FormatHelp => {
            println!(
                r#"%%    : %
%a    : weekday name (e.g., Sun)
%A    : full weekday name (e.g., Sunday)
%b    : month name (e.g., Jan)
%B    : full month name (e.g., January)
%c    : date and time (e.g., Thu Mar  3 23:05:25 2005)
%C    : first two digits of %Y
%d    : day of month (e.g., 01)
%D    : date; same as %m/%d/%y
%e    : day of month, space padded; same as %_d
%F    : full date; like %+4Y-%m-%d
%g    : last two digits of year of ISO week number (see %G)
%G    : year of ISO week number (see %V); normally useful only with %V
%h    : same as %b
%H    : hour (00..23)
%I    : hour (01..12)
%j    : day of year (001..366)
%k    : hour, space padded ( 0..23); same as %_H
%l    : hour, space padded ( 1..12); same as %_I
%m    : month (01..12)
%M    : minute (00..59)
%n    : a newline
%N    : nanoseconds
%p    : either AM or PM; blank if not known
%P    : like %p, but lower case
%q    : quarter of year (1..4)
%r    : 12-clock (e.g., 11:11:04 PM)
%R    : 24-clock; same as %H:%M
%s    : seconds
%S    : seconds mod 60 (00..60)
%t    : a tab
%T    : time; same as %H:%M:%S
%u    : day of week (1..7)
%U    : week number of year, with Sunday as first day of week (00..53)
%V    : ISO week number, with Monday as first day of week (01..53)
%w    : day of week (0..6); 0 is Sunday
%W    : week number of year, with Monday as first day of week (00..53)
%x    : locale's date representation (e.g., 12/31/99)
%X    : locale's time representation (e.g., 23:13:48)
%y    : last two digits of %Y
%Y    : year
%z    : +hhmm numeric time zone (e.g., -0400)
%:z   : +hh:mm numeric time zone (e.g., -04:00)
%::z  : +hh:mm:ss numeric time zone (e.g., -04:00:00)
%:::z : numeric time zone with : to necessary precision (e.g., -04, +05:30)
%Z    : alphabetic time zone abbreviation (e.g., EDT)"#
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
pub struct ISO8601Flags(u8);

impl core::ops::BitOr<Self> for ISO8601Flags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl Default for ISO8601Flags {
    fn default() -> Self {
        Self::SECOND | Self::NANOS
    }
}

impl ISO8601Flags {
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
}

const YEAR_IN_SECONDS: i64 = 31_536_000;
const MONTH_IN_SECONDS: i64 = YEAR_IN_SECONDS / 12;
const WEAK_IN_SECONDS: i64 = 604800;
const DAY_IN_SECONDS: i64 = 86400;
const HOUR_IN_SECONDS: i64 = 3600;
const MINUTE_IN_SECONDS: i64 = 60;

fn timedelta_to_str(timedelta: chrono::TimeDelta, flags: ISO8601Flags) -> String {
    let mut out = String::default();
    if timedelta.num_seconds().is_negative() {
        out.push('-');
    }
    out.push('P');

    let mut seconds = unsafe { (&timedelta as *const _ as *const i64).offset(0).read() }.abs();

    if flags.contains(ISO8601Flags::YEAR) {
        let years = seconds / YEAR_IN_SECONDS;
        if years > 0 {
            seconds -= years * YEAR_IN_SECONDS;
            out.write_fmt(format_args!("{years}Y")).unwrap();
        }
    }

    if flags.contains(ISO8601Flags::MONTH) {
        let months = seconds / MONTH_IN_SECONDS;
        if months > 0 {
            seconds -= months * MONTH_IN_SECONDS;
            out.write_fmt(format_args!("{months}M")).unwrap();
        }
    }

    if flags.contains(ISO8601Flags::WEEK) {
        let weeks = seconds / WEAK_IN_SECONDS;
        if weeks > 0 {
            seconds -= weeks * WEAK_IN_SECONDS;
            out.write_fmt(format_args!("{weeks}W")).unwrap();
        }
    }

    if flags.contains(ISO8601Flags::DAY) {
        let days = seconds / DAY_IN_SECONDS;
        if days > 0 {
            seconds -= days * DAY_IN_SECONDS;
            out.write_fmt(format_args!("{days}D")).unwrap();
        }
    }

    if flags.contains(ISO8601Flags::HOUR)
        || flags.contains(ISO8601Flags::MINUTE)
        || flags.contains(ISO8601Flags::SECOND)
    {
        out.push('T');
    }

    if flags.contains(ISO8601Flags::HOUR) {
        let hours = seconds / HOUR_IN_SECONDS;
        if hours > 0 {
            seconds -= hours * HOUR_IN_SECONDS;
            out.write_fmt(format_args!("{hours}H")).unwrap();
        }
    }

    if flags.contains(ISO8601Flags::MINUTE) {
        let minutes = seconds / MINUTE_IN_SECONDS;
        if minutes > 0 {
            seconds -= minutes * MINUTE_IN_SECONDS;
            out.write_fmt(format_args!("{minutes}M")).unwrap();
        }
    }

    if flags.contains(ISO8601Flags::SECOND) {
        let nanos = unsafe { (&timedelta as *const _ as *const i32).offset(2).read() }.abs();
        if nanos != 0 && flags.contains(ISO8601Flags::NANOS) {
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
        timedelta_to_str(time_delta, ISO8601Flags::all()),
        "PT1.32S".to_owned()
    );
    assert_eq!(
        time_delta,
        timedelta_from_str(&timedelta_to_str(time_delta, ISO8601Flags::all())).unwrap()
    );

    let since = Utc::now().naive_utc() - NaiveDateTime::UNIX_EPOCH;
    assert_eq!(
        since,
        timedelta_from_str(&timedelta_to_str(since, ISO8601Flags::default())).unwrap()
    );
}
