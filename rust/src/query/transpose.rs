// Added by David Fernández, 2026. See git history for details.

use crate::decode::types::{contour_string_to_contour, ContourString};
use crate::ff_config;

pub type Pitches = Vec<i32>;
pub type Histogram = [f32; ff_config::MIDI_NUM as usize];

// A contour, pre-processed into the representations used by the query engine.
//  Settings are prepared once when the index is loaded; queries once per search.
pub struct PreparedContour {
    // Pitches relative to MIDI_LOW, one per quaver (as in the contour string).
    pub pitches: Pitches,
    // Intervals between consecutive *notes*, encoded one char per interval.
    //  Repeated pitches are merged first, so this ignores rhythm and tempo
    //  quantisation errors, and doesn't depend on the key the tune is played in.
    pub intervals: String,
    // How much time is spent on each pitch, normalised to unit length.
    pub histogram: Histogram,
}

impl PreparedContour {
    pub fn new(contour_string: &ContourString) -> PreparedContour {
        let pitches: Pitches = contour_string_to_contour(contour_string)
            .iter()
            .map(|pitch| (pitch - ff_config::MIDI_LOW) as i32)
            .collect();

        let mut notes = pitches.clone();
        notes.dedup();
        let intervals: String = notes
            .windows(2)
            .map(|pair| interval_to_char(pair[1] - pair[0]))
            .collect();

        let mut histogram: Histogram = [0.; ff_config::MIDI_NUM as usize];
        for pitch in &pitches {
            histogram[*pitch as usize] += 1.;
        }
        let norm = histogram.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0. {
            histogram.iter_mut().for_each(|x| *x /= norm);
        }

        PreparedContour {
            pitches,
            intervals,
            histogram,
        }
    }
}

fn interval_to_char(interval: i32) -> char {
    // Any interval larger than two octaves is almost certainly a transcription
    //  error, so they can all share the same symbol.
    const MAX_INTERVAL: i32 = 24;
    let clamped = interval.clamp(-MAX_INTERVAL, MAX_INTERVAL);
    (b'0' + (clamped + MAX_INTERVAL) as u8) as char
}

pub fn candidate_shifts(query: &Histogram, target: &Histogram, max_candidates: usize) -> Vec<i32> {
    // Rather than aligning the query in every possible key, pick the
    //  transpositions under which the query spends its time on the same
    //  pitches as the target. The original key is always tried first.
    let mut scored_shifts: Vec<(i32, f32)> = (-ff_config::TRANSPOSITION_RANGE
        ..=ff_config::TRANSPOSITION_RANGE)
        .filter(|shift| *shift != 0)
        .map(|shift| (shift, histogram_correlation(query, target, shift)))
        .filter(|(_, score)| *score > 0.)
        .collect();
    scored_shifts.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap());

    let mut shifts = vec![0];
    shifts.extend(
        scored_shifts
            .iter()
            .take(max_candidates)
            .map(|(shift, _)| *shift),
    );
    return shifts;
}

fn histogram_correlation(query: &Histogram, target: &Histogram, shift: i32) -> f32 {
    // Query pitch p is compared against target pitch p + shift
    let mut score = 0.;
    for (pitch, weight) in query.iter().enumerate() {
        let shifted = pitch as i32 + shift;
        if shifted >= 0 && (shifted as usize) < target.len() {
            score += weight * target[shifted as usize];
        }
    }
    return score;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intervals_ignore_key_and_rhythm() {
        let in_d = PreparedContour::new(&"ttxxyyxxqq".to_string());
        let in_f_slower = PreparedContour::new(&"wwwAAABBBAAAtt".to_string());
        assert_eq!(in_d.intervals, in_f_slower.intervals);
        assert_eq!(in_d.intervals.len(), 4);
    }

    #[test]
    fn finds_transposition_from_histogram() {
        let original = PreparedContour::new(&"ttxxyyxxqqvvoooo".to_string());
        let up_three = PreparedContour::new(&"wwAABBAAttyyrrrr".to_string());
        let shifts = candidate_shifts(&original.histogram, &up_three.histogram, 3);
        assert_eq!(shifts[0], 0);
        assert_eq!(shifts[1], 3);
    }
}
