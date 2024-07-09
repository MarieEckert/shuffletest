use itertools::Itertools;

#[derive(Debug, Clone)]
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

    //blocksize = (blocksize as f32 / 2.0).ceil() as usize;
    blocksize /= 2;

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

/// Makes sure that the given set of permutations is no longer than MAX_PARENT_PERMUTATIONS.
/// if the permutation count exceeds MAX_PARENT_PERMUTATIONS, split the
/// permutations into equal parts and set all permutations after the first
/// one of a part to be a child of the first. e.g:
/// ```
///    1, 2, 3, 4, 5, 6, 7, 8, 9, 10
/// -> [1, 2, 3, 4], [5, 6, 7, 8], [9, 10]
/// -> [1 children: [2, 3, 4]], [5 children: [6, 7, 8]], [9 children: [10]]
/// ```
fn optimize_permutations(permutations: &mut Vec<Shuffled>, count: &mut usize) {
    // The value of this constant could also be used to determine a count of
    // jobs which simultaneously check all permutations.
    const MAX_PARENT_PERMUTATIONS: usize = 2;

    *count += 1;
    eprint!(
        "optimizing permutation set.......................: {}\r",
        count
    );

    if permutations.len() > MAX_PARENT_PERMUTATIONS {
        let permutation_chunks = permutations.into_iter().chunks(MAX_PARENT_PERMUTATIONS);
        let mut new_permutations: Vec<Shuffled> = Vec::new();
        permutation_chunks.into_iter().for_each(|x| {
            let mut x_vec = x.collect::<Vec<&mut Shuffled>>();
            if x_vec.is_empty() {
                return;
            }

            if x_vec.len() == 1 {
                new_permutations.push(x_vec[0].clone());
                return;
            }

            let mut parent = x_vec[0].clone();
            x_vec.remove(0);

            if let None = parent.child_combinations {
                parent.child_combinations = Some(Vec::new());
            }

            if let Some(children) = parent.child_combinations.as_mut() {
                for child in x_vec {
                    children.push(child.clone());
                }
            }

            new_permutations.push(parent);
        });

        permutations.clear();
        permutations.append(&mut new_permutations);
    }

    for permutation in permutations {
        if let Some(child_permutations) = permutation.child_combinations.as_mut() {
            optimize_permutations(child_permutations, count);
        }
    }
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
fn count_combinations(combinations: &Vec<Shuffled>) -> usize {
    if combinations.is_empty() || combinations[0].child_combinations.as_ref().is_none() {
        return 0;
    }

    1 + count_combinations(&combinations[0].child_combinations.clone().unwrap())
}

fn main() {
    let text = "float i_event0 = 0;           // fade in
float i_event5 = 2 * spsec;   // end of fade in
float i_event28 = 22 * spsec;     // sixth scene (chains far away)
float i_event28 = 22 * spsec;     // sixth scene (chains far away)
float i_event28 = 22 * spsec;     // sixth scene (chains far away)
float i_event27p5 = 20 * spsec;   // ship from other side
float i_event30 = 24 * spsec;     // start of first ship scene
float i_event80 = 36 * spsec; // second dreamy bright scene"
        .to_string();

    // Minimal block size
    let minbs: usize = 1;

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

    let mut permutations = shuffle_text(
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

    count = 0;
    optimize_permutations(&mut permutations, &mut count);
    eprintln!("");

    eprintln!(
        "block depth......................................: {}",
        count_combinations(&permutations)
    );

    // When checking which permutation is the best, the current set of permutations
    // should be checked completely. The children of the smallest permutation in the
    // current set should then become the new set to check.
    // Idea #1: A permutation-history should be kept and before actually checking a
    // permutation, it should be made sure that this permutation is not a duplicate
    // of a previously checked permutation.
}
