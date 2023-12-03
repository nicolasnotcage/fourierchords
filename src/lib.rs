mod note_detection;

pub use egui;
use std::cmp::Ordering;
use nih_plug::prelude::*;
use nih_plug_egui;
use rustfft::{num_complex::Complex, FftPlanner, FftNum, Fft};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::Arc;
use ordered_float::OrderedFloat;
use crate::note_detection::get_note_data;
extern crate rustfft;

struct FourierChords {
    params: Arc<FourierChordsParams>,

    // Need to get upon initialization for proper windowing and FFT
    sample_rate: f32,

    // Buffer size
    buffer_size: u32,

    // Note data for real-time note detection
    note_data: HashMap<OrderedFloat<f32>, String>,

    // Vector of complex buffer values
    complex_buffer: Vec<Complex<f32>>,

    // Starting sample vector
    sample_vec: Vec<f32>,

    // Vector for FFT results
    fft_results: Vec<Complex<f32>>,

    // Vector for windowed values
    windowed_values: Vec<f32>,

    // FFT algorithm object
    fft_algorithm: Arc<dyn Fft<f32>>,

    // Spectrum data object
    spectrum_data: Vec<SpectrumData>,

    // Frequency resolution value. Equal to: Sample Rate / Buffer Size
    frequency_resolution: f32,

    // Nyquist Limit. Equal to: Sample Rate / 2
    nyquist_limit: usize,

    // Vector to hold local maxima
    local_maxima: Vec<SpectrumData>,

    // Vector to hold prominent peaks
    prominent_peaks: Vec<SpectrumData>,

    // Detected notes
    detected_notes: Vec<String>,

    // Magnitude threshold
    magnitude_threshold: f32,

    // Max identified magnitude
    max_magnitude: f32,

    // Prominence threshold
    prominence_threshold: f32,

    // General prominence value
    prominence: f32,
}

#[derive(Params)]
struct FourierChordsParams {
    // #[param(min = 512, max = 8192, step = 512)]
    #[id = "window_size"]
    fft_window_size: IntParam,

    // #[param(min = 0.0, max = 1.0, step = 0.05)]
    #[id = "window_overlap"]
    fft_window_overlap: FloatParam,

    // // #[param(min = 0.0, max = 1.0)]
    // #[id = "magnitude_threshold"]
    // magnitude_threshold: FloatParam,

    // // #[param(min = 0.0, max = 1.0)]
    // #[id = "prominence_threshold"]
    // prominence_threshold: FloatParam,

    // #[param(min = 20.0, max = 20000.0)]
    #[id = "frequency_range_min"]
    frequency_range_min: FloatParam,

    // #[param(min = 20.0, max = 20000.0)]
    #[id = "frequency_range_max"]
    frequency_range_max: FloatParam,

    // #[param(min = 0.0, max = 5.0)]
    #[id = "smoothing_time"]
    smoothing_time: FloatParam,
}

impl Default for FourierChords {
    fn default() -> Self {
        // Initialize an FFT Planner
        let mut planner = FftPlanner::new();

        // Use planner to create FFT algorithm
        let fft_algorithm = planner.plan_fft_forward(1024);

        Self {
            params: Arc::new(FourierChordsParams::default()),
            // Initialize sample rate to standard of 44.1khz
            sample_rate: 44100.0,

            // Initialize default buffer size
            // TODO: Make this adaptable to host's buffer size
            buffer_size: 1024,

            // Initialize note data hashmap
            note_data: get_note_data(),

            // Initialize complex buffer
            // TODO: Change this and the below vectors to adapt to buffer size
            complex_buffer: vec![Complex { re: 0.0, im: 0.0 }; 1024],

            // Initialize sample vector
            sample_vec: Vec::new(),

            // Vector for FFT results
            fft_results: vec![Complex { re: 0.0, im: 0.0 }; 1024],

            // Vector for windowed values
            windowed_values: vec![0.0; 1024],

            // Initialize fft_algorithm to the one initialized in default function
            fft_algorithm,

            // Initialize Spectrum Data object with zeroed values
            spectrum_data: vec![SpectrumData { frequency: 0.0, magnitude: 0.0, index: 0 }; 1024],

            // Initialize frequency resolution
            frequency_resolution: 43.07,

            // Initialize nyquist limit
            nyquist_limit: 22050,

            // TODO: Local maxima and prominent peaks may need more thorough implementation
            //  If you see crashes, try pre-allocating space because dynamic real-time growth may
            //  cause unexpected crashes
            local_maxima: Vec::new(),
            prominent_peaks: Vec::new(),
            detected_notes: Vec::new(),

            // Magnitude threshold
            magnitude_threshold: 0.0,

            // Max magnitude
            max_magnitude: 0.0,

            // Prominence threshold
            prominence_threshold: 0.0,

            // General prominence value
            prominence: 0.0,
        }
    }
}

