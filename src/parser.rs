use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while_m_n},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    combinator::{fail, opt, recognize},
    multi::many0,
    sequence::delimited,
    IResult,
};

use self::beats_parser::{parse_beat_expression, BeatExpression};

pub mod beats_parser;

#[derive(Debug)]
pub struct DirectoryDeclaration {
    pub directory: String,
}

pub fn parse_directory_declaration(input: &str) -> IResult<&str, DirectoryDeclaration> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("directory")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, directory) = alt((
        delimited(char('\''), recognize(take_until("'")), char('\'')),
        delimited(char('"'), recognize(take_until("\"")), char('"')),
    ))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag(";")(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        DirectoryDeclaration {
            directory: directory.into(),
        },
    ))
}

#[derive(Debug)]
pub struct ExtensionDeclaration {
    pub extension: String,
}

pub fn parse_extension_declaration(input: &str) -> IResult<&str, ExtensionDeclaration> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("extension")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, directory) = alt((
        delimited(char('\''), recognize(take_until("'")), char('\'')),
        delimited(char('"'), recognize(take_until("\"")), char('"')),
    ))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag(";")(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        ExtensionDeclaration {
            extension: directory.into(),
        },
    ))
}

#[derive(Debug)]
pub enum Declaration {
    ClipDeclaration(ClipDeclaration),
    BeatDeclaration(BeatDeclaration),
}

pub fn parse_declaration(input: &str) -> IResult<&str, Declaration> {
    let (rest_input, input) = take_until(";")(input)?;
    let (input, declaration) = alt((parse_clip_declaration, parse_beat_declaration))(input)?;
    let (input, _) = multispace0(input)?;
    if !input.is_empty() {
        return fail(input);
    }
    let (input, _) = tag(";")(rest_input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, declaration))
}

#[derive(Debug)]
pub struct BeatDeclaration {
    pub expression: BeatExpression,
    pub name: String,
}

pub fn parse_beat_declaration(input: &str) -> IResult<&str, Declaration> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("beat")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, name) = alpha1(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, beat_expression) = parse_beat_expression(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        Declaration::BeatDeclaration(BeatDeclaration {
            expression: beat_expression,
            name: name.to_string(),
        }),
    ))
}

#[derive(Debug)]
pub struct ClipDeclaration {
    pub expression: ClipExpression,
    pub name: String,
}

pub fn parse_clip_declaration(input: &str) -> IResult<&str, Declaration> {
    let (input, _) = multispace0(input)?;
    let (input, _) = alt((tag("clip"), tag("clp")))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, name) = alpha1(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, expression) = parse_clip_expression(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        Declaration::ClipDeclaration(ClipDeclaration {
            expression,
            name: name.into(),
        }),
    ))
}

#[derive(Debug, Clone)]
pub enum ClipExpression {
    Empty,
    Chain(ClipChainExpression),
    Layer(ClipLayerExpression),
    Loop(ClipLoopExpression),
    Restart(RestartExpression),
    Truncated(TruncatedClipExpression),
    RawVideo(RawVideoExpression),
    MultiVideo(MultiVideoExpression),
    Reference(ReferenceClipExpression),
    ApplyBeat(ApplyBeatExpression),
    ParenthesesClipExpression(ParenthesesClipExpression),
}

pub fn parse_clip_expression(input: &str) -> IResult<&str, ClipExpression> {
    alt((
        parse_apply_beat_expression,
        parse_restart_beat_expression,
        parse_clip_chain_expression,
        parse_layer_expression,
        parse_clip_loop_expression,
        parse_truncated_clip_expression,
        parse_parentheses_clip_expression,
        parse_multi_video_expression,
        parse_raw_video_expression,
        parse_reference_clip_expression,
        parse_empty_expression,
    ))(input)
}

pub fn parse_empty_expression(input: &str) -> IResult<&str, ClipExpression> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("-")(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, ClipExpression::Empty))
}

#[derive(Debug, Clone)]
pub struct RestartExpression {
    pub beat_expression: BeatExpression,
    pub clip_expression: Box<ClipExpression>,
}

pub fn parse_restart_beat_expression(input: &str) -> IResult<&str, ClipExpression> {
    let (input, _) = multispace0(input)?;
    let (input, beat_expression) = parse_beat_expression(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag(">>")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, clip_expression) = parse_clip_expression(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        ClipExpression::Restart(RestartExpression {
            beat_expression,
            clip_expression: Box::new(clip_expression),
        }),
    ))
}
#[derive(Debug, Clone)]
pub struct ApplyBeatExpression {
    pub beat_expression: BeatExpression,
    pub clip_expression: Box<ClipExpression>,
}

