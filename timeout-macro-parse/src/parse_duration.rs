use std::time::Duration;

pub fn parse_duration(dur: &str) -> Result<Duration, String> {
    let dur = dur.trim_matches('"');
    let mut it = dur.chars().enumerate();
    let mut prev_ind = None;
    let mut parsed_dur = Duration::ZERO;
    let mut dirty = false;
    loop {
        let Some((ind, ch)) = it.next() else {
            return if parsed_dur == Duration::ZERO {
                Err(format!("parsing '{dur}' resulted in a zero duration"))
            } else if dirty {
                Err(format!(
                    "parsing '{dur}' resulted in an unfinished calculation"
                ))
            } else {
                Ok(parsed_dur)
            };
        };
        parse_next(
            &mut prev_ind,
            ind,
            ch,
            dur,
            &mut parsed_dur,
            &mut it,
            &mut dirty,
        )?;
    }
}

fn parse_next(
    prev_ind: &mut Option<usize>,
    ind: usize,
    ch: char,
    dur: &str,
    cumulative_dur: &mut Duration,
    iterator: &mut impl Iterator<Item = (usize, char)>,
    dirty: &mut bool,
) -> Result<(), String> {
    if ch.is_alphabetic() {
        let pi = prev_ind.unwrap_or_default();
        let Some(prev) = dur.get(pi..ind) else {
            return Err(format!("failed to parse duration from: '{dur}'"));
        };
        let num = parse_num(prev)?;
        let (add_dur, rem, add) = create_duration(num, ch, iterator)?;
        *cumulative_dur = cumulative_dur.saturating_add(add_dur);
        *dirty = false;
        *prev_ind = Some(ind + add);
        if let Some((next_ind, ch)) = rem {
            parse_next(prev_ind, next_ind, ch, dur, cumulative_dur, iterator, dirty)?;
        }
    } else {
        *dirty = true;
    }
    Ok(())
}

fn parse_num(sect: &str) -> Result<u64, String> {
    if sect.is_empty() {
        return Err("failed to parse num, empty section".to_string());
    };
    sect.parse()
        .map_err(|e| format!("failed to parse num from '{sect}': {e}"))
}

fn create_duration(
    num: u64,
    lead_char: char,
    iterator: &mut impl Iterator<Item = (usize, char)>,
) -> Result<(Duration, Option<(usize, char)>, usize), String> {
    let (unit, rem, add) = parse_unit(lead_char, iterator)?;
    let dur = match unit {
        AcceptedUnits::Hour => Duration::from_secs(num * 60 * 60),
        AcceptedUnits::Minute => Duration::from_secs(num * 60),
        AcceptedUnits::Second => Duration::from_secs(num),
        AcceptedUnits::Millisecond => Duration::from_millis(num),
    };
    Ok((dur, rem, add))
}

fn parse_unit(
    start: char,
    iterator: &mut impl Iterator<Item = (usize, char)>,
) -> Result<(AcceptedUnits, Option<(usize, char)>, usize), String> {
    match start {
        'h' => Ok((AcceptedUnits::Hour, None, 1)),
        'm' => {
            let next = iterator.next();
            if let Some((_, 's')) = next {
                Ok((AcceptedUnits::Millisecond, None, 2))
            } else {
                Ok((AcceptedUnits::Minute, next, 1))
            }
        }
        's' => Ok((AcceptedUnits::Second, None, 1)),
        unk => Err(format!("unknown unit start: '{unk}'")),
    }
}

enum AcceptedUnits {
    Hour,
    Minute,
    Second,
    Millisecond,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_reasonable_durations() {
        let hours = parse_duration("3h").unwrap();
        assert_eq!(hours, Duration::from_secs(3 * 3600));
        let minutes = parse_duration("11m").unwrap();
        assert_eq!(minutes, Duration::from_secs(11 * 60));
        let seconds = parse_duration("55s").unwrap();
        assert_eq!(seconds, Duration::from_secs(55));
        let millis = parse_duration("100ms").unwrap();
        assert_eq!(millis, Duration::from_millis(100));
        let combined = "1h2m3s4ms";
        let dur = parse_duration(combined).unwrap();
        let expect = Duration::from_secs(3600)
            + Duration::from_secs(120)
            + Duration::from_secs(3)
            + Duration::from_millis(4);
        assert_eq!(dur, expect);
    }

    #[test]
    fn parse_unreasonable_additive_durations() {
        let dur = "1h1h1h1h";
        let dur = parse_duration(dur).unwrap();
        assert_eq!(Duration::from_secs(3600 * 4), dur);
        let dur = "1m1m1m";
        let dur = parse_duration(dur).unwrap();
        assert_eq!(Duration::from_secs(60 * 3), dur);
        let dur = "1s1s";
        let dur = parse_duration(dur).unwrap();
        assert_eq!(Duration::from_secs(2), dur);
        let dur = "1ms1ms1ms1ms1ms1ms1ms";
        let dur = parse_duration(dur).unwrap();
        assert_eq!(Duration::from_millis(7), dur);
        let dur = "5ms2s1h5ms1m1s";
        let dur = parse_duration(dur).unwrap();
        assert_eq!(
            Duration::from_millis(10) + Duration::from_secs(63) + Duration::from_secs(3600),
            dur
        );
    }
}
