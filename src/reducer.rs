use std::{
    collections::HashMap,
    ops::{Add, Sub},
};

use crate::{
    parser::{
        beats_parser::{BeatChainExpression, BeatExpression, DotBeatExpression, NumberBeatExpression, ReferenceBeatExpression}, ApplyBeatExpression, ClipChainExpression, ClipExpression, ClipLoopExpression, ClipLayerExpression, Main, MultiVideoExpression, ParenthesesClipExpression, RawVideoExpression, ReferenceClipExpression, RestartExpression, TimeExpression, TruncatedClipExpression
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
            let (result, mut reduced_clips) = reduce_clip_expression(
                &input.directory_declaration.directory,
                &input.extension_declaration.extension,
                clip,
                &clips,
                &reduced_clips,
                &reduced_beats,
            );
            reduced_clips.insert(name.clone(), result);
            reduced_clips
        });
    reduce_clip_expression(
        &input.directory_declaration.directory,
        &input.extension_declaration.extension,
        &input.main_expression,
        &clips,
        &reduced_clips,
        &reduced_beats,
    )
    .0
}

fn reduce_clip_expression(
    path: &str,
    extension: &str,
    clip: &ClipExpression,
    all_clip_expressions: &HashMap<String, ClipExpression>,
    reduced_clips: &HashMap<String, ReducedClip>,
    reduced_beats: &HashMap<String, ReducedBeat>,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    match clip {
        ClipExpression::ParenthesesClipExpression(ParenthesesClipExpression {
            clip,
        }) => reduce_clip_expression(
            path,
            extension,
            &(*clip),
            all_clip_expressions,
            reduced_clips,
            reduced_beats,
        ),
        ClipExpression::Restart(RestartExpression {
            clip_expression,
            beat_expression,
        }) => reduce_restart_expression(
            path,
            extension,
            &(*clip_expression),
            all_clip_expressions,
            reduced_clips,
            reduced_beats,
            beat_expression,
        ),
        ClipExpression::Layer(ClipLayerExpression { clip_a, clip_b }) => reduce_layer_expression(
            path,
            extension,
            clip_a,
            &all_clip_expressions,
            &reduced_clips,
            &reduced_beats,
            clip_b,
        ),
        ClipExpression::Chain(ClipChainExpression { clip_a, clip_b }) => reduce_chain_expression(
            path,
            extension,
            clip_a,
            &all_clip_expressions,
            &reduced_clips,
            &reduced_beats,
            clip_b,
        ),
        ClipExpression::Truncated(TruncatedClipExpression { clip, timerange }) => {
            reduce_truncate_expression(
                path,
                extension,
                clip,
                &all_clip_expressions,
                &reduced_clips,
                &reduced_beats,
                timerange,
            )
        }
        ClipExpression::RawVideo(RawVideoExpression { filename }) => {
            reduce_raw_video_expression(path, extension, filename, &reduced_clips)
        }
        ClipExpression::MultiVideo(MultiVideoExpression { filename, subclips }) => {
            reduce_multi_video_expression(path, extension, filename, subclips, &reduced_clips)
        }
        ClipExpression::Reference(ReferenceClipExpression { name }) => reduce_reference_expression(
            path,
            extension,
            &reduced_clips,
            name,
            &all_clip_expressions,
            &reduced_beats,
        ),
        ClipExpression::ApplyBeat(ApplyBeatExpression {
            beat_expression,
            clip_expression,
        }) => reduce_apply_beat_expression(
            path,
            extension,
            &(*clip_expression),
            all_clip_expressions,
            reduced_clips,
            reduced_beats,
            beat_expression,
        ),
        ClipExpression::Loop(ClipLoopExpression { clip, repetitions }) => {
            reduce_clip_loop_expression(
                path,
                extension,
                *repetitions,
                &(*clip),
                all_clip_expressions,
                reduced_clips,
                reduced_beats,
            )
        }
    }
}

