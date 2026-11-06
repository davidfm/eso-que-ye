// Added by David Fernández, 2026. See git history for details.
//
// Rough sanity check of the query engine without real recordings. Queries are
//  slices of the indexed contours themselves, with transcription-like errors
//  and optional transposition added. Real recordings remain the proper test.
//
// cargo run --release --example synthetic_eval -- ../app/public/folkfriend-non-user-data.json

use folkfriend::decode::types::{contour_string_to_contour, contour_to_contour_string, Contour};
use folkfriend::ff_config;
use folkfriend::index::tune_index_from_string;
use folkfriend::query::QueryEngine;
use std::env;
use std::fs;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn chance(&mut self, p: f32) -> bool {
        (self.next() % 10_000) as f32 / 10_000. < p
    }
}

struct Scenario {
    name: &'static str,
    quavers: usize,
    transpose: bool,
    error_rate: f32,
    octave_error_rate: f32,
    reversed: bool,
}

fn make_query(contour: &Contour, s: &Scenario, rng: &mut Rng) -> Option<Contour> {
    if contour.len() < s.quavers {
        return None;
    }
    let start = rng.below(contour.len() - s.quavers + 1);
    let mut slice: Contour = contour[start..start + s.quavers].to_vec();
    if s.reversed {
        slice.reverse();
    }

    // Split into notes, then apply errors per note
    let mut notes: Vec<(i32, usize)> = Vec::new();
    for pitch in slice {
        match notes.last_mut() {
            Some((p, d)) if *p == pitch as i32 => *d += 1,
            _ => notes.push((pitch as i32, 1)),
        }
    }
    let shift = if s.transpose {
        let shifts = [-7, -5, -4, -3, -2, 2, 3, 4, 5, 7];
        shifts[rng.below(shifts.len())]
    } else {
        0
    };
    let mut query = Contour::new();
    for (pitch, duration) in notes {
        if rng.chance(s.error_rate / 2.) {
            continue; // Missed note
        }
        let mut pitch = pitch + shift;
        if rng.chance(s.error_rate) {
            pitch += [-2, -1, 1, 2][rng.below(4)];
        } else if rng.chance(s.octave_error_rate) {
            pitch += if rng.chance(0.5) { 12 } else { -12 };
        }
        let mut duration = duration as i32;
        if rng.chance(s.error_rate) {
            duration += if rng.chance(0.5) { 1 } else { -1 };
        }
        let low = ff_config::MIDI_LOW as i32;
        let high = ff_config::MIDI_HIGH as i32;
        if pitch < low || pitch > high {
            if pitch + shift - 12 >= low && pitch + shift + 12 <= high {
                continue; // Octave error pushed the note out of range: drop it
            }
            return None;
        }
        query.extend(vec![pitch as u32; duration.max(1) as usize]);
    }
    Some(query)
}

fn main() {
    let path = env::args().nth(1).expect("Pass the path to the tune index JSON");
    let index_json = fs::read_to_string(path).unwrap();

    let mut engines: Vec<(&str, QueryEngine)> = Vec::new();
    for (name, candidates) in [("original key only", 0), ("with transposition", ff_config::TRANSPOSITION_CANDIDATES)] {
        let mut engine = QueryEngine::new();
        engine.use_tune_index(tune_index_from_string(&index_json));
        engine.transposition_candidates = candidates;
        engines.push((name, engine));
    }

    let tune_index = tune_index_from_string(&index_json);
    let mut setting_ids: Vec<&String> = tune_index.settings.keys().collect();
    setting_ids.sort();

    let scenarios = [
        Scenario { name: "~10 s, same key, clean", quavers: 40, transpose: false, error_rate: 0.0, octave_error_rate: 0.0, reversed: false },
        Scenario { name: "~10 s, same key, noisy", quavers: 40, transpose: false, error_rate: 0.15, octave_error_rate: 0.0, reversed: false },
        Scenario { name: "~10 s, other key, noisy", quavers: 40, transpose: true, error_rate: 0.15, octave_error_rate: 0.0, reversed: false },
        Scenario { name: "~25 s, same key, noisy", quavers: 100, transpose: false, error_rate: 0.15, octave_error_rate: 0.0, reversed: false },
        Scenario { name: "~25 s, other key, noisy", quavers: 100, transpose: true, error_rate: 0.15, octave_error_rate: 0.0, reversed: false },
        Scenario { name: "~10 s, same key, noisy + octave errors", quavers: 40, transpose: false, error_rate: 0.15, octave_error_rate: 0.1, reversed: false },
        Scenario { name: "~10 s, other key, noisy + octave errors", quavers: 40, transpose: true, error_rate: 0.15, octave_error_rate: 0.1, reversed: false },
        Scenario { name: "~10 s, not in index", quavers: 40, transpose: false, error_rate: 0.15, octave_error_rate: 0.0, reversed: true },
        Scenario { name: "~25 s, not in index", quavers: 100, transpose: false, error_rate: 0.15, octave_error_rate: 0.0, reversed: true },
    ];
    const QUERIES: usize = 300;

    for scenario in &scenarios {
        println!("\n== {} ==", scenario.name);
        for (engine_name, engine) in &engines {
            let mut rng = Rng(0x5eed_1234_abcd);
            let (mut top1, mut top5, mut n) = (0, 0, 0);
            let mut top_scores: Vec<f32> = Vec::new();
            let mut margins: Vec<f32> = Vec::new();
            while n < QUERIES {
                let setting_id = setting_ids[rng.below(setting_ids.len())];
                let setting = &tune_index.settings[setting_id];
                let contour = contour_string_to_contour(&setting.contour);
                let query = match make_query(&contour, scenario, &mut rng) {
                    Some(q) if q.len() > 4 => q,
                    _ => continue,
                };
                let results = engine
                    .run_contour_query(&contour_to_contour_string(&query))
                    .unwrap();
                n += 1;
                let rank = results.iter().position(|r| r.setting.tune_id == setting.tune_id);
                if rank == Some(0) {
                    top1 += 1;
                }
                if matches!(rank, Some(r) if r < 5) {
                    top5 += 1;
                }
                top_scores.push(results[0].score);
                margins.push(results[0].score - results[1].score);
            }
            top_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
            margins.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let pct = |q: f32| top_scores[((n - 1) as f32 * q) as usize];
            let mpct = |q: f32| margins[((n - 1) as f32 * q) as usize];
            if scenario.reversed {
                print!("  {:<20}", engine_name);
            } else {
                print!(
                    "  {:<20} top-1 {:>5.1}%  top-5 {:>5.1}%",
                    engine_name,
                    100. * top1 as f32 / n as f32,
                    100. * top5 as f32 / n as f32
                );
            }
            println!(
                "  | best score p10/p50/p90 {:.2}/{:.2}/{:.2}  margin p50/p90 {:.2}/{:.2}",
                pct(0.1), pct(0.5), pct(0.9), mpct(0.5), mpct(0.9)
            );
        }
    }
}
