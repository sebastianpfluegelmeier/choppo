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

pub fn interpret(input: Main) -> InterpretedClip {
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
    let beats_interpreted = beats
        .iter()
        .fold(HashMap::new(), |interpreted_beats, (name, beat)| {
            let (result, mut interpreted_beats) =
                interpret_beat_expression(beat, beats.clone(), interpreted_beats);
            interpreted_beats.insert(name.clone(), result);
            return interpreted_beats;
        });
    let clips_interpreted = clips
        .iter()
        .fold(HashMap::new(), |interpreted_clips, (name, clip)| {
            let (result, mut interpreted_clips) = interpret_clip_expression(
                clip,
                clips.clone(),
                interpreted_clips,
                beats_interpreted.clone(),
            );
            interpreted_clips.insert(name.clone(), result);
            return interpreted_clips;
        });
    interpret_clip_expression(
        &input.main_expression,
        clips,
        clips_interpreted,
        beats_interpreted,
    )
    .0
}

fn interpret_clip_expression(
    clip: &ClipExpression,
    all_clip_expressions: HashMap<String, ClipExpression>,
    interpreted_clips: HashMap<String, InterpretedClip>,
    interpreted_beats: HashMap<String, InterpretedBeat>,
) -> (InterpretedClip, HashMap<String, InterpretedClip>) {
    match clip {
        ClipExpression::Chain(ClipChainExpression { clip_a, clip_b }) => {
            let (clip_a, interpreted_clips) = interpret_clip_expression(
                clip_a,
                all_clip_expressions.clone(),
                interpreted_clips,
                interpreted_beats.clone(),
            );
            let (mut clip_b, interpreted_clips) = interpret_clip_expression(
                clip_b,
                all_clip_expressions,
                interpreted_clips,
                interpreted_beats,
            );
            for command in &mut clip_b.commands {
                command.0 = &command.0 + &clip_a.length;
            }
            (
                InterpretedClip {
                    commands: vec![clip_a.commands, clip_b.commands]
                        .into_iter()
                        .flatten()
                        .collect(),
                    length: &clip_a.length + &clip_b.length,
                },
                interpreted_clips,
            )
        }
        ClipExpression::Truncated(TruncatedClipExpression { clip, timerange }) => {
            let (mut clip, interpreted_clips) = interpret_clip_expression(
                clip,
                all_clip_expressions,
                interpreted_clips,
                interpreted_beats,
            );
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
                                    num: command.0.num,
                                    denom: command.0.denom,
                                },
                                *subclips,
                            )
                        }
                        ClipCommand::PlayMultiFrom(path, time, subclips) => {
                            command.1 =
                                ClipCommand::PlayMultiFrom(path.clone(), time + &from, *subclips)
                        }
                        ClipCommand::MultiNext => (),
                    };
                    command.0 = Time { num: 0, denom: 1 };
                }
            }
            if let Some(to) = &timerange.to {
                let to = time_expression_to_time(to) - from;
                clip.commands = clip
                    .commands
                    .into_iter()
                    .filter(|c| time_to_frac(&c.0) < time_to_frac(&to))
                    .collect();
                clip.length = to;
            }
            (clip, interpreted_clips)
        }
        ClipExpression::RawVideo(RawVideoExpression { filename }) => (
            InterpretedClip {
                commands: vec![(
                    Time { num: 0, denom: 1 },
                    ClipCommand::PlayClip(filename.clone()),
                )],
                length: Time { num: 1, denom: 1 },
            },
            interpreted_clips,
        ),
        ClipExpression::MultiVideo(MultiVideoExpression { filename, subclips }) => (
            InterpretedClip {
                commands: vec![(
                    Time { num: 0, denom: 1 },
                    ClipCommand::PlayMulti(filename.clone(), *subclips),
                )],
                length: Time { num: 1, denom: 1 },
            },
            interpreted_clips,
        ),
        ClipExpression::Reference(ReferenceClipExpression { name }) => {
            if let Some(clip) = interpreted_clips.get(name) {
                (clip.clone(), interpreted_clips)
            } else {
                let (clip, mut interpreted_clips) = interpret_clip_expression(
                    &all_clip_expressions[name],
                    all_clip_expressions.clone(),
                    interpreted_clips,
                    interpreted_beats,
                );
                interpreted_clips.insert(name.clone(), clip.clone());
                (clip, interpreted_clips)
            }
        }
        ClipExpression::ApplyBeat(ApplyBeatExpression {
            beat_expression,
            clip_expression,
        }) => {
            let (mut clip, interpreted_clips) = interpret_clip_expression(
                clip_expression,
                all_clip_expressions.clone(),
                interpreted_clips,
                interpreted_beats.clone(),
            );
            let (beat, _) =
                interpret_beat_expression(beat_expression, HashMap::new(), interpreted_beats);
            let mut beat_commands = beat
                .beats
                .into_iter()
                .map(|b| (b, ClipCommand::MultiNext))
                .collect();
            clip.commands.append(&mut beat_commands);
            clip.commands.sort_by_key(|b| time_to_frac(&b.0));
            (clip, interpreted_clips)
        }
    }
}