fn reduce_raw_video_expression(
    path: &str,
    extension: &str,
    filename: &str,
    reduced_clips: &HashMap<String, ReducedClip>,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    (
        ReducedClip {
            commands: vec![(
                Time { num: 0, denom: 1 },
                ClipCommand::PlayClip(format!("{}{}{}", path, filename, extension), 0),
            )],
            length: Time { num: 1, denom: 1 },
        },
        reduced_clips.clone(),
    )
}

fn reduce_multi_video_expression(
    path: &str,
    extension: &str,
    filename: &str,
    subclips: &usize,
    reduced_clips: &HashMap<String, ReducedClip>,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    (
        ReducedClip {
            commands: vec![(
                Time { num: 0, denom: 1 },
                ClipCommand::PlayMulti(
                    format!("{}{}", path, filename),
                    *subclips,
                    extension.to_string(),
                ),
            )],
            length: Time { num: 1, denom: 1 },
        },
        reduced_clips.clone(),
    )
}

fn reduce_reference_expression(
    path: &str,
    extension: &str,
    reduced_clips: &HashMap<String, ReducedClip>,
    name: &String,
    all_clip_expressions: &HashMap<String, ClipExpression>,
    reduced_beats: &HashMap<String, ReducedBeat>,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    if let Some(clip) = reduced_clips.get(name) {
        (clip.clone(), reduced_clips.clone())
    } else {
        let (clip, mut reduced_clips) = reduce_clip_expression(
            path,
            extension,
            &all_clip_expressions[name],
            all_clip_expressions,
            reduced_clips,
            reduced_beats,
        );
        reduced_clips.insert(name.clone(), clip.clone());
        (clip, reduced_clips)
    }
}

fn reduce_restart_expression(
    path: &str,
    extension: &str,
    clip_expression: &Box<ClipExpression>,
    all_clip_expressions: &HashMap<String, ClipExpression>,
    reduced_clips: &HashMap<String, ReducedClip>,
    reduced_beats: &HashMap<String, ReducedBeat>,
    beat_expression: &BeatExpression,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    let (mut clip, reduced_clips) = reduce_clip_expression(
        path,
        extension,
        clip_expression,
        all_clip_expressions,
        reduced_clips,
        reduced_beats,
    );
    let (mut beat, _) = reduce_beat_expression(beat_expression, &HashMap::new(), reduced_beats);
    beat.beats.push(beat.length);
    let left_shift = beat.beats[..beat.beats.len() - 1].iter();
    let right_shift = beat.beats[1..].iter();
    let lengths: Vec<Time> = left_shift.zip(right_shift).map(|(b1, b2)| {b2 - b1}).collect();
    let beat_commands = lengths
        .into_iter()
        .map(|beat_time| {
            let mut clip = clip.clone();
            slice_clip(&mut clip, &None, &Some(beat_time));
            clip
        })
        .reduce(|l, r| chain(l, r));
    clip.commands.sort_by_key(|b| time_to_frac(&b.0));
    (beat_commands.unwrap_or(clip), reduced_clips)
}

fn reduce_apply_beat_expression(
    path: &str,
    extension: &str,
    clip_expression: &Box<ClipExpression>,
    all_clip_expressions: &HashMap<String, ClipExpression>,
    reduced_clips: &HashMap<String, ReducedClip>,
    reduced_beats: &HashMap<String, ReducedBeat>,
    beat_expression: &BeatExpression,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    let (mut clip, reduced_clips) = reduce_clip_expression(
        path,
        extension,
        clip_expression,
        all_clip_expressions,
        reduced_clips,
        reduced_beats,
    );
    let (beat, _) = reduce_beat_expression(beat_expression, &HashMap::new(), reduced_beats);
    let mut beat_commands = beat
        .beats
        .into_iter()
        .map(|b| (b, ClipCommand::MultiNext(0)))
        .collect();
    clip.commands.append(&mut beat_commands);
    clip.commands.sort_by_key(|b| time_to_frac(&b.0));
    (clip, reduced_clips)
}

