use core::{num::ParseIntError, str::Split};

#[derive(Debug)]
pub enum GpsParseError {
    NoFix,
    ParseIntError(ParseIntError),
    MalformedInput(&'static str),
    NoInput(&'static str),
    InvalidChecksum,
}

impl From<ParseIntError> for GpsParseError {
    fn from(value: ParseIntError) -> Self {
        return GpsParseError::ParseIntError(value);
    }
}

pub fn parse_gps_command(sentence: super::GpsSentence) -> Result<super::GpsCommand, GpsParseError> {
    let string = core::str::from_utf8(&sentence.data[..sentence.len])
        .map_err(|_| GpsParseError::MalformedInput("sentence not convertable to &str"))?;

    let body = validate_checksum(string)?;

    for c in body.chars() {
        match c {
            '\n' => esp_println::print!("\\n"),
            '\r' => esp_println::print!("\\r"),
            '\0' => esp_println::print!("\\0"),
            _ => esp_println::print!("{}", c),
        }
    }
    esp_println::print!("\n");

    let mut iter = body.split(",");
    if let Some(opcode) = iter.next() {
        match &opcode[2..] {
            "GGA" => {
                let opt_utc_time = get_next_field(&mut iter);
                let opt_latitude = get_next_field(&mut iter);
                let opt_latitude_dir = get_next_field(&mut iter);
                let opt_longitude = get_next_field(&mut iter);
                let opt_longitude_dir = get_next_field(&mut iter);

                let fix_indicator = get_next_field(&mut iter)
                    .ok_or(GpsParseError::MalformedInput("GGA position_indicator"))?
                    .as_bytes()[0]
                    - b'0';
                if fix_indicator == 0 {
                    return Err(GpsParseError::NoFix);
                }

                let utc_time =
                    parse_utc_time(opt_utc_time.ok_or(GpsParseError::NoInput("GGA utc_time"))?)?;

                let mut latitude =
                    parse_latitude(opt_latitude.ok_or(GpsParseError::NoInput("GGA latitude"))?)?;
                match opt_latitude_dir {
                    Some("N") => {}
                    Some("S") => latitude = -latitude,
                    _ => {
                        return Err(GpsParseError::MalformedInput(
                            "GGA latitude direction (N/S)",
                        ));
                    }
                }

                let mut longitude =
                    parse_longitude(opt_longitude.ok_or(GpsParseError::NoInput("GGA longitude"))?)?;
                match opt_longitude_dir {
                    Some("E") => {}
                    Some("W") => longitude = -longitude,
                    _ => {
                        return Err(GpsParseError::MalformedInput(
                            "GGA longitude direction (E/W)",
                        ));
                    }
                }

                let satellites_used = get_next_field(&mut iter)
                    .ok_or(GpsParseError::MalformedInput("GGA satellites_used"))?
                    .as_bytes()[0]
                    - b'0';

                let hdop = parse_str_u32(
                    get_next_field(&mut iter).ok_or(GpsParseError::NoInput("GGA hdop"))?,
                    8,
                )? as u16; // loss of precision is acceptable here
                let antenna_altitude = parse_str_u32(
                    get_next_field(&mut iter)
                        .ok_or(GpsParseError::NoInput("GGA antenna_altitude"))?,
                    10,
                )?; // up to 4_194_304 meters
                let antenna_altitude_units = get_next_field(&mut iter)
                    .ok_or(GpsParseError::NoInput("GGA antenna_altitude_units"))?; // altitude units, assumed to be meters 'M'
                if antenna_altitude_units != "M" {
                    return Err(GpsParseError::MalformedInput("GGA antenna_altitude_units"));
                }

                let _ = get_next_field(&mut iter)
                    .ok_or(GpsParseError::NoInput("GGA geodial seperation"))?; // geodial seperation
                let _ = get_next_field(&mut iter)
                    .ok_or(GpsParseError::NoInput("GGA geodial seperation units"))?; // geodial seperation units

                let age_of_diff_correction = get_next_field(&mut iter)
                    .map(|s| s.parse::<u16>())
                    .transpose()
                    .map_err(|_| GpsParseError::MalformedInput("GGA age_of_diff_correction"))?;

                return Ok(super::GpsCommand::GGA(super::GGACommand {
                    utc_time: utc_time,
                    latitude: latitude,
                    longitude: longitude,
                    fix_indicator,
                    satilites_used: satellites_used,
                    horizontal_dilution_of_precision: hdop,
                    antenna_altitude: antenna_altitude,
                    age_of_diff_correction: age_of_diff_correction,
                }));
            }
            "GSA" => {
                let opt_mode = get_next_field(&mut iter);

                let fix_indicator = get_next_field(&mut iter)
                    .ok_or(GpsParseError::MalformedInput("GSA fix indicator"))?
                    .as_bytes()[0]
                    - b'0';

                if fix_indicator == 1 {
                    return Err(GpsParseError::NoFix);
                }

                let mode = opt_mode
                    .ok_or(GpsParseError::MalformedInput("GSA mode"))?
                    .as_bytes()[0]
                    - b'0';

                let mut sats = [0u8; 12];
                for i in 0..12 {
                    let sat = get_next_field(&mut iter).unwrap_or("0").parse::<u8>()?;
                    sats[i] = sat;
                }

                let pdop = parse_str_u32(iter.next().ok_or(GpsParseError::NoInput("GSA pdop"))?, 8)?
                    as u16; // loss of precision is acceptable here

                let hdop = parse_str_u32(iter.next().ok_or(GpsParseError::NoInput("GSA hdop"))?, 8)?
                    as u16; // loss of precision is acceptable here

                let vdop = parse_str_u32(iter.next().ok_or(GpsParseError::NoInput("GSA vdop"))?, 8)?
                    as u16; // loss of precision is acceptable here

                return Ok(super::GpsCommand::GSA(super::GSACommand {
                    mode: mode,
                    fix: fix_indicator,
                    satilites_used: sats,
                    position_dilution_of_precision: pdop,
                    horizontal_dilution_of_precision: hdop,
                    vertical_dilution_of_precision: vdop,
                }));
            }
            "GSV" => {
                let number_of_messages = get_next_field(&mut iter)
                    .ok_or(GpsParseError::MalformedInput("GSV number_of_messages"))?
                    .as_bytes()[0]
                    - b'0';

                let message_number = get_next_field(&mut iter)
                    .ok_or(GpsParseError::MalformedInput("GSV message_number"))?
                    .as_bytes()[0]
                    - b'0';

                let sats_in_view = get_next_field(&mut iter)
                    .ok_or(GpsParseError::MalformedInput("GSV sats_in_view"))?
                    .as_bytes()[0]
                    - b'0';

                let mut sats = [super::Satellite {
                    satellite_id: 0b11111111,
                    elevation: 0,
                    azimuth: 0,
                    signal_to_noise_ratio: None,
                }; 4];

                for i in 0..4 {
                    let sat_id = get_next_field(&mut iter);

                    if sat_id.is_none() {
                        break;
                    }

                    let sat_id = sat_id
                        .ok_or(GpsParseError::MalformedInput("GSV sat_id"))?
                        .as_bytes()[0]
                        - b'0';

                    let elevation = get_next_field(&mut iter)
                        .ok_or(GpsParseError::MalformedInput("GSV elevation"))?
                        .as_bytes()[0]
                        - b'0';

                    let azimuth = get_next_field(&mut iter)
                        .ok_or(GpsParseError::MalformedInput("GSV azimuth"))?
                        .as_bytes()[0]
                        - b'0';

                    let snr = get_next_field(&mut iter).map(|snr| snr.as_bytes()[0] - b'0');

                    sats[i] = super::Satellite {
                        satellite_id: sat_id,
                        elevation,
                        azimuth,
                        signal_to_noise_ratio: snr,
                    }
                }

                return Ok(super::GpsCommand::GSV(super::GSVCommand {
                    number_of_messages: number_of_messages,
                    message_number: message_number,
                    satellites_in_view: sats_in_view,
                    satellites: sats,
                }));
            }
            "RMC" => {
                let utc_time = parse_utc_time(
                    get_next_field(&mut iter).ok_or(GpsParseError::NoInput("GGA utc_time"))?,
                )?;

                let status = get_next_field(&mut iter)
                    .ok_or(GpsParseError::MalformedInput("RMC status"))?
                    .as_bytes()[0]
                    - b'0';

                let mut latitude = parse_latitude(
                    get_next_field(&mut iter).ok_or(GpsParseError::NoInput("RMC latitude"))?,
                )?;
                match get_next_field(&mut iter) {
                    Some("N") => {}
                    Some("S") => latitude = -latitude,
                    _ => {
                        return Err(GpsParseError::MalformedInput(
                            "RMC latitude direction (N/S)",
                        ));
                    }
                }

                let mut longitude = parse_longitude(
                    get_next_field(&mut iter).ok_or(GpsParseError::NoInput("RMC longitude"))?,
                )?;
                match get_next_field(&mut iter) {
                    Some("E") => {}
                    Some("W") => longitude = -longitude,
                    _ => {
                        return Err(GpsParseError::MalformedInput(
                            "RMC longitude direction (E/W)",
                        ));
                    }
                }

                let knots = parse_str_u32(
                    get_next_field(&mut iter).ok_or(GpsParseError::NoInput("RMC knots"))?,
                    8,
                )? as u16;

                let course = parse_str_u32(
                    get_next_field(&mut iter).ok_or(GpsParseError::NoInput("RMC course"))?,
                    8,
                )? as u16;

                let date = get_next_field(&mut iter)
                    .ok_or(GpsParseError::NoInput("RMC date"))?
                    .parse::<u32>()?;

                let magnetic_var = get_next_field(&mut iter)
                    .map(|m| parse_str_u32(m, 8).map(|v| v as i16))
                    .transpose()?;

                if get_next_field(&mut iter) == Some("W") {
                    magnetic_var.map(|m| -m);
                }

                let mode = get_next_field(&mut iter)
                    .ok_or(GpsParseError::NoInput("RMC mode"))?
                    .as_bytes()[0];

                return Ok(super::GpsCommand::RMC(super::RMCCommand {
                    utc_time: utc_time,
                    status: status,
                    latitude: latitude,
                    longitude: longitude,
                    knots: knots,
                    course: course,
                    date: date,
                    magnetic_variation: magnetic_var,
                    mode: mode,
                }));
            }
            "VTG" => {
                let course = parse_str_u32(
                    get_next_field(&mut iter).ok_or(GpsParseError::NoInput("VTG course"))?,
                    8,
                )? as u16;

                let _ = iter.next(); // drop T for true heading

                let _ = iter.next(); // drop magnetic course
                let _ = iter.next(); // drop M for magnetic heading

                let knots = parse_str_u32(
                    get_next_field(&mut iter).ok_or(GpsParseError::NoInput("VTG knots"))?,
                    8,
                )? as u16;

                let _ = iter.next(); // drop L for knots unit

                let mode = get_next_field(&mut iter)
                    .ok_or(GpsParseError::NoInput("VTG mode"))?
                    .as_bytes()[0];

                return Ok(super::GpsCommand::VTG(super::VTGCommand {
                    course: course,
                    knots: knots,
                    mode: mode,
                }));
            }
            _ => {
                return Err(GpsParseError::MalformedInput("unchecked opcode"));
            }
        }
    }

    Err(GpsParseError::MalformedInput("no opcode"))
}

fn get_next_field<'a>(iterator: &mut Split<'a, &str>) -> Option<&'a str> {
    iterator.next().filter(|s| !s.is_empty())
}

