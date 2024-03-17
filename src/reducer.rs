use std::{
    collections::HashMap,
    ops::{Add, Sub},
};

use crate::{
    parser::{
        ApplyBeatExpression, BeatChainExpression, BeatExpression, ClipChainExpression,
        ClipExpression, DotBeatExpression, Main, MultiVideoExpression, NumberBeatExpression,
        RawVideoExpression, ReferenceBeatExpression, ReferenceClipExpression, TimeExpression,
        TruncatedClipExpression,
    },
    util::{frac_to_time, time_expression_to_time, time_to_frac},
};

pub fn reduce(input: Main) -> ReducedClip {
    let beats: HashMap<String, BeatExpression> = input
        .declarations
        .iter()
        .filter_map(|declaration| match declaration {
            crate::parser::Declaration::BeatDeclaration(bd) => Some(bd),
            _ => None,
        })
        .map(|declaration| (declaration.name.clone(), declaration.expression.clone()))
        .collect();

    let clips: HashMap<String, ClipExpression> = input
        .declarations
        .iter()
        .filter_map(|declaration| match declaration {
            crate::parser::Declaration::ClipDeclaration(cd) => Some(cd),
            _ => None,
        })
        .map(|declaration| (declaration.name.clone(), declaration.expression.clone()))
        .collect();
    let reduced_beats = beats
        .iter()
        .fold(HashMap::new(), |reduced_beats, (name, beat)| {
            let (result, mut reduced_beats) = reduce_beat_expression(beat, &beats, &reduced_beats);
            reduced_beats.insert(name.clone(), result);
            reduced_beats
        });
    let reduced_clips = clips
        .iter()
        .fold(HashMap::new(), |reduced_clips, (name, clip)| {
            let (result, mut reduced_clips) =
                reduce_clip_expression(clip, &clips, &reduced_clips, &reduced_beats);
            reduced_clips.insert(name.clone(), result);
            reduced_clips
        });
    reduce_clip_expression(
        &input.main_expression,
        &clips,
        &reduced_clips,
        &reduced_beats,
    )
    .0
}

fn reduce_clip_expression(
    clip: &ClipExpression,
    all_clip_expressions: &HashMap<String, ClipExpression>,
    reduced_clips: &HashMap<String, ReducedClip>,
    reduced_beats: &HashMap<String, ReducedBeat>,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    match clip {
        ClipExpression::Chain(ClipChainExpression { clip_a, clip_b }) => reduce_chain_expression(
            clip_a,
            &all_clip_expressions,
            &reduced_clips,
            &reduced_beats,
            clip_b,
        ),
        ClipExpression::Truncated(TruncatedClipExpression { clip, timerange }) => {
            reduce_truncate_expression(
                clip,
                &all_clip_expressions,
                &reduced_clips,
                &reduced_beats,
                timerange,
            )
        }
        ClipExpression::RawVideo(RawVideoExpression { filename }) => {
            reduce_raw_video_expression(filename, &reduced_clips)
        }
        ClipExpression::MultiVideo(MultiVideoExpression { filename, subclips }) => {
            reduce_multi_video_expression(filename, subclips, &reduced_clips)
        }
        ClipExpression::Reference(ReferenceClipExpression { name }) => {
            reduce_reference_expression(&reduced_clips, name, &all_clip_expressions, &reduced_beats)
        }
        ClipExpression::ApplyBeat(ApplyBeatExpression {
            beat_expression,
            clip_expression,
        }) => reduce_apply_beat_expression(
            &(*clip_expression),
            all_clip_expressions,
            reduced_clips,
            reduced_beats,
            beat_expression,
        ),
    }
}

fn reduce_raw_video_expression(
    filename: &String,
    reduced_clips: &HashMap<String, ReducedClip>,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    (
        ReducedClip {
            commands: vec![(
                Time { num: 0, denom: 1 },
                ClipCommand::PlayClip(filename.clone()),
            )],
            length: Time { num: 1, denom: 1 },
        },
        reduced_clips.clone(),
    )
}

fn reduce_multi_video_expression(
    filename: &String,
    subclips: &usize,
    reduced_clips: &HashMap<String, ReducedClip>,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    (
        ReducedClip {
            commands: vec![(
                Time { num: 0, denom: 1 },
                ClipCommand::PlayMulti(filename.clone(), *subclips),
            )],
            length: Time { num: 1, denom: 1 },
        },
        reduced_clips.clone(),
    )
}