fn reduce_truncate_expression(
    path: &str,
    extension: &str,
    clip: &Box<ClipExpression>,
    all_clip_expressions: &HashMap<String, ClipExpression>,
    reduced_clips: &HashMap<String, ReducedClip>,
    reduced_beats: &HashMap<String, ReducedBeat>,
    timerange: &crate::parser::TimeRangeExpression,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    let (mut clip, reduced_clips) = reduce_clip_expression(
        path,
        extension,
        clip,
        all_clip_expressions,
        reduced_clips,
        reduced_beats,
    );
    let from = timerange.from.as_ref().map(|t| time_expression_to_time(&t));
    let to = timerange.to.as_ref().map(|t| time_expression_to_time(&t));
    slice_clip(&mut clip, &from, &to);
    (clip, reduced_clips)
}

fn slice_clip(clip: &mut ReducedClip, from: &Option<Time>, to: &Option<Time>) {
    let zero = Time::zero();
    let from = from.as_ref().unwrap_or(&zero);
    for command in &mut clip.commands {
        command.0 = &command.0 - &from;
        if command.0.num < 0 {
            match &command.1 {
                ClipCommand::PlayClip(path, layer) => {
                    command.1 = ClipCommand::PlayClipFrom(
                        path.clone(),
                        *layer,
                        Time {
                            num: -command.0.num,
                            denom: command.0.denom,
                        },
                    )
                }
                ClipCommand::PlayClipFrom(path, layer, time) => {
                    command.1 = ClipCommand::PlayClipFrom(path.clone(), *layer, time + &from)
                }
                ClipCommand::PlayMulti(path, subclips, extension) => {
                    command.1 = ClipCommand::PlayMultiFrom(
                        path.clone(),
                        Time {
                            num: -command.0.num,
                            denom: command.0.denom,
                        },
                        *subclips,
                        extension.clone(),
                    )
                }
                ClipCommand::PlayMultiFrom(path, time, subclips, extension) => {
                    command.1 = ClipCommand::PlayMultiFrom(
                        path.clone(),
                        time + &from,
                        *subclips,
                        extension.clone(),
                    )
                }
                ClipCommand::MultiNext(_) => (),
            };
            command.0 = Time { num: 0, denom: 1 };
        }
    }
    if let Some(to) = to {
        clip.commands
            .retain(|c| time_to_frac(&c.0) < time_to_frac(&to));
        clip.length = to.clone();
    }
}

fn reduce_clip_loop_expression(
    path: &str,
    extension: &str,
    repetitions: usize,
    clip: &Box<ClipExpression>,
    all_clip_expressions: &HashMap<String, ClipExpression>,
    reduced_clips: &HashMap<String, ReducedClip>,
    reduced_beats: &HashMap<String, ReducedBeat>,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    let (clip, reduced_clips) = reduce_clip_expression(
        path,
        extension,
        clip,
        all_clip_expressions,
        reduced_clips,
        reduced_beats,
    );
    let mut clips = Vec::new();

    for i in 0..repetitions {
        let mut new_clip = clip.clone();
        for c in &mut new_clip.commands {
            c.0 = &c.0 + &clip.length.mul(i as isize);
        }
        clips.push(new_clip.commands);
    }
    (
        ReducedClip {
            commands: clips.into_iter().flatten().collect(),
            length: clip.length.mul(repetitions as isize),
        },
        reduced_clips,
    )
}

fn reduce_layer_expression(
    path: &str,
    extension: &str,
    clip_a: &Box<ClipExpression>,
    all_clip_expressions: &HashMap<String, ClipExpression>,
    reduced_clips: &HashMap<String, ReducedClip>,
    reduced_beats: &HashMap<String, ReducedBeat>,
    clip_b: &Box<ClipExpression>,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    let (clip_a, reduced_clips) = reduce_clip_expression(
        path,
        extension,
        clip_a,
        all_clip_expressions,
        reduced_clips,
        reduced_beats,
    );
    let max_layer = clip_a.commands.iter().map(|(_, c) | c.layer()).max().unwrap_or(0);
    let (clip_b, reduced_clips) = reduce_clip_expression(
        path,
        extension,
        clip_b,
        all_clip_expressions,
        &reduced_clips,
        reduced_beats,
    );
    let clip = layer(clip_b, clip_a, max_layer);
    (clip, reduced_clips)
}
fn layer(mut clip_a: ReducedClip, mut clip_b: ReducedClip, max_layer: usize) -> ReducedClip {
    for command in &mut clip_b.commands {
        command.1 = command.1.add_layer(max_layer + 1);
    }
    let clip = ReducedClip {
        commands: vec![clip_a.commands, clip_b.commands]
            .into_iter()
            .flatten()
            .collect(),
        length: if f64::from(&clip_a.length) < f64::from(&clip_b.length) { clip_a.length.clone() } else {clip_b.length.clone() },
    };
    clip
}

