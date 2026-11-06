// Modified by David Fernández, 2026. See git history for details.

use std::cmp;

const MATCH_SCORE: i32 = 2;
const MISMATCH_SCORE: i32 = -2;
// Same note an octave or more away, a common pitch tracking error
const OCTAVE_MATCH_SCORE: i32 = 1;
const GAP_SCORE: i32 = -1;

pub fn needleman_wunsch(query: &[i32], target: &[i32], shift: i32) -> f32 {

    //  Memory-efficient version of Needleman-Wunsch written for Rust.
    //    ~ Tom Wyllie 2021

    //  `shift` is added to every query pitch before comparing, so the query
    //    can be aligned as if it had been played in a different key.

    if query.len() == 0 || target.len() == 0 {
        return 0.0;
    }

    // Swap a and b such that b always longer than a. The shift follows the query.
    let (a, b, a_shift) = if query.len() > target.len() {
        (target, query, -shift)
    } else {
        (query, target, shift)
    };

    let mut last_row: Vec<i32> = vec![0; a.len() + 1];
    let last_col_index: usize = a.len();

    //       a1 a2 a3 .. aN
    //    b1
    //    b2
    //    b3
    //    ..
    //    bN

    let mut prev_diag: i32;
    let mut curr_diag: i32;

    //  Populate dynamic programming lattice
    for row in 0..b.len() {
        prev_diag = 0;
        for col in 1..last_row.len() {
            curr_diag = prev_diag;
            prev_diag = last_row[col];

            //  We store the previous row in this buffer and work along it,
            //    updating it to "this" row. This is for efficiency.
            last_row[col] = cmp::max(
                    curr_diag + match_score(a[col - 1] + a_shift, b[row]),
                    cmp::max(
                        last_row[col - 1] + GAP_SCORE,
                        last_row[col] + GAP_SCORE
                    ));

            last_row[last_col_index] = cmp::max(last_row[last_col_index], prev_diag);
        }
    }

    //  Normalised [0, 1].
    let highscore: f32 = *last_row.iter().max().unwrap_or(&0) as f32;
    let norm_const: f32 = a.len() as f32;
    return 0.5 * highscore / norm_const;
}

fn match_score(x: i32, y: i32) -> i32 {
    if x == y {
        MATCH_SCORE
    } else if (x - y) % 12 == 0 {
        OCTAVE_MATCH_SCORE
    } else {
        MISMATCH_SCORE
    }
}