pub fn parse_apply_beat_expression(input: &str) -> IResult<&str, ClipExpression> {
    let (input, _) = multispace0(input)?;
    let (input, beat_expression) = parse_beat_expression(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("@")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, clip_expression) = parse_clip_expression(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        ClipExpression::ApplyBeat(ApplyBeatExpression {
            beat_expression,
            clip_expression: Box::new(clip_expression),
        }),
    ))
}

#[derive(Debug, Clone)]
pub struct ReferenceClipExpression {
    pub name: String,
}

fn parse_reference_clip_expression(input: &str) -> IResult<&str, ClipExpression> {
    let (input, _) = multispace0(input)?;
    let (input, name) = alphanumeric1(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        ClipExpression::Reference(ReferenceClipExpression { name: name.into() }),
    ))
}

#[derive(Debug, Clone)]
pub struct ParenthesesClipExpression {
    pub clip: Box<ClipExpression>,
}
pub fn parse_parentheses_clip_expression(input: &str) -> IResult<&str, ClipExpression> {
    let (input, _) = multispace0(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, clip) = parse_clip_expression(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(')')(input)?;
    let (input, _) = multispace0(input)?;
    Ok((
        input,
        ClipExpression::ParenthesesClipExpression(ParenthesesClipExpression {
            clip: Box::new(clip),
        }),
    ))
}

#[derive(Debug, Clone)]
pub struct ClipLoopExpression {
    pub clip: Box<ClipExpression>,
    pub repetitions: usize,
}
pub fn parse_clip_loop_expression(input: &str) -> IResult<&str, ClipExpression> {
    let (input, _) = multispace0(input)?;
    let (input, first_part) = take_until("*")(input)?;
    let (rest_first, clip) = parse_clip_expression(first_part)?;
    if !rest_first.is_empty() {
        return fail(rest_first);
    }
    let (input, _) = multispace0(input)?;
    let (input, _) = char('*')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, repetitions) = take_while_m_n(1, 1, |c: char| c.is_ascii_digit())(input)?;
    let repetitions = repetitions
        .parse::<usize>()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Char)))?;
    let (input, _) = multispace0(input)?;
    Ok((
        input,
        ClipExpression::Loop(ClipLoopExpression {
            clip: Box::new(clip),
            repetitions,
        }),
    ))
}

#[derive(Debug, Clone)]
pub struct ClipChainExpression {
    pub clip_a: Box<ClipExpression>,
    pub clip_b: Box<ClipExpression>,
}
pub fn parse_clip_chain_expression(input: &str) -> IResult<&str, ClipExpression> {
    let (input, _) = multispace0(input)?;
    let (input, first_part) = take_until("|")(input)?;
    let (rest_first, clip_a) = parse_clip_expression(first_part)?;
    if !rest_first.is_empty() {
        return fail(rest_first);
    }
    let (input, _) = multispace0(input)?;
    let (input, _) = char('|')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, clip_b) = parse_clip_expression(input)?;
    let (input, _) = multispace0(input)?;
    Ok((
        input,
        ClipExpression::Chain(ClipChainExpression {
            clip_a: Box::new(clip_a),
            clip_b: Box::new(clip_b),
        }),
    ))
}

#[derive(Debug, Clone)]
pub struct ClipLayerExpression {
    pub clip_a: Box<ClipExpression>,
    pub clip_b: Box<ClipExpression>,
}
pub fn parse_layer_expression(input: &str) -> IResult<&str, ClipExpression> {
    let (input, _) = multispace0(input)?;
    let (input, first_part) = take_until(":")(input)?;
    let (rest_first, clip_a) = parse_clip_expression(first_part)?;
    if !rest_first.is_empty() {
        return fail(rest_first);
    }
    let (input, _) = multispace0(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, clip_b) = parse_clip_expression(input)?;
    let (input, _) = multispace0(input)?;
    Ok((
        input,
        ClipExpression::Layer(ClipLayerExpression {
            clip_a: Box::new(clip_a),
            clip_b: Box::new(clip_b),
        }),
    ))
}