fn reduce_reference_expression(
    reduced_clips: &HashMap<String, ReducedClip>,
    name: &String,
    all_clip_expressions: &HashMap<String, ClipExpression>,
    reduced_beats: &HashMap<String, ReducedBeat>,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    if let Some(clip) = reduced_clips.get(name) {
        (clip.clone(), reduced_clips.clone())
    } else {
        let (clip, mut reduced_clips) = reduce_clip_expression(
            &all_clip_expressions[name],
            all_clip_expressions,
            reduced_clips,
            reduced_beats,
        );
        reduced_clips.insert(name.clone(), clip.clone());
        (clip, reduced_clips)
    }
}

fn reduce_apply_beat_expression(
    clip_expression: &Box<ClipExpression>,
    all_clip_expressions: &HashMap<String, ClipExpression>,
    reduced_clips: &HashMap<String, ReducedClip>,
    reduced_beats: &HashMap<String, ReducedBeat>,
    beat_expression: &BeatExpression,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    let (mut clip, reduced_clips) = reduce_clip_expression(
        clip_expression,
        all_clip_expressions,
        reduced_clips,
        reduced_beats,
    );
    let (beat, _) = reduce_beat_expression(beat_expression, &HashMap::new(), reduced_beats);
    let mut beat_commands = beat
        .beats
        .into_iter()
        .map(|b| (b, ClipCommand::MultiNext))
        .collect();
    clip.commands.append(&mut beat_commands);
    clip.commands.sort_by_key(|b| time_to_frac(&b.0));
    (clip, reduced_clips)
}

fn reduce_truncate_expression(
    clip: &Box<ClipExpression>,
    all_clip_expressions: &HashMap<String, ClipExpression>,
    reduced_clips: &HashMap<String, ReducedClip>,
    reduced_beats: &HashMap<String, ReducedBeat>,
    timerange: &crate::parser::TimeRangeExpression,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    let (mut clip, reduced_clips) =
        reduce_clip_expression(clip, all_clip_expressions, reduced_clips, reduced_beats);
    let from = time_expression_to_time(&timerange.from.clone().unwrap_or(TimeExpression {
        beat: 0,
        sixteenth: None,
    }));
    for command in &mut clip.commands {
        command.0 = &command.0 - &from;
        if command.0.num < 0 {
            match &command.1 {
                ClipCommand::PlayClip(path) => {
                    command.1 = ClipCommand::PlayClipFrom(
                        path.clone(),
                        Time {
                            num: -command.0.num,
                            denom: command.0.denom,
                        },
                    )
                }
                ClipCommand::PlayClipFrom(path, time) => {
                    command.1 = ClipCommand::PlayClipFrom(path.clone(), time + &from)
                }
                ClipCommand::PlayMulti(path, subclips) => {
                    command.1 = ClipCommand::PlayMultiFrom(
                        path.clone(),
                        Time {
                            num: -command.0.num,
                            denom: command.0.denom,
                        },
                        *subclips,
                    )
                }
                ClipCommand::PlayMultiFrom(path, time, subclips) => {
                    command.1 = ClipCommand::PlayMultiFrom(path.clone(), time + &from, *subclips)
                }
                ClipCommand::MultiNext => (),
            };
            command.0 = Time { num: 0, denom: 1 };
        }
    }
    if let Some(to) = &timerange.to {
        let to = time_expression_to_time(to) - from;
        clip.commands
            .retain(|c| time_to_frac(&c.0) < time_to_frac(&to));
        clip.length = to;
    }
    (clip, reduced_clips)
}

fn reduce_chain_expression(
    clip_a: &Box<ClipExpression>,
    all_clip_expressions: &HashMap<String, ClipExpression>,
    reduced_clips: &HashMap<String, ReducedClip>,
    reduced_beats: &HashMap<String, ReducedBeat>,
    clip_b: &Box<ClipExpression>,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    let (clip_a, reduced_clips) =
        reduce_clip_expression(clip_a, all_clip_expressions, reduced_clips, reduced_beats);
    let (mut clip_b, reduced_clips) =
        reduce_clip_expression(clip_b, all_clip_expressions, &reduced_clips, reduced_beats);
    for command in &mut clip_b.commands {
        command.0 = &command.0 + &clip_a.length;
    }
    (
        ReducedClip {
            commands: vec![clip_a.commands, clip_b.commands]
                .into_iter()
                .flatten()
                .collect(),
            length: &clip_a.length + &clip_b.length,
        },
        reduced_clips,
    )
}

fn reduce_beat_expression(
    beat: &BeatExpression,
    all_beat_expressions: &HashMap<String, BeatExpression>,
    reduced_beats: &HashMap<String, ReducedBeat>,
) -> (ReducedBeat, HashMap<String, ReducedBeat>) {
    let (result, reduced_beats) = match beat {
        BeatExpression::DotBeatExpression(e) => {
            (reduce_dot_beat_expression(e), reduced_beats.clone())
        }
        BeatExpression::NumberBeatExpression(e) => {
            (reduce_number_beat_expression(e), reduced_beats.clone())
        }
        BeatExpression::BeatChainExpression(e) => {
            reduce_beat_chain_expression(e, all_beat_expressions, reduced_beats)
        }
        BeatExpression::ReferenceBeatExpression(e) => {
            reduce_reference_beat_expression(e, all_beat_expressions, reduced_beats)
        }
    };
    let result = order_beat(result);
    (result, reduced_beats)
}

