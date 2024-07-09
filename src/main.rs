use itertools::Itertools;

#[derive(Debug)]
struct Shuffled {
    text: Vec<usize>,
    entropy: f32,
    child_combinations: Option<Vec<Shuffled>>,
}

/// generate all possible permutations for the given lines. The count and total
/// parameters are used to output a progress indicator.
fn shuffle_text(
    lines: Vec<usize>,
    mut blocksize: usize,
    min_blocksize: usize,
    count: &mut usize,
    total: usize,
) -> Option<Vec<Shuffled>> {
    if blocksize == 1 || blocksize < min_blocksize {
        return None;
    }

    blocksize = blocksize / 2;

    let mut combinations: Vec<Shuffled> = Vec::new();

    let chunks = lines.into_iter().chunks(blocksize);
    let mut chunks_vec: Vec<Vec<usize>> = Vec::new();
    chunks
        .into_iter()
        .for_each(|x| chunks_vec.push(x.collect::<Vec<usize>>()));
    let chunks_count = chunks_vec.len();

    let permutations = chunks_vec.into_iter().permutations(chunks_count);

    for permutation in permutations {
        *count = *count + 1;

        let mut text: Vec<usize> = Vec::new();
        for chunk in &permutation {
            text.append(&mut chunk.clone());
        }
        eprint!(
            "generated permutation............................: {}/{}\r",
            count, total
        );

        combinations.push(Shuffled {
            text: text.clone(),
            entropy: 999999.999999,
            child_combinations: shuffle_text(text.clone(), blocksize, min_blocksize, count, total),
        });
    }

    Some(combinations)
}

/// Calculates an estimate of the total amount of combinations which would be generated
/// with the linecount, starting blocksize and minimum blocksize.
fn calculate_estimated_combination_count(
    line_count: usize,
    mut blocksize: usize,
    min_blocksize: usize,
) -> usize {
    if blocksize == 1 || blocksize < min_blocksize {
        return 0;
    }

    let precise_blocksize: f32 = blocksize as f32 / 2.0;
    blocksize = precise_blocksize.floor() as usize;

    let precise_blockcount: f32 = line_count as f32 / blocksize as f32;
    let blockcount: usize = precise_blockcount.ceil() as usize;

    let counter: usize = (1..=blockcount).product();
    counter
        + (calculate_estimated_combination_count(line_count, blocksize, min_blocksize) * counter)
}

/// Counts the total amount of generated combination in a vector of boxed Shuffled structs.
fn count_combinations(combinations: Vec<Shuffled>) -> usize {
    let mut count = combinations.len();

    for combination in combinations {
        if let Some(children) = combination.child_combinations {
            count = count + count_combinations(children);
        }
    }

    count
}

fn main() {
    let text = "  float c = hash1(n + 317);
  float e = hash1(n + 157);
  float g = hash1(n + 474);
  float f = hash1(n + 268);
  float d = hash1(n + 428);
  float a = hash1(n);
  float h = hash1(n + 585);
  float b = hash1(n + 111);
"
    .to_string();

    // Minimal block size
    let minbs: usize = 4;

    let lines = text.split("\n").collect::<Vec<&str>>();
    let line_count: usize = lines.clone().len();

    let estimated_combination_count: usize =
        calculate_estimated_combination_count(line_count, line_count, minbs);
    eprintln!(
        "estimated combination count......................: {}",
        estimated_combination_count
    );

    let estimated_memory_usage: f32 = estimated_combination_count as f32
        * (std::mem::size_of::<Shuffled>() as f32
            + (line_count as f32 * std::mem::size_of::<usize>() as f32));
    eprintln!(
        "estimated memory usage for all combinations (GiB): {}",
        estimated_memory_usage / (1024.0_f32.powf(3.0))
    );

    let mut count: usize = 0;

    let _ = shuffle_text(
        (0..line_count).collect(),
        line_count,
        minbs,
        &mut count,
        estimated_combination_count,
    )
    .expect("should get some combinations");
    eprintln!("");
    eprintln!(
        "actual combinations to try.......................: {}",
        count
    );
}
