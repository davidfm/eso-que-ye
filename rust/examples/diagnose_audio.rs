// Added by David Fernández, 2026. See git history for details.
//
// Shows what the engine does with a recording: the transcribed contour, the
//  top results, how well the expected tune matches at every transposition, and
//  how far the instrument is from A = 440 Hz.
//
// cargo run --release --example diagnose_audio -- <index.json> <tune_id | reference.wav> <recording.wav>...
//
// With a reference .wav instead of a tune ID, the reference is transcribed and
//  added to the index as tune "reference". Recordings must be 16-bit PCM WAV.

use folkfriend::decode::types::{contour_string_to_contour, contour_to_contour_string};
use folkfriend::decode::FeatureDecoder;
use folkfriend::feature::FeatureExtractor;
use folkfriend::ff_config;
use folkfriend::index::schema::Setting;
use folkfriend::index::{tune_index_from_string, TuneIndex};
use folkfriend::query::QueryEngine;
use rustfft::{num_complex::Complex, FftPlanner};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};

fn read_wav(path: &str) -> (Vec<f32>, u32) {
    let (header, data) = wav::read(&mut File::open(path).unwrap()).unwrap();
    let signal = data.try_into_sixteen().expect("Expected 16-bit PCM WAV");
    let channels = header.channel_count as usize;
    let mono = signal
        .chunks(channels)
        .map(|frame| frame.iter().map(|x| *x as f32 / 32768.).sum::<f32>() / channels as f32)
        .collect();
    (mono, header.sampling_rate)
}

fn transcribe(signal: Vec<f32>, sample_rate: u32) -> String {
    let mut extractor = FeatureExtractor::new(sample_rate).expect("Invalid sample rate");
    let decoder = FeatureDecoder::new(sample_rate).unwrap();
    extractor.feed_signal(signal);
    println!("tuning estimated by the app: {:+.0} cents", extractor.tuning_offset_cents());
    let mut features = extractor.features();
    let path = decoder.decode_lattice_path(&mut features).unwrap_or_default();
    decoder.decode_contour(&path, &features).unwrap_or_default()
}

fn tuning_offset_cents(signal: &[f32], sample_rate: u32) -> Option<f32> {
    // Strongest spectral peak per window, refined by parabolic interpolation,
    //  then a circular mean of its distance from the nearest semitone.
    const N: usize = 8192;
    let fft = FftPlanner::<f32>::new().plan_fft_forward(N);
    let (mut sum_sin, mut sum_cos) = (0f32, 0f32);
    for window in signal.chunks_exact(N).step_by(1) {
        let mut buf: Vec<Complex<f32>> = window
            .iter()
            .enumerate()
            .map(|(i, x)| {
                let hann = 0.5 - 0.5 * (2. * std::f32::consts::PI * i as f32 / N as f32).cos();
                Complex { re: x * hann, im: 0. }
            })
            .collect();
        fft.process(&mut buf);
        let mags: Vec<f32> = buf[..N / 2].iter().map(|c| c.norm()).collect();
        let lo = (200. * N as f32 / sample_rate as f32) as usize;
        let hi = (2500. * N as f32 / sample_rate as f32) as usize;
        let (peak, &mag) = mags[lo..hi]
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap();
        let k = peak + lo;
        let (a, b, c) = (mags[k - 1].ln(), mags[k].ln(), mags[k + 1].ln());
        let delta = 0.5 * (a - c) / (a - 2. * b + c);
        let freq = (k as f32 + delta) * sample_rate as f32 / N as f32;
        let midi = 69. + 12. * (freq / 440.).log2();
        let angle = 2. * std::f32::consts::PI * (midi - midi.round());
        sum_sin += mag * angle.sin();
        sum_cos += mag * angle.cos();
    }
    if sum_sin == 0. && sum_cos == 0. {
        return None;
    }
    Some(100. * sum_sin.atan2(sum_cos) / (2. * std::f32::consts::PI))
}

