use std::collections::HashMap;

use crate::parser::{BeatChainExpression, BeatExpression, ClipExpression, DotBeatExpression, Main, NumberBeatExpression, ReferenceBeatExpression};

pub fn interpret(input: Main) -> Frames {
    let directory = input.directory_declaration.directory;
    let extension = input.extension_declaration.extension;
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
    let beats_interpreted = beats.iter().fold(HashMap::new(), |interpreted_beats, (name, beat) | {
        let (result, mut interpreted_beats) = interpret_beat_expression(beat, beats.clone(), interpreted_beats);
        interpreted_beats.insert(name.clone(), result);
        return interpreted_beats;
    });
    println!("{:?}", beats_interpreted);


    Frames { frames: Vec::new() }
}

fn interpret_beat_expression(beat: &BeatExpression,  all_beat_expressions: HashMap<String, BeatExpression>,interpreted_beats: HashMap<String, InterpretedBeat>) -> (InterpretedBeat, HashMap<String, InterpretedBeat>) {
    let (result, interpreted_beats) = match beat {
        BeatExpression::DotBeatExpression(e) => (interpret_dot_beat_expression(e), interpreted_beats),
        BeatExpression::NumberBeatExpression(e) => (interpret_number_beat_expression(e), interpreted_beats),
        BeatExpression::BeatChainExpression(e) => interpret_beat_chain_expression(e, all_beat_expressions, interpreted_beats),
        BeatExpression::ReferenceBeatExpression(e) => interpret_reference_beat_expression(e, all_beat_expressions, interpreted_beats),
    };
    let result = order_beat(result);
    (result, interpreted_beats)
}

fn order_beat(mut input: InterpretedBeat) -> InterpretedBeat {
    input.beats.sort_by_key(|t| ((t.nominator as f64 / t.denominator as f64) * 100000.0) as usize);
    input
}

fn interpret_beat_chain_expression(expression: &BeatChainExpression, all_beat_expressions: HashMap<String, BeatExpression>, interpreted_beats: HashMap<String, InterpretedBeat>) -> (InterpretedBeat, HashMap<String,InterpretedBeat>) {
    let (beat_a, interpreted_beats) = interpret_beat_expression(&*expression.beat_a, all_beat_expressions.clone(), interpreted_beats);
    let (beat_b, interpreted_beats) = interpret_beat_expression(&*expression.beat_b, all_beat_expressions, interpreted_beats);
    let a_length = fraction::Fraction::new(beat_a.length.nominator as u64, beat_a.length.denominator as u64);
    let b_length = fraction::Fraction::new(beat_b.length.nominator as u64, beat_b.length.denominator as u64);
    let sum_length = a_length + b_length;
    let sum_length = Time { nominator: *sum_length.fract().numer().unwrap_or(&0) as usize, denominator: *sum_length.fract().denom().unwrap_or(&0) as usize};
    let beats_b_updated: Vec<Time> = beat_b.beats.iter().map(|b| {
        let b_time = fraction::Fraction::new(b.nominator as u64, b.denominator as u64);
        let sum = a_length + b_time;
        Time {nominator: *sum.fract().numer().unwrap_or(&0) as usize, denominator: *sum.fract().denom().unwrap_or(&0) as usize}
    }).collect();
    let all_beats = beats_b_updated.into_iter().chain(beat_a.beats).collect();
    (InterpretedBeat { beats: all_beats, length: sum_length}, interpreted_beats)


}

fn interpret_reference_beat_expression(expression: &ReferenceBeatExpression, all_beat_expressions: HashMap<String, BeatExpression>,  interpreted_beats: HashMap<String, InterpretedBeat>) -> (InterpretedBeat, HashMap<String,InterpretedBeat>) {
    let name = &expression.name;
    if interpreted_beats.contains_key(name) {
        (interpreted_beats[name].clone(), interpreted_beats)
    } else {
        interpret_beat_expression(&all_beat_expressions[name].clone(), all_beat_expressions, interpreted_beats)
    }
}

fn interpret_dot_beat_expression(expression: &DotBeatExpression) -> InterpretedBeat {
    let beats = expression.beats.iter().enumerate().filter_map(|(index, beat_on)| {
        let time = Time {nominator: index + 1, denominator: 16};
        if *beat_on {
            Some(time)
        } else {
            None
        }
    }).collect();
    InterpretedBeat {beats, length: Time {nominator: expression.beats.len(), denominator: 16}}
}

fn interpret_number_beat_expression(expression: &NumberBeatExpression) -> InterpretedBeat {
    let (length, beats) = expression.beats.iter().fold((1, Vec::new()), |(current_position, mut beats), beat| {
        let new_beat = Time {nominator: current_position, denominator: 16};
        beats.push(new_beat);
        (current_position + beat, beats)
    });
    InterpretedBeat {beats, length: Time { nominator: length - 1, denominator: 16}}
}

pub struct Frames {
    frames: Vec<(usize, String)>,
}

#[derive(Clone, Debug)]
pub struct InterpretedBeat {
    beats: Vec<Time>,
    length: Time
}

#[derive(Clone, Debug)]
pub struct Time{
    nominator: usize,
    denominator: usize
}