#[derive(Debug, Clone)]
pub struct TruncatedClipExpression {
    pub clip: Box<ClipExpression>,
    pub timerange: TimeRangeExpression,
}
pub fn parse_truncated_clip_expression(input: &str) -> IResult<&str, ClipExpression> {
    if !input.contains('[') && !input.contains(']') {
        return fail(input);
    }
    let (input, _) = multispace0(input)?;
    let (input, first_part) = take_until("[")(input)?;
    let (_, clip) = parse_clip_expression(first_part)?;
    let (input, _) = multispace0(input)?;
    let (input, timerange) = parse_time_range_expression(input)?;
    let (input, _) = multispace0(input)?;
    Ok((
        input,
        ClipExpression::Truncated(TruncatedClipExpression {
            clip: Box::new(clip),
            timerange,
        }),
    ))
}

#[derive(Debug, Clone)]
pub struct TimeRangeExpression {
    pub from: Option<TimeExpression>,
    pub to: Option<TimeExpression>,
}
fn parse_time_range_expression(input: &str) -> IResult<&str, TimeRangeExpression> {
    let (input, _) = tag("[")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, from) = opt(parse_time_expression)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, to) = opt(parse_time_expression)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("]")(input)?;
    Ok((input, TimeRangeExpression { from, to }))
}

#[derive(Debug, Clone)]
pub struct TimeExpression {
    pub beat: usize,
    pub sixteenth: Option<usize>,
}

fn parse_time_expression(input: &str) -> IResult<&str, TimeExpression> {
    let (input, _) = multispace0(input)?;
    let (input, beat) = digit1(input)?;
    let beat = beat
        .parse::<usize>()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Char)))?;
    let (input, sixteenth) = opt(parse_time_sixteenth_expression)(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        TimeExpression {
            beat: (beat as isize - 1).max(0) as usize,
            sixteenth: sixteenth.map(|b| (b as isize - 1).max(0) as usize),
        },
    ))
}

fn parse_time_sixteenth_expression(input: &str) -> IResult<&str, usize> {
    let (input, _) = tag(".")(input)?;
    let (input, sixteenth) = digit1(input)?;

    let sixteenth = sixteenth
        .parse::<usize>()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Char)))?;
    Ok((input, sixteenth))
}

#[derive(Debug, Clone)]
pub struct MultiVideoExpression {
    pub filename: String,
    pub subclips: usize,
}
pub fn parse_multi_video_expression(input: &str) -> IResult<&str, ClipExpression> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("multi")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, subclips) = digit1(input)?;

    let subclips = subclips
        .parse::<usize>()
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Char)))?;
    let (input, _) = multispace0(input)?;
    let (input, filename) = alt((
        delimited(char('\''), recognize(take_until("'")), char('\'')),
        delimited(char('"'), recognize(take_until("\"")), char('"')),
    ))(input)?;
    let (input, _) = multispace0(input)?;
    Ok((
        input,
        ClipExpression::MultiVideo(MultiVideoExpression {
            filename: filename.into(),
            subclips,
        }),
    ))
}
#[derive(Debug, Clone)]
pub struct RawVideoExpression {
    pub filename: String,
}
pub fn parse_raw_video_expression(input: &str) -> IResult<&str, ClipExpression> {
    let (input, _) = multispace0(input)?;
    let (input, filename) = alt((
        delimited(char('\''), recognize(take_until("'")), char('\'')),
        delimited(char('"'), recognize(take_until("\"")), char('"')),
    ))(input)?;
    let (input, _) = multispace0(input)?;
    Ok((
        input,
        ClipExpression::RawVideo(RawVideoExpression {
            filename: filename.into(),
        }),
    ))
}

#[derive(Debug)]
pub struct Main {
    pub directory_declaration: DirectoryDeclaration,
    pub extension_declaration: ExtensionDeclaration,
    pub declarations: Vec<Declaration>,
    pub main_expression: ClipExpression,
}

pub fn parse_main(input: &str) -> IResult<&str, Main> {
    let (input, _) = multispace0(input)?;
    let (input, directory_declaration) = parse_directory_declaration(input)?;
    let (input, _) = multispace0(input)?;
    let (input, extension_declaration) = parse_extension_declaration(input)?;
    let (input, _) = multispace0(input)?;
    let (input, declarations) = many0(parse_declaration)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, main_expression) = parse_clip_expression(input)?;
    let (input, _) = multispace0(input)?;
    Ok((
        input,
        Main {
            directory_declaration,
            extension_declaration,
            declarations,
            main_expression,
        },
    ))
}
