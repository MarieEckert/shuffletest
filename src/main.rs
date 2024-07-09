//! Test implementation of permutating and optimising permutation
//! for prost-shuffle.

use deepsize::DeepSizeOf;
use itertools::Itertools;

#[derive(Debug, Clone, DeepSizeOf)]
struct Permutation {
    /// Relative line number of the block of text the permutation is applied to
    lines: Vec<usize>,

    /// The entropy of the final generated executable with this permutation
    entropy: f32,

    /// The permutations derived from this permutation and "adopted" permutations
    child_permutations: Option<Vec<Permutation>>,
}

/// generate all possible permutations for the given lines. The count and total
/// parameters are used to output a progress indicator.
fn shuffle_lines(
    lines: Vec<usize>,
    mut blocksize: usize,
    min_blocksize: usize,
    count: &mut usize,
    total: usize,
) -> Option<Vec<Permutation>> {
    if blocksize == 1 || blocksize < min_blocksize {
        return None;
    }

    //blocksize = (blocksize as f32 / 2.0).ceil() as usize;
    blocksize /= 2;

    let mut line_permutations: Vec<Permutation> = Vec::new();

    let chunks = lines.into_iter().chunks(blocksize);
    let mut chunks_vec: Vec<Vec<usize>> = Vec::new();
    chunks
        .into_iter()
        .for_each(|x| chunks_vec.push(x.collect::<Vec<usize>>()));
    let chunks_count = chunks_vec.len();

    let permutations = chunks_vec.into_iter().permutations(chunks_count);

    for permutation in permutations {
        *count = *count + 1;

        let mut lines: Vec<usize> = Vec::new();
        for chunk in &permutation {
            lines.append(&mut chunk.clone());
        }
        eprint!(
            "generated permutation............................: {}/{}\r",
            count, total
        );

        line_permutations.push(Permutation {
            lines: lines.clone(),
            entropy: 999999.999999,
            child_permutations: shuffle_lines(
                lines.clone(),
                blocksize,
                min_blocksize,
                count,
                total,
            ),
        });
    }

    Some(line_permutations)
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
fn optimize_permutations(permutations: &mut Vec<Permutation>, count: &mut usize) {
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
        let mut new_permutations: Vec<Permutation> = Vec::new();
        permutation_chunks.into_iter().for_each(|iter_chunk| {
            let mut chunk = iter_chunk.collect::<Vec<&mut Permutation>>();
            if chunk.is_empty() {
                return;
            }

            if chunk.len() == 1 {
                new_permutations.push(chunk[0].clone());
                return;
            }

            let mut parent = chunk[0].clone();
            chunk.remove(0);

            if let None = parent.child_permutations {
                parent.child_permutations = Some(Vec::new());
            }

            if let Some(children) = parent.child_permutations.as_mut() {
                for child in chunk {
                    children.push(child.clone());
                }
            }

            new_permutations.push(parent);
        });

        permutations.clear();
        permutations.append(&mut new_permutations);
    }

    for permutation in permutations {
        if let Some(child_permutations) = permutation.child_permutations.as_mut() {
            optimize_permutations(child_permutations, count);
        }
    }
}

/// Calculates an estimate of the total amount of permutations which would be generated
/// with the linecount, starting blocksize and minimum blocksize.
fn calculate_estimated_permutation_count(
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
        + (calculate_estimated_permutation_count(line_count, blocksize, min_blocksize) * counter)
}

/// Counts the total amount of generated permutation in a vector of boxed Permutation structs.
fn count_permutations(permutations: &Vec<Permutation>) -> usize {
    if permutations.is_empty() || permutations[0].child_permutations.as_ref().is_none() {
        return 0;
    }

    1 + count_permutations(&permutations[0].child_permutations.clone().unwrap())
}

fn main() {
    let lines = "float i_event0 = 0;           // fade in
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

    let lines = lines.split("\n").collect::<Vec<&str>>();
    let line_count: usize = lines.clone().len();

    let estimated_permutation_count: usize =
        calculate_estimated_permutation_count(line_count, line_count, minbs);
    eprintln!(
        "estimated permutation count......................: {}",
        estimated_permutation_count
    );

    // The estimate is doubled since the optimisation step likely doubles the memory
    // usage in favour of more efficient searching
    let estimated_memory_usage: f32 = estimated_permutation_count as f32
        * (std::mem::size_of::<Permutation>() as f32
            + (line_count as f32 * std::mem::size_of::<usize>() as f32))
        * 2.0;
    eprintln!(
        "estimated memory usage for all permutations (GiB): {}",
        estimated_memory_usage / (1024.0_f32.powf(3.0))
    );

    let mut count: usize = 0;

    let mut permutations = shuffle_lines(
        (0..line_count).collect(),
        line_count,
        minbs,
        &mut count,
        estimated_permutation_count,
    )
    .expect("should get some permutations");

    eprintln!("");
    eprintln!(
        "actual permutations to try.......................: {}",
        count
    );

    count = 0;
    optimize_permutations(&mut permutations, &mut count);
    eprintln!("");

    eprintln!(
        "\"actual\" memory usage for all permutations (GiB).: {}",
        permutations.deep_size_of() as f32 / (1024.0_f32.powf(3.0))
    );

    eprintln!(
        "block depth......................................: {}",
        count_permutations(&permutations)
    );

    // When checking which permutation is the best, the current set of permutations
    // should be checked completely. The children of the smallest permutation in the
    // current set should then become the new set to check.
    // Idea #1: A permutation-history should be kept and before actually checking a
    // permutation, it should be made sure that this permutation is not a duplicate
    // of a previously checked permutation.
}