fn shift_contour(contour: &str, shift: i32) -> String {
    // Notes pushed out of range are clamped, which just makes them mismatch
    let shifted: Vec<u32> = contour_string_to_contour(&contour.to_string())
        .iter()
        .map(|p| (*p as i32 + shift).clamp(ff_config::MIDI_LOW as i32, ff_config::MIDI_HIGH as i32) as u32)
        .collect();
    contour_to_contour_string(&shifted)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: diagnose_audio <index.json> <tune_id | reference.wav> <recording.wav>...");
        std::process::exit(1);
    }
    let mut index: TuneIndex = tune_index_from_string(&fs::read_to_string(&args[1]).unwrap());

    let expected_tune = if args[2].ends_with(".wav") {
        let (signal, sample_rate) = read_wav(&args[2]);
        let contour = transcribe(signal, sample_rate);
        println!("reference contour: {}\n", contour);
        index.settings.insert(
            "reference".to_string(),
            Setting {
                tune_id: "reference".to_string(),
                meter: String::new(),
                mode: String::new(),
                abc: String::new(),
                dance: String::new(),
                contour,
            },
        );
        index.aliases.insert("reference".to_string(), vec!["(reference recording)".to_string()]);
        "reference".to_string()
    } else {
        args[2].clone()
    };

    let expected_settings: HashMap<String, Setting> = index
        .settings
        .iter()
        .filter(|(_, s)| s.tune_id == expected_tune)
        .map(|(id, s)| (id.clone(), s.clone()))
        .collect();
    assert!(!expected_settings.is_empty(), "Tune ID {} not in index", expected_tune);
    for (id, s) in &expected_settings {
        println!("expected setting {}: {}", id, s.contour);
    }
    let mut single = QueryEngine::new();
    single.use_tune_index(TuneIndex {
        settings: expected_settings,
        aliases: vec![(expected_tune.clone(), vec![expected_tune.clone()])].into_iter().collect(),
    });
    single.transposition_candidates = 0;

    let index_json = serde_json::to_string(&index).unwrap();
    let mut engine = QueryEngine::new();
    engine.use_tune_index(tune_index_from_string(&index_json));

    for path in &args[3..] {
        let (signal, sample_rate) = read_wav(path);
        println!("\n=== {} ({:.1} s) ===", path, signal.len() as f32 / sample_rate as f32);
        if let Some(cents) = tuning_offset_cents(&signal, sample_rate) {
            println!("tuning: {:+.0} cents from A = 440 Hz", cents);
        }
        let contour = transcribe(signal, sample_rate);
        println!("contour ({} quavers): {}", contour.len(), contour);
        if contour.is_empty() {
            continue;
        }

        let results = engine.run_contour_query(&contour).unwrap();
        for (rank, r) in results.iter().take(5).enumerate() {
            let marker = if r.setting.tune_id == expected_tune { "  <== expected" } else { "" };
            println!(
                "  #{} {:.3}  {:+} st  {}{}",
                rank + 1, r.score, r.transposition, r.display_name, marker
            );
        }
        if let Some(rank) = results.iter().position(|r| r.setting.tune_id == expected_tune) {
            println!("  expected tune ranked #{}", rank + 1);
        } else {
            println!("  expected tune not in results");
        }

        // Score of the expected tune with the recording shifted by every
        //  amount, regardless of which keys the engine chose to try.
        let mut by_shift: Vec<(i32, f32)> = (-24..=24)
            .filter_map(|shift| {
                let shifted = shift_contour(&contour, shift);
                let score = single.run_contour_query(&shifted).ok()?.first()?.score;
                Some((-shift, score))
            })
            .collect();
        by_shift.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let best: Vec<String> = by_shift
            .iter()
            .take(4)
            .map(|(t, s)| format!("{:+} st: {:.3}", t, s))
            .collect();
        println!("  expected tune, best transpositions: {}", best.join(", "));
    }
}