impl Default for FourierChordsParams {
    fn default() -> Self {
        Self {
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            fft_window_size: IntParam::new("Window Size", 1024, IntRange::Linear { min: 512, max: 8191 }),
            fft_window_overlap: FloatParam::new("Window Overlap", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }),
            // fft_window_type: EnumParam::new("Window Type", WindowType::Hanning),

            // TODO: Define default prominence and magnitude threshold values
            // magnitude_threshold: FloatParam::new("Magnitude Threshold", 0.1, FloatRange::Linear { min: 0.0, max: 1.0 }),
            // prominence_threshold: FloatParam::new("Prominence Threshold", 0.1, FloatRange::Linear { min: 0.0, max: 1.0 }),
            frequency_range_min: FloatParam::new("Frequency Range Min", 20.0, FloatRange::Linear { min: 20.0, max: 20000.0 }),
            frequency_range_max: FloatParam::new("Frequency Range Max", 20000.0, FloatRange::Linear { min: 20.0, max: 20000.0 }),

            // Not sure about how we've defined this, but it's not a big priority right now.
            smoothing_time: FloatParam::new("Smoothing Time", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
        }
    }
}

impl Plugin for FourierChords {
    const NAME: &'static str = "Fourier Chords";
    const VENDOR: &'static str = "Nicolas Miller";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "njmiller1208@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];


    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        // self.sample_rate = _buffer_config.sample_rate as f32;
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Get samples from buffer iterator
        for mut sample_frame in buffer.iter_samples() {
            if let Some(&mut left_sample) = sample_frame.get_mut(0) {
                self.sample_vec.push(left_sample);
            }
        }

        // Apply the window function to the audio data (Hanning, etc.)
        apply_window_function(self);

        // Perform the FFT
        perform_fft(self);

        // Get the spectrum data
        get_spectrum_data(self);

        // Identify notes
        identify_notes(self);

        // Here you can do something with the detected notes, like updating a GUI
        // Update GUI logic goes here
        nih_plug_egui::

        ProcessStatus::Normal
    }
}

