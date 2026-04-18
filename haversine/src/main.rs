use std::io::Read;

fn main() -> Result<(), &'static str> {
    let mut input_buffer = Vec::new();
    std::io::stdin()
        .lock()
        .read_to_end(&mut input_buffer)
        .unwrap();
    let input = parse_input(&input_buffer).ok_or("could not parse input")?;

    let mut haversine_count = 0usize;
    let sum: f64 = input
        .pairs
        .iter()
        .map(|pair| {
            haversine_count += 1;
            haversine_of_degrees(pair.lat0, pair.long0, pair.lat1, pair.long1, 6371.0)
        })
        .sum();
    println!("computed {haversine_count} haversines");
    println!(
        "got sum: {sum}, expected sum: {}, difference: {}",
        input.sum,
        sum - input.sum
    );
    let average = sum / f64::from(u32::try_from(input.pairs.len()).unwrap());
    println!(
        "got average: {average}, expected: {}, difference: {}",
        input.average,
        average - input.average
    );

    Ok(())
}

// baseline naïve version given in the course
fn haversine_of_degrees(lat0: f64, long0: f64, lat1: f64, long1: f64, radius: f64) -> f64 {
    let d_y = (lat1 - lat0).to_radians();
    let d_x = (long1 - long0).to_radians();
    let y0 = lat0.to_radians();
    let y1 = lat1.to_radians();

    let root_term = (d_y / 2.0).sin().powi(2) + y0.cos() * y1.cos() * (d_x / 2.0).sin().powi(2);
    2.0 * root_term.sqrt().asin() * radius
}

/*
fn within_epsilon(left: f64, right: f64) -> bool {
    let (greater, lesser) = if right > left {
        (right, left)
    } else {
        (left, right)
    };

    lesser > greater - greater * f64::EPSILON
}
*/

#[derive(Debug, PartialEq)]
struct Pair {
    lat0: f64,
    long0: f64,
    lat1: f64,
    long1: f64,
}

#[derive(Debug, PartialEq)]
struct Input {
    pairs: Vec<Pair>,
    sum: f64,
    average: f64,
}