fn interpret_beat_expression(
    beat: &BeatExpression,
    all_beat_expressions: HashMap<String, BeatExpression>,
    interpreted_beats: HashMap<String, InterpretedBeat>,
) -> (InterpretedBeat, HashMap<String, InterpretedBeat>) {
    let (result, interpreted_beats) = match beat {
        BeatExpression::DotBeatExpression(e) => {
            (interpret_dot_beat_expression(e), interpreted_beats)
        }
        BeatExpression::NumberBeatExpression(e) => {
            (interpret_number_beat_expression(e), interpreted_beats)
        }
        BeatExpression::BeatChainExpression(e) => {
            interpret_beat_chain_expression(e, all_beat_expressions, interpreted_beats)
        }
        BeatExpression::ReferenceBeatExpression(e) => {
            interpret_reference_beat_expression(e, all_beat_expressions, interpreted_beats)
        }
    };
    let result = order_beat(result);
    (result, interpreted_beats)
}

fn order_beat(mut input: InterpretedBeat) -> InterpretedBeat {
    input.beats.sort_unstable_by_key(time_to_frac);
    input
}

fn interpret_beat_chain_expression(
    expression: &BeatChainExpression,
    all_beat_expressions: HashMap<String, BeatExpression>,
    interpreted_beats: HashMap<String, InterpretedBeat>,
) -> (InterpretedBeat, HashMap<String, InterpretedBeat>) {
    let (beat_a, interpreted_beats) = interpret_beat_expression(
        &*expression.beat_a,
        all_beat_expressions.clone(),
        interpreted_beats,
    );
    let (beat_b, interpreted_beats) =
        interpret_beat_expression(&*expression.beat_b, all_beat_expressions, interpreted_beats);
    let a_length = time_to_frac(&beat_a.length);
    let sum_length = frac_to_time(&(a_length + time_to_frac(&beat_b.length)));
    let beats_b_updated: Vec<Time> = beat_b
        .beats
        .iter()
        .map(|b| frac_to_time(&(a_length + time_to_frac(b))))
        .collect();
    let all_beats = beats_b_updated.into_iter().chain(beat_a.beats).collect();
    (
        InterpretedBeat {
            beats: all_beats,
            length: sum_length,
        },
        interpreted_beats,
    )
}

fn interpret_reference_beat_expression(
    expression: &ReferenceBeatExpression,
    all_beat_expressions: HashMap<String, BeatExpression>,
    interpreted_beats: HashMap<String, InterpretedBeat>,
) -> (InterpretedBeat, HashMap<String, InterpretedBeat>) {
    let name = &expression.name;
    if interpreted_beats.contains_key(name) {
        (interpreted_beats[name].clone(), interpreted_beats)
    } else {
        interpret_beat_expression(
            &all_beat_expressions[name].clone(),
            all_beat_expressions,
            interpreted_beats,
        )
    }
}

fn interpret_dot_beat_expression(expression: &DotBeatExpression) -> InterpretedBeat {
    let beats = expression
        .beats
        .iter()
        .enumerate()
        .filter_map(|(index, beat_on)| {
            let time = Time {
                num: (index + 1) as isize,
                denom: 16,
            };
            if *beat_on {
                Some(time)
            } else {
                None
            }
        })
        .collect();
    InterpretedBeat {
        beats,
        length: Time {
            num: expression.beats.len() as isize,
            denom: 16,
        },
    }
}

fn interpret_number_beat_expression(expression: &NumberBeatExpression) -> InterpretedBeat {
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
    InterpretedBeat {
        beats,
        length: Time {
            num: length - 1,
            denom: 16,
        },
    }
}

#[derive(Clone, Debug)]
pub struct InterpretedClip {
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
pub struct InterpretedBeat {
    pub beats: Vec<Time>,
    pub length: Time,
}

#[derive(Clone, Debug)]
pub struct Time {
    pub num: isize,
    pub denom: usize,
}

impl Into<f64> for Time {
    fn into(self) -> f64 {
        self.num as f64 / self.denom as f64
    }
}

impl Sub for &Time {
    type Output = Time;

    fn sub(self, rhs: Self) -> Self::Output {
        return frac_to_time(&(time_to_frac(&self) - time_to_frac(&rhs)));
    }
}

impl Sub for Time {
    type Output = Time;

    fn sub(self, rhs: Self) -> Self::Output {
        return frac_to_time(&(time_to_frac(&self) - time_to_frac(&rhs)));
    }
}

impl Add for &Time {
    type Output = Time;

    fn add(self, rhs: Self) -> Self::Output {
        return frac_to_time(&(time_to_frac(&self) + time_to_frac(&rhs)));
    }
}