fn validate_checksum(sentence: &str) -> Result<&str, GpsParseError> {
    let sentence = sentence
        .strip_prefix("$")
        .ok_or(GpsParseError::MalformedInput(
            "checksum sentence not prefixed with $",
        ))?;

    let (body, checksum) = sentence
        .split_once('*')
        .ok_or(GpsParseError::MalformedInput(
            "checksum sentence does not have a '*' for the checksum",
        ))?;

    if checksum.len() < 2 {
        return Err(GpsParseError::InvalidChecksum);
    }

    let expected = u8::from_str_radix(&checksum[..2], 16)
        .map_err(|_| GpsParseError::MalformedInput("checksum not in hex notation"))?;

    let actual = body.bytes().fold(0u8, |acc, byte| acc ^ byte);

    if actual != expected {
        return Err(GpsParseError::InvalidChecksum);
    }

    Ok(body)
}

fn parse_latitude(s: &str) -> Result<i32, GpsParseError> {
    let bytes = s.as_bytes();

    if bytes.len() != 9 || bytes[4] != b'.' {
        return Err(GpsParseError::MalformedInput(
            "latitude not in proper notation",
        ));
    }

    let degrees = ((bytes[0] - b'0') as i32) * 10 + (bytes[1] - b'0') as i32;

    let minutes = ((bytes[2] - b'0') as i32) * 10 + (bytes[3] - b'0') as i32;

    let fractional_minutes = ((bytes[5] - b'0') as i32) * 1000
        + ((bytes[6] - b'0') as i32) * 100
        + ((bytes[7] - b'0') as i32) * 10
        + (bytes[8] - b'0') as i32;

    // Convert minutes to microdegrees.
    //
    // 07.1256 minutes = 7.1256 / 60 degrees
    let microdegrees = degrees * 1_000_000 + (minutes * 1_000_000 + fractional_minutes * 100) / 60;

    Ok(microdegrees)
}

