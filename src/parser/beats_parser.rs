
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while_m_n},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    combinator::{fail, opt, recognize},
    multi::{many0, many1},
    sequence::delimited,
    IResult,
};

#[derive(Debug, Clone)]
pub enum BeatExpression {
    DotBeatExpression(DotBeatExpression),
    NumberBeatExpression(NumberBeatExpression),
    BeatChainExpression(BeatChainExpression),
    ReferenceBeatExpression(ReferenceBeatExpression),
}

pub fn parse_beat_expression(input: &str) -> IResult<&str, BeatExpression> {

    let (input, beat_expression) = alt((
        parse_beat_chain_expression,
        parse_dot_beat_expression,
        parse_number_beat_expression,
        parse_reference_beat_expression,
    ))(input)?;

    Ok((input, beat_expression))
}

pub fn parse_dot_beat_expression(input: &str) -> IResult<&str, BeatExpression> {
    let (input, _) = multispace0(input)?;
    let (input, beats) = parse_dot_beats(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        BeatExpression::DotBeatExpression(DotBeatExpression { beats }),
    ))
}

pub fn parse_dot_beats(input: &str) -> IResult<&str, Vec<bool>> {
    let (input, _) = multispace0(input)?;
    let (input, beats) = many1(parse_dot_beat)(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, beats))
}

pub fn parse_dot_beat(input: &str) -> IResult<&str, bool> {
    let (input, beat) = alt((tag("."), tag("-")))(input)?;

    Ok((input, beat == "."))
}

pub fn parse_number_beat_expression(input: &str) -> IResult<&str, BeatExpression> {
    let (input, _) = multispace0(input)?;
    let (input, beats) = parse_number_beats(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        BeatExpression::NumberBeatExpression(NumberBeatExpression { beats }),
    ))
}

pub fn parse_number_beats(input: &str) -> IResult<&str, Vec<usize>> {
    let (input, _) = multispace0(input)?;
    let (input, beats) = many1(parse_number_beat)(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, beats))
}

pub fn parse_number_beat(input: &str) -> IResult<&str, usize> {
    let (input, _) = multispace0(input)?;
    let (input, beat) = take_while_m_n(1, 1, |c: char| c.is_ascii_digit())(input)?;
    let beat = beat
        .parse::<usize>()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Char)))?;

    Ok((input, beat))
}

#[derive(Debug, Clone)]
pub struct DotBeatExpression {
    pub beats: Vec<bool>,
}

#[derive(Debug, Clone)]
pub struct NumberBeatExpression {
    pub beats: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct BeatChainExpression {
    pub beat_a: Box<BeatExpression>,
    pub beat_b: Box<BeatExpression>,
}

#[derive(Debug, Clone)]
pub struct ReferenceBeatExpression {
    pub name: String,
}

pub fn parse_reference_beat_expression(input: &str) -> IResult<&str, BeatExpression> {
    let (input, _) = multispace0(input)?;
    let (input, name) = alphanumeric1(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        BeatExpression::ReferenceBeatExpression(ReferenceBeatExpression { name: name.into() }),
    ))
}

pub fn parse_beat_chain_expression(input: &str) -> IResult<&str, BeatExpression> {
    let (input, _) = multispace0(input)?;
    let (input, first_part) = take_until("|")(input)?;
    let (rest_first, beat_a) = parse_beat_expression(first_part)?;
    if !rest_first.is_empty() {
        return fail(rest_first);
    }
    let (input, _) = multispace0(input)?;
    let (input, _) = char('|')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, beat_b) = parse_beat_expression(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        BeatExpression::BeatChainExpression(BeatChainExpression {
            beat_a: Box::new(beat_a),
            beat_b: Box::new(beat_b),
        }),
    ))
}