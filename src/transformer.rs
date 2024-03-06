use crate::{output_instructions::OutputInstructions, program::Program};

pub fn transform(input: &Program) -> OutputInstructions {
    let mut output_instructions = vec!["".to_string(); 120];
    for (timespan, inp) in &input.instructions {
        let (start, end) = match timespan {
            crate::program::Timespan::All => (0, 129) ,
            crate::program::Timespan::Span(start, end) => (start.beat * 30 + start.frame, end.beat * 30 + end.frame),
        };
        for frame in start..end {
            match inp {
                crate::program::Instruction::PlayVideo(_) => todo!(),
                crate::program::Instruction::PlayTrack(_) => todo!(),
            }
        }

    }

    OutputInstructions {
        instructions: output_instructions
    }
}