use std::str::FromStr;

use anyhow::anyhow;
use chumsky::{prelude::*, span::Span, text::Char};
use time::{UtcDateTime, format_description::BorrowedFormatItem};
use time_macros::format_description;

use crate::{
    log::{LogLevel, LogLine},
    prelude::*,
};

const DATETIME_FORMAT: &[BorrowedFormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond] ");

type ParserErr<'src> = extra::Err<Rich<'src, char>>;

pub fn parse_log_line(line: &str) -> Result<LogLine> {
    logcat_header_parser()
        .or(log_entry_parser())
        .then_ignore(end().or(text::newline().then_ignore(end())))
        .parse(line)
        .into_result()
        .map_err(|errors| {
            anyhow!(
                "Failed to parse log line: {}",
                errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        })
}

fn logcat_header_parser<'src>() -> impl Parser<'src, &'src str, LogLine, ParserErr<'src>> {
    just("--------- beginning of ")
        .ignore_then(text::ascii::ident())
        .map(LogLine::header)
}

// 10-01 12:10:45.848  1515  1971 I MiuiNetworkPolicy: removeUidState uid = 10147
fn log_entry_parser<'src>() -> impl Parser<'src, &'src str, LogLine, ParserErr<'src>> {
    let datetime = datetime_parser();
    let pid = u32_parser().padded();
    let tid = u32_parser().padded();
    let level = level_parser().padded();
    let tag_and_message = tag_and_message_parser();

    datetime
        .then(pid)
        .then(tid)
        .then(level)
        .then(tag_and_message)
        .try_map(|((((datetime, pid), tid), level), (tag, message)), _| {
            Ok(LogLine::entry(datetime, pid, tid, level, tag, message))
        })
}

// "10-01 12:10:45.873" -> "MM-DD hh:mm:ss.mmm"
fn datetime_parser<'src>() -> impl Parser<'src, &'src str, UtcDateTime, ParserErr<'src>> {
    let digits = |c| text::digits(10).exactly(c);
    digits(2) // mm
        .then_ignore(just('-'))
        .then(digits(2)) // dd
        .then_ignore(just(' '))
        .then(digits(2)) // hh
        .then_ignore(just(':'))
        .then(digits(2)) // mm
        .then_ignore(just(':'))
        .then(digits(2)) // ss
        .then_ignore(just('.'))
        .then(digits(3)) // mmm
        .then_ignore(just(' '))
        .to_slice()
        .try_map(|input, span| parse_datetime(input).map_err(|e| Rich::custom(span, e)))
}

fn parse_datetime(input: &str) -> Result<UtcDateTime> {
    let input = format!("{}-{input}", UtcDateTime::now().date().year());
    let result = UtcDateTime::parse(&input, DATETIME_FORMAT)
        .map_err(|e| anyhow!("unexpected error while parsing datetime: {e}"))?;

    Ok(result)
}

fn u32_parser<'src>() -> impl Parser<'src, &'src str, u32, ParserErr<'src>> {
    text::int(10).try_map(|s: &str, span| s.parse::<u32>().map_err(|e| Rich::custom(span, e)))
}

// I -> Info
fn level_parser<'src>() -> impl Parser<'src, &'src str, LogLevel, ParserErr<'src>> {
    one_of("VDIWE").map(|c: char| LogLevel::from_str(&c.to_string()).unwrap())
}

fn tag_and_message_parser<'src>() -> impl Parser<'src, &'src str, (String, String), ParserErr<'src>> {
    custom(|input| {
        let before = input.cursor();
        let any_non_newline = any().filter(|c: &char| !c.is_newline()).repeated();
        let tag_and_message = input.parse(any_non_newline.collect::<String>())?;

        let (tag, message) = tag_and_message
            .split_once(": ")
            .ok_or(Rich::custom(input.span_since(&before), "missing ': ' after log tag"))?;

        Ok((tag.trim().to_owned(), message.trim().to_owned()))
    })
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;
    use time_macros::utc_datetime;

    use super::*;

    #[test]
    fn test_parse_logcat_header() {
        let test_case = |header: &str| (format!("--------- beginning of {header}"), LogLine::header(header));
        let headers = vec!["main", "system", "radio"];

        for header in headers {
            let (line, expected) = test_case(header);
            let result = parse_log_line(&line).unwrap();

            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_parse_log_line_sanity() {
        let line = r#"10-01 12:10:45.848  1515  1971 I MiuiNetworkPolicy: removeUidState uid = 10147"#;
        let result = parse_log_line(line).unwrap();

        let expected = LogLine::entry(
            utc_datetime!(2025-10-01 12:10:45.848),
            1515,
            1971,
            LogLevel::Info,
            "MiuiNetworkPolicy".to_owned(),
            "removeUidState uid = 10147".to_owned(),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_log_line_with_padded_tag() {
        let line = r#"10-01 12:10:45.588 14344 14376 E Finsky  : [89] AU2: RequiredVehicleState is missing."#;
        let result = parse_log_line(line).unwrap();

        let expected = LogLine::entry(
            utc_datetime!(2025-10-01 12:10:45.588),
            14344,
            14376,
            LogLevel::Error,
            "Finsky".to_owned(),
            "[89] AU2: RequiredVehicleState is missing.".to_owned(),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_log_line_with_dots_in_tag_and_long_message() {
        let line = r#"10-01 12:10:45.853 11472 11487 I com.xiaomi.xmsf: oneway function results for code 3 on binder at 0xb400007d87527a00 will be dropped but finished with status UNKNOWN_TRANSACTION"#;
        let result = parse_log_line(line).unwrap();

        let expected = LogLine::entry(
            utc_datetime!(2025-10-01 12:10:45.853),
            11472,
            11487,
            LogLevel::Info,
            "com.xiaomi.xmsf".to_owned(),
            "oneway function results for code 3 on binder at 0xb400007d87527a00 will be dropped but finished with status UNKNOWN_TRANSACTION".to_owned(),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_log_line_with_colons_in_tag_and_complex_message() {
        let line = r#"10-01 12:10:45.813  4375  4375 W binder:4375_3: type=1400 audit(0.0:1427): avc: denied { read } for name="u:object_r:system_adbd_prop:s0" dev="tmpfs" ino=1260 scontext=u:r:gmscore_app:s0:c512,c768 tcontext=u:object_r:system_adbd_prop:s0 tclass=file permissive=0 app=com.google.android.gms"#;
        let result = parse_log_line(line).unwrap();

        let expected = LogLine::entry(
            utc_datetime!(2025-10-01 12:10:45.813),
            4375,
            4375,
            LogLevel::Warning,
            "binder:4375_3".to_owned(),
            r#"type=1400 audit(0.0:1427): avc: denied { read } for name="u:object_r:system_adbd_prop:s0" dev="tmpfs" ino=1260 scontext=u:r:gmscore_app:s0:c512,c768 tcontext=u:object_r:system_adbd_prop:s0 tclass=file permissive=0 app=com.google.android.gms"#.to_owned(),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_log_line_with_all_log_levels() {
        let test_case = |log_level: LogLevel| {
            (
                format!("10-01 12:10:45.100  1000  1000 {log_level} TestTag: verbose message"),
                LogLine::entry(
                    utc_datetime!(2025-10-01 12:10:45.100),
                    1000,
                    1000,
                    log_level,
                    "TestTag".to_owned(),
                    "verbose message".to_owned(),
                ),
            )
        };

        for log_level in LogLevel::iter() {
            let (line, expected) = test_case(log_level);
            let result = parse_log_line(&line).unwrap();

            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_parse_log_line_with_invalid_log_lines() {
        let invalid_lines = vec![
            "invalid log line",
            "--------- beginning of ",                           // missing header
            "-------- beginning of main",                        // invalid header
            "10-01 12:10:45.100",                                // incomplete
            "10-01 12:10:45.100 1000",                           // missing fields
            "10-01 12:10:45.100 1000 1000 X TestTag: message",   // invalid level
            "10-01 12:10:45.100 1000 1000 I NoColon other text", // missing colon in message
        ];

        for line in invalid_lines {
            let result = parse_log_line(line);

            assert!(result.is_err(), "Expected error for: {line}");
        }
    }
}