fn reduce_chain_expression(
    path: &str,
    extension: &str,
    clip_a: &Box<ClipExpression>,
    all_clip_expressions: &HashMap<String, ClipExpression>,
    reduced_clips: &HashMap<String, ReducedClip>,
    reduced_beats: &HashMap<String, ReducedBeat>,
    clip_b: &Box<ClipExpression>,
) -> (ReducedClip, HashMap<String, ReducedClip>) {
    let (clip_a, reduced_clips) = reduce_clip_expression(
        path,
        extension,
        clip_a,
        all_clip_expressions,
        reduced_clips,
        reduced_beats,
    );
    let (clip_b, reduced_clips) = reduce_clip_expression(
        path,
        extension,
        clip_b,
        all_clip_expressions,
        &reduced_clips,
        reduced_beats,
    );
    let clip = chain(clip_b, clip_a);
    (clip, reduced_clips)
}

fn chain(clip_a: ReducedClip, mut clip_b: ReducedClip) -> ReducedClip {
    for command in &mut clip_b.commands {
        command.0 = &command.0 + &clip_a.length;
    }
    let clip = ReducedClip {
        commands: vec![clip_a.commands, clip_b.commands]
            .into_iter()
            .flatten()
            .collect(),
        length: &clip_a.length + &clip_b.length,
    };
    clip
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

impl ReducedClip {
    pub fn print(&self) {
        println!("length {}:{}={}", self.length.num ,self.length.denom, self.length.num as f64/ self.length.denom as f64);
        for (time, command) in &self.commands {
            print!("time {}:{}={}", time.num ,time.denom, time.num as f64/ time.denom as f64);
            println!("command {:?}", command);
        }
    }
}

#[derive(Clone, Debug)]
pub enum ClipCommand {
    PlayClip(String, usize),
    PlayClipFrom(String, usize, Time),
    PlayMulti(String, usize, String),
    PlayMultiFrom(String, Time, usize, String),
    MultiNext(usize),
}

impl ClipCommand {
    pub fn layer(&self) -> usize {
        match self {
            ClipCommand::PlayClip(_, layer) => *layer,
            ClipCommand::PlayClipFrom(_, layer, _) => *layer,
            ClipCommand::PlayMulti(_, _, _) => 0,
            ClipCommand::PlayMultiFrom(_, _, _, _) => 0,
            ClipCommand::MultiNext(layer) => *layer,
        }
    }

    pub fn add_layer(&self, layer: usize) -> ClipCommand{
        match self {
            ClipCommand::PlayClip(file, l) => ClipCommand::PlayClip(file.clone(), layer + l),
            ClipCommand::PlayClipFrom(file, l, time) => ClipCommand::PlayClipFrom(file.clone(), layer + l, time.clone()),
            ClipCommand::PlayMulti(_, _, _) => self.clone(),
            ClipCommand::PlayMultiFrom(_, _, _, _) => self.clone(),
            ClipCommand::MultiNext(l) => ClipCommand::MultiNext(layer + l),
        }
    }
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

impl Time {
    pub fn mul(&self, other: isize) -> Self {
        Time {
            num: self.num * other,
            denom: self.denom,
        }
    }

    pub fn zero() -> Self {
        Time { num: 0, denom: 1 }
    }
}

impl From<Time> for f64 {
    fn from(val: Time) -> Self {
        val.num as f64 / val.denom as f64
    }
}

impl From<&Time> for f64 {
    fn from(val: &Time) -> Self {
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
