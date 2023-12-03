// use crate::perform_fft::SpectrumData;
use ::ordered_float::OrderedFloat;
use rustfft::num_complex::Complex;
use std::cmp::Ordering;
use std::collections::HashMap;


// This function is the main algorithm to identify notes in a collection of spectrum data. It begins
// by identifying local maxima. It then performs prominence-based peak picking. It then performs
// width-based peak picking. This series of algorithms should isolate and identify fundamental
// frequencies from the given data.
// pub fn identify_notes(spectrum: Vec<SpectrumData>) -> Vec<String> {
//     // Calculate local maxima of given spectrum data
//     let local_maxima = get_local_maxima(&spectrum);
//     println!("{:?}", local_maxima[0]);
//
//     // TODO: Finish peak picking algorithm process
//     let prominent_peaks = get_prominent_peaks(&spectrum, &local_maxima);
//     println!("{:?}", prominent_peaks[0]);
//
//     // Initializes frequency to note hash map
//     let freq_to_note = get_note_data();
//
//     // Initialize vector to store notes
//     let mut notes: Vec<String> = Vec::new();
//
//     for value in &prominent_peaks {
//         let closest_frequency = freq_to_note
//             .keys()
//             .min_by(|&freq1, &freq2| {
//                 (freq1 - value.frequency)
//                     .abs()
//                     .partial_cmp(&(freq2 - value.frequency).abs())
//                     .unwrap()
//             })
//             .unwrap();
//
//         let note = freq_to_note.get(closest_frequency).unwrap(); // Get the note corresponding to the closest matching frequency
//
//         notes.push(note.clone()); // Clone the note string and add it to matched_notes
//     }
//
//     notes
// }

// Identifies maximum magnitude from SpectrumData vector
// // TODO: Error handling
// fn max_magnitude(spectrum: &Vec<SpectrumData>) -> f32 {
//     let max_magnitude = spectrum.iter().max_by(|a, b| {
//         a.magnitude
//             .partial_cmp(&b.magnitude)
//             .unwrap_or(Ordering::Equal)
//     });
//
//     max_magnitude.unwrap().magnitude
// }

// Determines local maxima present among frequencies and returns vector of these maxima
// fn get_local_maxima(spectrum: &Vec<SpectrumData>) -> Vec<SpectrumData> {
//     // Calculate magnitude threshold and assign it. Play with this value to optimize execution time.
//     // Currently, we're using a threshold of half of the maximum magnitude.
//     let magnitude_threshold: f32 = max_magnitude(&spectrum) / 3.0;
//
//     // Initialize vector to store local maxima.
//     let mut local_maxima: Vec<SpectrumData> = Vec::new();
//
//     // Identifies local maxima and pushes them to maxima vector
//     for i in 1..spectrum.len() - 1 {
//         if spectrum[i].magnitude < magnitude_threshold {
//             continue;
//         }
//         if spectrum[i].magnitude > spectrum[i - 1].magnitude
//             && spectrum[i].magnitude > spectrum[i + 1].magnitude
//         {
//             local_maxima.push(spectrum[i].clone());
//         }
//     }
//
//     local_maxima
// }
//
// // TODO: Implement prominent peak picking algorithm
// fn get_prominent_peaks(
//     spectrum: &Vec<SpectrumData>,
//     maxima: &Vec<SpectrumData>,
// ) -> Vec<SpectrumData> {
//     let mut prominent_peaks = Vec::new();
//     let prominence_threshold = max_magnitude(&maxima) / 4.0;
//
//     for peak in maxima {
//         let prominence = calculate_prominence(&spectrum, peak.index);
//         if prominence >= prominence_threshold {
//             prominent_peaks.push(peak.clone());
//         }
//     }
//
//     prominent_peaks
// }
//
// fn calculate_prominence(spectrum: &[SpectrumData], peak_index: usize) -> f32 {
//     // Find the lowest contour line around the peak
//     let mut left_min = f32::MAX;
//     let mut right_min = f32::MAX;
//
//     // Find the left minimum by iterating to the left of the peak until we find a peak that's
//     // higher or end of spectrum is reached
//     for i in (0..peak_index).rev() {
//         if spectrum[i].magnitude > spectrum[peak_index].magnitude {
//             break;
//         }
//
//         left_min = right_min.min(spectrum[i].magnitude);
//     }
//
//     for i in peak_index + 1..spectrum.len() {
//         if spectrum[i].magnitude > spectrum[peak_index].magnitude {
//             break;
//         }
//
//         right_min = right_min.min(spectrum[i].magnitude);
//     }
//
//     // The prominence is the height of the peak minus the maximum of the left and right minima (since we want the higher trough)
//     let prominence = spectrum[peak_index].magnitude - left_min.max(right_min);
//
//     // Return prominence
//     prominence
// }

