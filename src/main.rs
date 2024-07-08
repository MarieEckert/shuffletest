use itertools::Itertools;
use lazy_static::lazy_static;
use std::sync::RwLock;

#[derive(Debug)]
struct Shuffled {
    text: Vec<String>,
    entropy: f32,
    child_combinations: Option<Vec<Box<Shuffled>>>
}

fn shuffle_text(lines: Vec<String>, mut blocksize: usize, min_blocksize: usize, count: &mut usize, total: usize) -> Option<Vec<Box<Shuffled>>> {
    if blocksize == 1 || blocksize < min_blocksize {
        return None;
    }

    blocksize = blocksize / 2;

    let mut combinations: Vec<Box<Shuffled>> = Vec::new();

    let chunks = lines.into_iter().chunks(blocksize);
    let mut string_chunks: Vec<Vec<String>> = Vec::new();
    for chunk in &chunks {
        string_chunks.push(chunk.collect::<Vec<String>>());
    }

    let string_chunk_count: usize = string_chunks.clone().len();
    let permutations = string_chunks.into_iter().permutations(string_chunk_count);
    
    for permutation in permutations {
        *count = *count + 1;

        let mut text: Vec<String> = Vec::new();
        for mut chunk in &permutation {
            text.append(&mut chunk.clone());
        }
        eprint!("generated permutation............................: {}/{}\r", count, total);

        combinations.push(Box::new(Shuffled {
            text: text.clone(),
            entropy: 999999.999999,
            child_combinations: shuffle_text(text.clone(), blocksize, min_blocksize, count, total)
        }));
    }

    Some(combinations)
}

/// Calculates an estimate of the total amount of combinations which would be generated
/// with the linecount, starting blocksize and minimum blocksize.
fn calculate_estimated_combination_count(line_count: usize, mut blocksize: usize, min_blocksize: usize) -> usize {
    if blocksize == 1 || blocksize < min_blocksize {
        return 0;
    }
    
    let precise_blocksize: f32 = blocksize as f32 / 2.0;
    blocksize = precise_blocksize.floor() as usize;

    let precise_blockcount: f32 = line_count as f32 / blocksize as f32;
    let blockcount: usize = precise_blockcount.ceil() as usize;

    let mut counter: usize = (1..=blockcount).product();
    counter + (calculate_estimated_combination_count(line_count, blocksize, min_blocksize) * counter)
}

/// Counts the total amount of generated combination in a vector of boxed Shuffled structs.
fn count_combinations(combinations: Vec<Box<Shuffled>>) -> usize {
    let mut count = combinations.len();

    for combination in combinations {
        if let Some(children) = combination.child_combinations {
            count = count + count_combinations(children);
        }
    }

    count
}

fn main() {
    let text = "# Prost


Prost is a 4k intro framework: it uses handlebar templating and provides example democode

## Usage

If you clone an existing prost project, run `build_tools.sh` to clone and build the tools (prost, clinkshter, cold).".to_string();

    let str_lines = text.split("\n").collect::<Vec<&str>>();
    let mut lines: Vec<String> = Vec::new();
    for line in str_lines {
        lines.push(line.to_string());
    }

    let line_count: usize = lines.clone().len();

    let minbs: usize = 2;

    let estimated_combination_count: usize = calculate_estimated_combination_count(line_count, line_count, minbs);
    eprintln!("estimated combination count......................: {}", estimated_combination_count);

    let estimated_memory_usage: f32 = estimated_combination_count as f32 * (std::mem::size_of::<Shuffled>() as f32 + text.len() as f32);
    eprintln!("estimated memory usage for all combinations (kiB): {}", estimated_memory_usage / 1024.0);

    let mut count: usize = 0;

    let shuffle_combinations = shuffle_text(lines, line_count, minbs, &mut count, estimated_combination_count).expect("should get some combinations");
    eprintln!("");
    eprintln!("actual combinations to try.......................: {}", count_combinations(shuffle_combinations));
}
