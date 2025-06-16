use std::time::Duration;

pub(crate) fn parse_duration(dur: &str) -> Result<Duration, String>{
    eprintln!("{dur}");
    let dur = dur.trim_matches('"');
    eprintln!("{dur}");
    let mut it = dur.chars().enumerate();
    let mut prev_ind = None;
    let mut parsed_dur = Duration::ZERO;
    let mut dirty = false;
    loop {
        eprintln!("{parsed_dur:?}");
        let Some((ind, ch)) = it.next() else {
            return if parsed_dur == Duration::ZERO {
                Err(format!("parsing '{dur}' resulted in a zero duration"))
            } else if dirty {
                Err(format!("parsing '{dur}' resulted in an unfinished calculation"))
            } else {
                Ok(parsed_dur)
            }
        };
        parse_next(&mut prev_ind, ind, ch, dur, &mut parsed_dur, &mut it,&mut dirty)?;
    }
}

fn parse_next(prev_ind: &mut Option<usize>, ind: usize, ch:char, dur: &str, cumulative_dur: &mut Duration, iterator: &mut impl Iterator<Item = (usize, char)>, dirty: &mut bool) -> Result<(), String> {
    if ch.is_alphabetic() {
        let Some(prev) = dur.get(prev_ind.unwrap_or_default()..ind) else {
            return Err(format!("failed to parse duration from: '{dur}'"));
        };
        let num = parse_num(prev)?;
        let (add_dur, rem) = create_duration(num, ch, iterator)?;
        *cumulative_dur = cumulative_dur.saturating_add(add_dur);
        *dirty = false;
        *prev_ind = Some(ind + 1);
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
    sect.parse().map_err(|e| format!("failed to parse num from '{sect}': {e}"))
}



fn create_duration(num: u64, lead_char: char, iterator: &mut impl Iterator<Item = (usize, char)>) -> Result<(Duration, Option<(usize, char)>), String> {
    let (unit, rem) = parse_unit(lead_char, iterator)?;
    let dur = match unit {
        AcceptedUnits::Hour => {
            Duration::from_secs(num * 60 * 60)
        }
        AcceptedUnits::Minute => {
            Duration::from_secs(num * 60)
        }
        AcceptedUnits::Second => {
            Duration::from_secs(num)       
        }
        AcceptedUnits::Millisecond => {
            Duration::from_millis(num)
        }
    };
    Ok((dur, rem))
}

fn parse_unit(start: char, iterator: &mut impl Iterator<Item = (usize, char)>) -> Result<(AcceptedUnits, Option<(usize, char)>), String> {
    match start {
        'h' => Ok((AcceptedUnits::Hour, None)),
        'm' => {
            let next = iterator.next();
            if let Some((_, 's')) = next {
                Ok((AcceptedUnits::Millisecond, None))
            } else {
                Ok((AcceptedUnits::Minute, next))
            }
        }
        's' => Ok((AcceptedUnits::Second, None)),
        unk => {
            return Err(format!("unknown unit start: '{unk}'"));
        }
    }
}

enum AcceptedUnits {
    Hour,
    Minute,
    Second,
    Millisecond,
}