// Initializes HashMap of frequency and note values with f32 frequency values as keys
pub fn get_note_data() -> HashMap<OrderedFloat<f32>, String> {
    // // Get JSON data
    // let data = read_to_string(filename).expect("Unable to read JSON file.");
    //
    // // Convert JSON into initial hashmap with f32 frequencies
    // let note_data: HashMap<String, f32> = from_str(&data).expect("Unable to parse JSON.");
    //
    // // Invert and return hashmap that maps frequency to notes with Ordered Floats. This allows us to use floating point values as keys.
    // let freq_to_note = note_data
    //     .into_iter()
    //     .map(|(note, freq)| (OrderedFloat(freq), note))
    //     .collect();

    let note_data = HashMap::from([
        (OrderedFloat(880.0), "A5".to_string()),
        (OrderedFloat(1661.22), "G#6".to_string()),
        (OrderedFloat(29.14), "A#0".to_string()),
        (OrderedFloat(32.7), "C1".to_string()),
        (OrderedFloat(36.71), "D1".to_string()),
        (OrderedFloat(246.94), "B3".to_string()),
        (OrderedFloat(3322.44), "G#7".to_string()),
        (OrderedFloat(311.13), "D#4".to_string()),
        (OrderedFloat(1864.66), "A#6".to_string()),
        (OrderedFloat(196.0), "G3".to_string()),
        (OrderedFloat(4978.03), "D#8".to_string()),
        (OrderedFloat(24.5), "G0".to_string()),
        (OrderedFloat(2793.83), "F7".to_string()),
        (OrderedFloat(146.83), "D3".to_string()),
        (OrderedFloat(55.0), "A1".to_string()),
        (OrderedFloat(1760.0), "A6".to_string()),
        (OrderedFloat(46.25), "F#1".to_string()),
        (OrderedFloat(1108.73), "C#6".to_string()),
        (OrderedFloat(92.5), "F#2".to_string()),
        (OrderedFloat(2637.02), "E7".to_string()),
        (OrderedFloat(21.83), "F0".to_string()),
        (OrderedFloat(123.47), "B2".to_string()),
        (OrderedFloat(392.0), "G4".to_string()),
        (OrderedFloat(2349.32), "D7".to_string()),
        (OrderedFloat(369.99), "F#4".to_string()),
        (OrderedFloat(27.5), "A0".to_string()),
        (OrderedFloat(61.74), "B1".to_string()),
        (OrderedFloat(116.54), "A#2".to_string()),
        (OrderedFloat(18.35), "D0".to_string()),
        (OrderedFloat(38.89), "D#1".to_string()),
        (OrderedFloat(2489.02), "D#7".to_string()),
        (OrderedFloat(77.78), "D#2".to_string()),
        (OrderedFloat(69.3), "C#2".to_string()),
        (OrderedFloat(1396.91), "F6".to_string()),
        (OrderedFloat(1975.53), "B6".to_string()),
        (OrderedFloat(233.08), "A#3".to_string()),
        (OrderedFloat(185.0), "F#3".to_string()),
        (OrderedFloat(49.0), "G1".to_string()),
        (OrderedFloat(87.31), "F2".to_string()),
        (OrderedFloat(2217.46), "C#7".to_string()),
        (OrderedFloat(2093.0), "C7".to_string()),
        (OrderedFloat(30.87), "B0".to_string()),
        (OrderedFloat(3951.07), "B7".to_string()),
        (OrderedFloat(98.0), "G2".to_string()),
        (OrderedFloat(493.88), "B4".to_string()),
        (OrderedFloat(17.32), "C#0".to_string()),
        (OrderedFloat(1174.66), "D6".to_string()),
        (OrderedFloat(51.91), "G#1".to_string()),
        (OrderedFloat(698.46), "F5".to_string()),
        (OrderedFloat(554.37), "C#5".to_string()),
        (OrderedFloat(466.16), "A#4".to_string()),
        (OrderedFloat(164.81), "E3".to_string()),
        (OrderedFloat(130.81), "C3".to_string()),
        (OrderedFloat(783.99), "G5".to_string()),
        (OrderedFloat(622.25), "D#5".to_string()),
        (OrderedFloat(277.18), "C#4".to_string()),
        (OrderedFloat(1567.98), "G6".to_string()),
        (OrderedFloat(659.26), "E5".to_string()),
        (OrderedFloat(932.33), "A#5".to_string()),
        (OrderedFloat(19.45), "D#0".to_string()),
        (OrderedFloat(523.25), "C5".to_string()),
        (OrderedFloat(138.59), "C#3".to_string()),
        (OrderedFloat(23.12), "F#0".to_string()),
        (OrderedFloat(3520.0), "A7".to_string()),
        (OrderedFloat(987.77), "B5".to_string()),
        (OrderedFloat(1318.51), "E6".to_string()),
        (OrderedFloat(587.33), "D5".to_string()),
        (OrderedFloat(82.41), "E2".to_string()),
        (OrderedFloat(43.65), "F1".to_string()),
        (OrderedFloat(440.0), "A4".to_string()),
        (OrderedFloat(293.66), "D4".to_string()),
        (OrderedFloat(110.0), "A2".to_string()),
        (OrderedFloat(220.0), "A3".to_string()),
        (OrderedFloat(349.23), "F4".to_string()),
        (OrderedFloat(65.41), "C2".to_string()),
        (OrderedFloat(329.63), "E4".to_string()),
        (OrderedFloat(103.83), "G#2".to_string()),
        (OrderedFloat(174.61), "F3".to_string()),
        (OrderedFloat(20.6), "E0".to_string()),
        (OrderedFloat(1046.5), "C6".to_string()),
        (OrderedFloat(41.2), "E1".to_string()),
        (OrderedFloat(4434.92), "C#8".to_string()),
        (OrderedFloat(261.63), "C4".to_string()),
        (OrderedFloat(3135.96), "G7".to_string()),
        (OrderedFloat(73.42), "D2".to_string()),
        (OrderedFloat(4186.01), "C8".to_string()),
        (OrderedFloat(4698.64), "D8".to_string()),
        (OrderedFloat(1244.51), "D#6".to_string()),
        (OrderedFloat(2959.96), "F#7".to_string()),
        (OrderedFloat(34.65), "C#1".to_string()),
        (OrderedFloat(155.56), "D#3".to_string()),
        (OrderedFloat(3729.31), "A#7".to_string()),
        (OrderedFloat(739.99), "F#5".to_string()),
        (OrderedFloat(16.35), "C0".to_string()),
        (OrderedFloat(25.96), "G#0".to_string()),
        (OrderedFloat(830.61), "G#5".to_string()),
        (OrderedFloat(415.3), "G#4".to_string()),
        (OrderedFloat(58.27), "A#1".to_string()),
        (OrderedFloat(1479.98), "F#6".to_string()),
        (OrderedFloat(207.65), "G#3".to_string())]);

    note_data
}