impl ClapPlugin for FourierChords {
    const CLAP_ID: &'static str = "com.nicolasmiller.fourierchords";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A utility to read notes from incoming audio using FFT.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for FourierChords {
    const VST3_CLASS_ID: [u8; 16] = *b"FourierChordsNJM";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(FourierChords);
nih_export_vst3!(FourierChords);


// Spectrum Data Structure Definition
#[derive(Debug, Clone)]
struct SpectrumData {
    frequency: f32,
    magnitude: f32,
    index: usize,
}

// Function Definitions
fn perform_fft(fft_chords: &mut FourierChords) -> () {
    // Check that the input and complex_vec are the same length
    assert_eq!(fft_chords.windowed_values.len(),
               fft_chords.complex_buffer.len(),
               "Input and complex_vec must be of the same length");

    // Populate the complex_vec with values from windowed vec
    for (complex, &input_val) in fft_chords.complex_buffer
        .iter_mut()
        .zip(fft_chords.windowed_values.iter()) {
        *complex = Complex { re: input_val, im: 0.0 };
    }

    // Perform forward FFT on input data
    fft_chords.fft_algorithm.process(&mut fft_chords.complex_buffer);
}


// Utility function to apply the window function in place
fn apply_window_function(fourier_chords: &mut FourierChords) {
    for (i, (&sample, windowed_value)) in fourier_chords.sample_vec.iter().zip(fourier_chords.windowed_values.iter_mut()).enumerate() {
        let window_value = 0.5 - 0.5 * ((2.0 * PI * i as f32 / (1024f32 - 1.0)).cos());
        *windowed_value = sample * window_value;
    }
}

// Transforms buffer of complex numbers from FFT forward transform and returns vector of
// SpectrumData, which contains fields for frequencies and magnitudes
fn get_spectrum_data(fft_chords: &mut FourierChords) -> () {
    for (i, spectrum_data) in fft_chords.spectrum_data
        .iter_mut()
        .enumerate()
        .take(fft_chords.nyquist_limit) {

        let frequency = i as f32 * fft_chords.frequency_resolution;
        let magnitude = fft_chords.complex_buffer[i].norm();

        spectrum_data.frequency = frequency;
        spectrum_data.magnitude = magnitude;
        spectrum_data.index = i;
    }
}

// Function to identify notes in the spectrum
fn identify_notes(fft_chords: &mut FourierChords) -> () {
    // Calculate local maxima of given spectrum data
    get_local_maxima(fft_chords);

    // TODO: Finish peak picking algorithm process
    get_prominent_peaks(fft_chords);

    // Initializes frequency to note hash map
    let freq_to_note = get_note_data();

    for value in &fft_chords.prominent_peaks {
        let closest_frequency = freq_to_note
            .keys()
            .min_by(|&freq1, &freq2| {
                (freq1 - value.frequency)
                    .abs()
                    .partial_cmp(&(freq2 - value.frequency).abs())
                    .unwrap()
            })
            .unwrap();

        let note = freq_to_note.get(closest_frequency).unwrap(); // Get the note corresponding to the closest matching frequency

        fft_chords.detected_notes.push(note.clone()); // Clone the note string and add it to matched_notes
    }
}

// TODO: Consider adding these as FFT Chord implementation functions; seems like that would be more
//  straightforward based on how many functions we're defining that depend on traits
// Calculate local maxima
fn get_local_maxima(fft_chords: &mut FourierChords) -> () {
    // Calculate magnitude threshold and assign it. Play with this value to optimize execution time.
    // Currently, we're using a threshold of one third of the maximum magnitude.
    fft_chords.magnitude_threshold = max_magnitude(&fft_chords.spectrum_data) / 3.0;

    // Identifies local maxima and pushes them to maxima vector
    for i in 1..fft_chords.spectrum_data.len() - 1 {
        if fft_chords.spectrum_data[i].magnitude < fft_chords.magnitude_threshold {
            continue;
        }
        if fft_chords.spectrum_data[i].magnitude > fft_chords.spectrum_data[i - 1].magnitude
            && fft_chords.spectrum_data[i].magnitude > fft_chords.spectrum_data[i + 1].magnitude
        {
            fft_chords.local_maxima.push(fft_chords.spectrum_data[i].clone());
        }
    }
}

// Identifies maximum magnitude from SpectrumData vector
// TODO: Error handling
fn max_magnitude(spectrum: &Vec<SpectrumData>) -> f32 {
    spectrum.iter()
        .map(|data| data.magnitude) // Extract the magnitude from each SpectrumData
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)) // Compare magnitudes
        .unwrap_or(0.0) // Handle the case where spectrum_data is empty
}

// TODO: Implement prominent peak picking algorithm
fn get_prominent_peaks(fft_chords: &mut FourierChords) -> () {
    fft_chords.prominence_threshold = max_magnitude(&fft_chords.local_maxima) / 4.0;

    for peak in &fft_chords.local_maxima {
        fft_chords.prominence = calculate_prominence(&fft_chords.spectrum_data, peak.index);
        if fft_chords.prominence >= fft_chords.prominence_threshold {
            fft_chords.prominent_peaks.push(peak.clone());
        }
    }
}

fn calculate_prominence(spectrum: &[SpectrumData], peak_index: usize) -> f32 {
    // Find the lowest contour line around the peak
    let mut left_min = f32::MAX;
    let mut right_min = f32::MAX;

    // Find the left minimum by iterating to the left of the peak until we find a peak that's
    // higher or end of spectrum is reached
    for i in (0..peak_index).rev() {
        if spectrum[i].magnitude > spectrum[peak_index].magnitude {
            break;
        }

        left_min = right_min.min(spectrum[i].magnitude);
    }

    for i in peak_index + 1..spectrum.len() {
        if spectrum[i].magnitude > spectrum[peak_index].magnitude {
            break;
        }
        right_min = right_min.min(spectrum[i].magnitude);
    }

    // The prominence is the height of the peak minus the maximum of the left and right minima (since we want the higher trough)
    let prominence = spectrum[peak_index].magnitude - left_min.max(right_min);

    // Return prominence
    prominence
}