#[test]
fn parse_input_test() {
    assert_eq!(parse_input(b"{}"), None);
    assert_eq!(
        parse_input(br#"{ "pairs": [ {"lat0":1.0,"long0":2.0,"lat1":3.0,"long1":4.0} ] }"#),
        None
    );
    assert_eq!(parse_input(br#"{ "pairs": [ {"lat0":1.0,"long0":2.0,"lat1":3.0,"long1":4.0} ], "sum": 10.00, "average":  11.0 }"#).unwrap(),
        Input {
            pairs: Vec::from([ Pair { lat0: 1.0, long0: 2.0, lat1: 3.0, long1: 4.0 } ]),
            sum: 10.0,
            average: 11.0,
        }
    );
}

fn parse_input(input: &[u8]) -> Option<Input> {
    let mut pairs = None;
    let mut sum = None;
    let mut average = None;

    parse_object(
        input,
        &mut [
            (b"pairs", &mut |input| {
                let (got_pairs, rest) = parse_pairs(input)?;
                pairs = Some(got_pairs);
                Some(rest)
            }),
            (b"sum", &mut |input| {
                let (number, rest) = parse_f64(input)?;
                sum = Some(number);
                Some(rest)
            }),
            (b"average", &mut |input| {
                let (number, rest) = parse_f64(input)?;
                average = Some(number);
                Some(rest)
            }),
        ],
    )?;

    if let Some(pairs) = pairs
        && let Some(sum) = sum
        && let Some(average) = average
    {
        Some(Input {
            pairs,
            sum,
            average,
        })
    } else {
        None
    }
}

#[test]
fn parse_pairs_test() {
    assert_eq!(
        parse_pairs(br#"[{"lat0":1.0,"long0":2.0,"lat1":3.0,"long1":4.0},{"lat0":5.0,"long0":4.0,"lat1":3.0,"long1":2.0}]"#)
            .unwrap()
            .0,
        [Pair {
            lat0: 1.0,
            long0: 2.0,
            lat1: 3.0,
            long1: 4.0,
        }, Pair {
            lat0: 5.0,
            long0: 4.0,
            lat1: 3.0,
            long1: 2.0,
        },
        ]
    );
    assert_eq!(parse_pairs(br#"[]"#).unwrap().0, []);
}

fn parse_pairs(mut input: &[u8]) -> Option<(Vec<Pair>, &[u8])> {
    let mut result = Vec::new();
    input = parse_byte(input, b'[')?;

    if let Some((pair, rest)) = parse_pair(input) {
        result.push(pair);
        input = rest;
    } else {
        return parse_byte(input, b']').map(|rest| (result, rest));
    };

    loop {
        if let Some(rest) = parse_byte(input, b',')
            && let Some((pair, rest)) = parse_pair(rest)
        {
            result.push(pair);
            input = rest;
        } else {
            return parse_byte(input, b']').map(|rest| (result, rest));
        }
    }
}

#[test]
fn parse_pair_test() {
    assert_eq!(parse_pair(br#"{"lat0":1.0}"#), None);
    assert_eq!(
        parse_pair(br#"{"lat0":1.0,"long0":2.0,"lat1":3.0,"long1":4.0}"#)
            .unwrap()
            .0,
        Pair {
            lat0: 1.0,
            long0: 2.0,
            lat1: 3.0,
            long1: 4.0,
        }
    );
}

fn parse_pair(input: &[u8]) -> Option<(Pair, &[u8])> {
    let mut lat0 = None;
    let mut long0 = None;
    let mut lat1 = None;
    let mut long1 = None;

    let rest = parse_object(
        input,
        &mut [
            (b"lat0", &mut |input| {
                let (number, rest) = parse_f64(input)?;
                lat0 = Some(number);
                Some(rest)
            }),
            (b"long0", &mut |input| {
                let (number, rest) = parse_f64(input)?;
                long0 = Some(number);
                Some(rest)
            }),
            (b"lat1", &mut |input| {
                let (number, rest) = parse_f64(input)?;
                lat1 = Some(number);
                Some(rest)
            }),
            (b"long1", &mut |input| {
                let (number, rest) = parse_f64(input)?;
                long1 = Some(number);
                Some(rest)
            }),
        ],
    )?;

    if let Some(lat0) = lat0
        && let Some(long0) = long0
        && let Some(lat1) = lat1
        && let Some(long1) = long1
    {
        Some((
            Pair {
                lat0,
                long0,
                lat1,
                long1,
            },
            rest,
        ))
    } else {
        None
    }
}

type ParserMap<'input, 'f> = [(
    &'static [u8],
    &'f mut dyn FnMut(&'input [u8]) -> Option<&'input [u8]>,
)];

fn parse_object<'input, 'f>(
    mut input: &'input [u8],
    key_parsers: &mut ParserMap<'input, 'f>,
) -> Option<&'input [u8]> {
    input = parse_byte(input, b'{')?;

    let mut parse_key_value = |input: &'input [u8], key: &[u8]| {
        for (key_name, parser) in key_parsers.iter_mut() {
            if *key_name == key {
                return parser(input);
            }
        }
        None
    };

    if let Some((key, rest)) = parse_key(input) {
        input = parse_key_value(rest, key)?;
    } else {
        return parse_byte(input, b'}');
    }

    loop {
        if let Some(rest) = parse_byte(input, b',')
            && let Some((key, rest)) = parse_key(rest)
        {
            input = parse_key_value(rest, key)?;
        } else {
            return parse_byte(input, b'}');
        }
    }
}

#[test]
fn parse_f64_test() {
    assert_eq!(parse_f64(b"1.0"), Some((1.0, b"".as_slice())));
    assert_eq!(parse_f64(b"-1.0"), Some((-1.0, b"".as_slice())));
    assert_eq!(
        parse_f64(b"  918273.12other"),
        Some((918273.12, b"other".as_slice()))
    );
    assert_eq!(parse_f64(b"ieaorfnt"), None);
}

fn parse_f64(mut input: &[u8]) -> Option<(f64, &[u8])> {
    // TODO support negative
    let mut result = 0.0;
    let mut got_digit = false;
    input = trim(input);

    let negative = if let Some(rest) = parse_byte(input, b'-') {
        input = rest;
        true
    } else {
        false
    };

    while let [c, rest @ ..] = input
        && c.is_ascii_digit()
    {
        result = result * 10.0 + (c - b'0') as f64;
        got_digit = true;
        input = rest;
    }

    if let [b'.', rest @ ..] = input {
        input = rest;
        let mut factor = 0.1;
        while let [c, rest @ ..] = input
            && c.is_ascii_digit()
        {
            result += (c - b'0') as f64 * factor;
            got_digit = true;
            input = rest;
            factor *= 0.1;
        }
    }

    if got_digit {
        if negative {
            result = -result;
        }
        Some((result, input))
    } else {
        None
    }
}

fn parse_key(input: &[u8]) -> Option<(&[u8], &[u8])> {
    let (key, rest) = parse_string(input)?;
    let rest = parse_byte(rest, b':')?;
    Some((key, rest))
}

#[test]
fn test_parse_string() {
    assert_eq!(
        parse_string(b"\"mykey\""),
        Some((b"mykey".as_slice(), b"".as_slice()))
    );
    assert_eq!(
        parse_string(b"\"mykey\": \"someval\""),
        Some((b"mykey".as_slice(), b": \"someval\"".as_slice()))
    );
}

// does not support escape sequences
fn parse_string(input: &[u8]) -> Option<(&[u8], &[u8])> {
    let mut rest = parse_byte(input, b'\"')?;

    let mut split = rest.splitn(2, |c| *c == b'\"');
    let string = split.next().unwrap();
    rest = split.next()?;

    Some((string, rest))
}

#[test]
fn test_parse_byte() {
    let string = b" \nhello";
    assert_eq!(Some(b"ello".as_ref()), parse_byte(string, b'h'));
}

fn parse_byte(mut input: &[u8], byte: u8) -> Option<&[u8]> {
    input = trim(input);
    if let Some(c) = input.first()
        && *c == byte
    {
        Some(&input[1..])
    } else {
        None
    }
}

fn trim(input: &[u8]) -> &[u8] {
    match input {
        [b' ' | b'\n' | b'\r' | b'\t', rest @ ..] => trim(rest),
        _ => input,
    }
}