fn parse_longitude(s: &str) -> Result<i32, GpsParseError> {
    let bytes = s.as_bytes();

    if bytes.len() != 10 || bytes[5] != b'.' {
        return Err(GpsParseError::MalformedInput(
            "longitude not in proper notation",
        ));
    }

    let degrees = ((bytes[0] - b'0') as i32) * 100
        + ((bytes[1] - b'0') as i32) * 10
        + (bytes[2] - b'0') as i32;

    let minutes = ((bytes[3] - b'0') as i32) * 10 + (bytes[4] - b'0') as i32;

    let fractional_minutes = ((bytes[6] - b'0') as i32) * 1000
        + ((bytes[7] - b'0') as i32) * 100
        + ((bytes[8] - b'0') as i32) * 10
        + (bytes[9] - b'0') as i32;

    let microdegrees = degrees * 1_000_000 + (minutes * 1_000_000 + fractional_minutes * 100) / 60;

    Ok(microdegrees)
}

fn parse_utc_time(s: &str) -> Result<u32, GpsParseError> {
    let bytes = s.as_bytes();

    // hhmmss.sss = 10 characters
    if bytes.len() != 10 || bytes[6] != b'.' {
        return Err(GpsParseError::MalformedInput(
            "utc_time not in proper notation",
        ));
    }

    let hours = ((bytes[0] - b'0') as u32) * 10 + (bytes[1] - b'0') as u32;

    let minutes = ((bytes[2] - b'0') as u32) * 10 + (bytes[3] - b'0') as u32;

    let seconds = ((bytes[4] - b'0') as u32) * 10 + (bytes[5] - b'0') as u32;

    let milliseconds = ((bytes[7] - b'0') as u32) * 100
        + ((bytes[8] - b'0') as u32) * 10
        + (bytes[9] - b'0') as u32;

    if hours >= 24 || minutes >= 60 || seconds >= 60 {
        return Err(GpsParseError::MalformedInput(
            "utc_time (hrs/min/sec) out of range",
        ));
    }

    Ok(((hours * 60 + minutes) * 60 + seconds) * 1000 + milliseconds)
}

fn parse_str_u32(s: &str, frac_bits: u32) -> Result<u32, GpsParseError> {
    let (whole, frac) = s.split_once('.').ok_or(GpsParseError::MalformedInput(
        "parse_str_u32 str does not have a '.'",
    ))?;
    let result_whole_part = whole.parse::<u32>()?;
    let result_frac_part = frac.parse::<u32>()?;

    let result = (result_whole_part << frac_bits) + result_frac_part.clamp(0, (1 << frac_bits) - 1);

    Ok(result)
}