fn order_beat(mut input: ReducedBeat) -> ReducedBeat {
    input.beats.sort_unstable_by_key(time_to_frac);
    input
}

fn reduce_beat_chain_expression(
    expression: &BeatChainExpression,
    all_beat_expressions: &HashMap<String, BeatExpression>,
    reduced_beats: &HashMap<String, ReducedBeat>,
) -> (ReducedBeat, HashMap<String, ReducedBeat>) {
    let (beat_a, reduced_beats) =
        reduce_beat_expression(&expression.beat_a, all_beat_expressions, reduced_beats);
    let (beat_b, reduced_beats) =
        reduce_beat_expression(&expression.beat_b, all_beat_expressions, &reduced_beats);
    let a_length = time_to_frac(&beat_a.length);
    let sum_length = frac_to_time(&(a_length + time_to_frac(&beat_b.length)));
    let beats_b_updated: Vec<Time> = beat_b
        .beats
        .iter()
        .map(|b| frac_to_time(&(a_length + time_to_frac(b))))
        .collect();
    let all_beats = beats_b_updated.into_iter().chain(beat_a.beats).collect();
    (
        ReducedBeat {
            beats: all_beats,
            length: sum_length,
        },
        reduced_beats,
    )
}

fn reduce_reference_beat_expression(
    expression: &ReferenceBeatExpression,
    all_beat_expressions: &HashMap<String, BeatExpression>,
    reduced_beats: &HashMap<String, ReducedBeat>,
) -> (ReducedBeat, HashMap<String, ReducedBeat>) {
    let name = &expression.name;
    if reduced_beats.contains_key(name) {
        (reduced_beats[name].clone(), reduced_beats.clone())
    } else {
        reduce_beat_expression(
            &all_beat_expressions[name].clone(),
            all_beat_expressions,
            reduced_beats,
        )
    }
}

fn reduce_dot_beat_expression(expression: &DotBeatExpression) -> ReducedBeat {
    let beats = expression
        .beats
        .iter()
        .enumerate()
        .filter_map(|(index, beat_on)| {
            let time = Time {
                num: index as isize,
                denom: 16,
            };
            if *beat_on {
                Some(time)
            } else {
                None
            }
        })
        .collect();
    ReducedBeat {
        beats,
        length: Time {
            num: expression.beats.len() as isize,
            denom: 16,
        },
    }
}

fn reduce_number_beat_expression(expression: &NumberBeatExpression) -> ReducedBeat {
    let (length, beats) =
        expression
            .beats
            .iter()
            .fold((1, Vec::new()), |(current_position, mut beats), beat| {
                let new_beat = Time {
                    num: current_position,
                    denom: 16,
                };
                beats.push(new_beat);
                (current_position + *beat as isize, beats)
            });
    ReducedBeat {
        beats,
        length: Time {
            num: length - 1,
            denom: 16,
        },
    }
}

#[derive(Clone, Debug)]
pub struct ReducedClip {
    pub commands: Vec<(Time, ClipCommand)>,
    pub length: Time,
}

#[derive(Clone, Debug)]
pub enum ClipCommand {
    PlayClip(String),
    PlayClipFrom(String, Time),
    PlayMulti(String, usize),
    PlayMultiFrom(String, Time, usize),
    MultiNext,
}

#[derive(Clone, Debug)]
pub struct ReducedBeat {
    pub beats: Vec<Time>,
    pub length: Time,
}

#[derive(Clone, Debug)]
pub struct Time {
    pub num: isize,
    pub denom: usize,
}

impl From<Time> for f64 {
    fn from(val: Time) -> Self {
        val.num as f64 / val.denom as f64
    }
}

impl Sub for &Time {
    type Output = Time;

    fn sub(self, rhs: Self) -> Self::Output {
        frac_to_time(&(time_to_frac(self) - time_to_frac(rhs)))
    }
}

impl Sub for Time {
    type Output = Time;

    fn sub(self, rhs: Self) -> Self::Output {
        frac_to_time(&(time_to_frac(&self) - time_to_frac(&rhs)))
    }
}

impl Add for &Time {
    type Output = Time;

    fn add(self, rhs: Self) -> Self::Output {
        frac_to_time(&(time_to_frac(self) + time_to_frac(rhs)))
    }
}