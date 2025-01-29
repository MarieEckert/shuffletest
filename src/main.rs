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
    child_permutations: Vec<Permutation>,
}

/// generate all possible permutations for the given lines. The count and total
/// parameters are used to output a progress indicator.
fn shuffle_lines(
    lines: Vec<usize>,
    mut blocksize: usize,
    min_blocksize: usize,
    count: &mut usize,
    total: usize,
) -> Vec<Permutation> {
    let mut line_permutations: Vec<Permutation> = Vec::new();

    if blocksize == 1 || blocksize < min_blocksize {
        return line_permutations;
    }

    //blocksize = (blocksize as f32 / 2.0).ceil() as usize;
    blocksize /= 2;

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

    line_permutations
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
fn optimize_permutations(
    permutations: &mut Vec<Permutation>,
    count: &mut usize,
    total_perms: usize,
) {
    // The value of this constant could also be used to determine a count of
    // jobs which simultaneously check all permutations.
    const MAX_PARENT_PERMUTATIONS: usize = 2;

    eprint!(
        "optimizing permutation set.......................: {}/{}\r",
        count, total_perms
    );
    *count += 1;

    while permutations.len() > MAX_PARENT_PERMUTATIONS {
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

            for child in chunk {
                parent.child_permutations.push(child.clone());
            }

            new_permutations.push(parent);
        });

        permutations.clear();
        permutations.append(&mut new_permutations);
    }

    for permutation in permutations {
        optimize_permutations(&mut permutation.child_permutations, count, total_perms);
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
    if permutations.is_empty() || permutations[0].child_permutations.is_empty() {
        return 0;
    }

    let mut prev: usize = 0;
    for permutation in permutations {
        let cur = count_permutations(&permutation.child_permutations);
        if cur > prev {
            prev = cur;
        }
    }

    1 + prev
}

fn make_mem_color(mem: f32) -> String {
    let memory_color: String;

    if mem >= 8.0 && mem < 64.0 {
        memory_color = String::from("\x1b[33m");
    } else if mem >= 64.0 {
        memory_color = String::from("\x1b[31m");
    } else {
        memory_color = String::from("\x1b[32m");
    }

    memory_color
}

fn main() {
    let lines = "float i_event0 = 0;           // fade in
float i_event5 = 2 * spsec;   // end of fade in
float i_event28 = 22 * spsec;     // sixth scene (chains far away)
float i_event28 = 22 * spsec;     // sixth scene (chains far away)
float i_event28 = 22 * spsec;     // sixth scene (chains far away)
float i_event80 = 36 * spsec; // second dreamy bright scene
float i_event30 = 24 * spsec;     // start of first ship scene
float i_event80 = 36 * spsec; // second dreamy bright scene"
        .to_string();

    // Minimal block size
    let minbs: usize = 2;

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
    let estimated_memory_usage: f32 = (estimated_permutation_count as f32
        * (std::mem::size_of::<Permutation>() as f32
            + (line_count as f32 * std::mem::size_of::<usize>() as f32))
        * 2.0)
        / (1024.0_f32.powf(3.0));

    eprintln!(
        "estimated memory usage for all permutations (GiB): \x1b[1m{}{}\x1b[0m",
        make_mem_color(estimated_memory_usage),
        estimated_memory_usage
    );

    let mut count: usize = 0;

    let mut permutations = shuffle_lines(
        (0..line_count).collect(),
        line_count,
        minbs,
        &mut count,
        estimated_permutation_count,
    );

    eprintln!("");
    eprintln!(
        "actual permutations to try.......................: {}",
        count
    );

    let mut opt_count = 0;
    optimize_permutations(&mut permutations, &mut opt_count, count);
    eprintln!("");

    let actual_memory_usage: f32 = permutations.deep_size_of() as f32 / (1024.0_f32.powf(3.0));
    eprintln!(
        "\"actual\" memory usage for all permutations (GiB).: \x1b[1m{}{}\x1b[0m",
        make_mem_color(actual_memory_usage),
        actual_memory_usage
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
