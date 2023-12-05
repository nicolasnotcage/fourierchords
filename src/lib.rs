mod note_detection;

use std::cmp::Ordering;
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use rustfft::{num_complex::Complex, FftPlanner, FftNum, Fft};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use ordered_float::OrderedFloat;
use crate::note_detection::get_note_data;
extern crate rustfft;

struct FourierChords {
    params: Arc<FourierChordsParams>,

    // Need to get upon initialization for proper windowing and FFT
    sample_rate: f32,

    // Buffer size
    window_size: usize,

    // Note data for real-time note detection
    note_data: HashMap<OrderedFloat<f32>, String>,

    // Vector of complex buffer values
    complex_buffer: Vec<Complex<f32>>,

    // Starting sample vector
    sample_vec: Vec<f32>,

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

    // String that is used to output the notes
    notes_string_for_display: String,

    // Utility index counter for overwriting sample_vec values
    counter: usize,

    // Boolean for debug printing buffer size
    buffer_displayed: bool,
}

#[derive(Params)]
struct FourierChordsParams {
    // Editor state
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    // Output for notes identified by the algorithm
    #[persist = "notes-output"]
    notes_output: Arc<Mutex<String>>,

    // Debug tracking
    debug_messages: Arc<Mutex<String>>,
}

impl Default for FourierChords {
    fn default() -> Self {
        // Initialize an FFT Planner
        let mut planner = FftPlanner::new();

        // Use planner to create FFT algorithm
        // Update value here if changing window size. Testing showed 65,536 to be a good balance
        // between performance and algorithm accuracy.
        let fft_algorithm = planner.plan_fft_forward(65536);

        Self {
            params: Arc::new(FourierChordsParams::default()),

            // Initialize sample rate to standard of 44.1khz. Will be updated in initialize function.
            sample_rate: 44100.0,

            // Initialize de
            window_size: 65536,

            // Initialize note data hashmap
            // TODO: This should likely be handled by initialization function.
            note_data: get_note_data(),

            // Initialize complex buffer
            complex_buffer: Vec::new(),

            // Initialize sample vector
            // sample_vec: vec![0.0; 264600],
            sample_vec: Vec::new(),

            // Vector for windowed values
            windowed_values: Vec::new(),

            // Initialize fft_algorithm to the one initialized in default function
            fft_algorithm,

            // Initialize Spectrum Data object with zeroed values
            spectrum_data: Vec::new(),

            // Initialize default frequency resolution. Will be calculated again when spectrum is created.
            frequency_resolution: 0.16666667,

            // Initialize default nyquist limit. Will be calculated again when spectrum is created.
            nyquist_limit: 132300,

            // Detection vectors
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

            // Default note display value
            notes_string_for_display: "".to_string(),

            // Utility index counter for overwriting sample_vec values
            counter: 0,

            // Defaults to false
            buffer_displayed: false,
        }
    }
}

impl Default for FourierChordsParams {
    fn default() -> Self {
        Self {
            // Default editor state           ]]
            editor_state: EguiState::from_size(400, 300),

            // Default note value
            notes_output: Arc::new(Mutex::new("".to_string())),

            // Default debug message
            debug_messages: Arc::new(Mutex::new("".to_string())),
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

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let editor_state = self.params.editor_state.clone();
        let notes_output = self.params.notes_output.clone();
        let debug_messages = self.params.debug_messages.clone();

        create_egui_editor(
            editor_state,
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    // Display a static label for "Identified Notes"
                    ui.vertical_centered(|ui| {
                        // Display "Identified Notes" with custom style
                        ui.label(
                            egui::RichText::new("Identified Notes")
                                .strong()
                                .size(30.0) // Example font size, adjust as needed
                        );
                    });

                    // Display the dynamically loaded notes
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Ok(notes_output) = notes_output.lock() {
                            // Center the dynamically loaded notes
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("{}", *notes_output))
                                        .size(16.0)
                                );
                            });
                        }
                    });

                    // Flexible space to push the debug area to the bottom
                    ui.add_space(ui.available_height() / 1.15);

                    // Scrollable debugging area, located beneath the notes display
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Ok(debug_messages) = debug_messages.lock() {
                            ui.vertical_centered(|ui| {
                                ui.label(&*debug_messages);
                            });
                        }
                    });
                });
            },
        )
    }


    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function.

        // Set sample rate
        self.sample_rate = _buffer_config.sample_rate;

        // Set all FFT buffers to be equal to desired window size
        self.complex_buffer.resize(self.window_size, Complex { re: 0.0, im: 0.0 });

        self.windowed_values.resize(self.window_size, 0.0);

        self.spectrum_data.resize(self.window_size, SpectrumData { frequency: 0.0, magnitude: 0.0, index: 0 });

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

        // Print buffer size to debug window
        if !self.buffer_displayed {
            {
                let message = format!("Sample Rate: {:?}\nBuffer Size: {:?}", self.sample_rate, buffer.samples());
                let mut debug_messages = self.params.debug_messages.lock().unwrap();
                debug_messages.push_str(&message);
            }

            self.buffer_displayed = true;
        }

        for mut sample_frame in buffer.iter_samples() {
            if let Some(&mut left_sample) = sample_frame.get_mut(0) {
                // Avoids adding noise
                if left_sample.abs() > 0.001 {
                    self.sample_vec.push(left_sample);
                }

                // Check if we've reached the desired sample count
                if self.sample_vec.len() >= self.window_size {
                    // Perform analysis (e.g., FFT)
                    perform_analysis(self);

                    // Clear the sample_vec or handle overlap
                    self.sample_vec.clear(); // or handle overlap as needed
                    break; // Important to break to avoid overfilling the buffer
                }
            }
        }


        // Update GUI with newly detected notes
        if self.params.editor_state.is_open() && !self.detected_notes.is_empty() {
            // Build a string from the detected notes
            let new_notes_string = self.detected_notes.join(", ");

            // Update only if there are new notes to display
            if !self.notes_string_for_display.contains(&new_notes_string) {
                let updated_notes = if new_notes_string.is_empty() {
                    "None".to_string()
                } else {
                    new_notes_string
                };

                // Update the shared notes_output variable
                if let Ok(mut notes_output) = self.params.notes_output.lock() {
                    *notes_output = updated_notes;
                }

                // Now that we've updated the GUI, clear the detected_notes
                // self.detected_notes.clear();
            }
        }

        self.local_maxima.clear();
        self.prominent_peaks.clear();

        // This breaks the VST
        // self.spectrum_data.clear();

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

// Executes algorithm
fn perform_analysis(fourier_chords: &mut FourierChords) {
    // Apply the window function to the audio data (Hanning, etc.)
    apply_window_function(fourier_chords);

    // Perform the FFT
    perform_fft(fourier_chords);

    // Get the spectrum data
    get_spectrum_data(fourier_chords);


    // Identify notes
    identify_notes(fourier_chords);
}

// Function Definitions
fn perform_fft(fourier_chords: &mut FourierChords) -> () {
    // Check that the input and complex_vec are the same length
    assert_eq!(fourier_chords.windowed_values.len(),
               fourier_chords.complex_buffer.len(),
               "Input and complex_vec must be of the same length");

    // Populate the complex_vec with values from windowed vec
    for (complex, &input_val) in fourier_chords.complex_buffer
        .iter_mut()
        .zip(fourier_chords.windowed_values.iter()) {
        *complex = Complex { re: input_val, im: 0.0 };
    }

    // Perform forward FFT on input data
    fourier_chords.fft_algorithm.process(&mut fourier_chords.complex_buffer);
}


// Utility function to apply the window function in place
fn apply_window_function(fourier_chords: &mut FourierChords) {
    for (i, (&sample, windowed_value)) in fourier_chords.sample_vec.iter().zip(fourier_chords.windowed_values.iter_mut()).enumerate() {
        let window_value = 0.5 - 0.5 * (2.0 * PI * i as f32 / (fourier_chords.window_size as f32 - 1.0)).cos();
        *windowed_value = sample * window_value;
    }
}

// Transforms buffer of complex numbers from FFT forward transform and returns vector of
// SpectrumData, which contains fields for frequencies and magnitudes
fn get_spectrum_data(fourier_chords: &mut FourierChords) -> () {
    fourier_chords.nyquist_limit = fourier_chords.sample_vec.len() / 2;
    fourier_chords.frequency_resolution = fourier_chords.sample_rate / fourier_chords.sample_vec.len() as f32;

    for (i, spectrum_data) in fourier_chords.spectrum_data
        .iter_mut()
        .enumerate()
        .take(fourier_chords.nyquist_limit) {

        let frequency = i as f32 * fourier_chords.frequency_resolution;
        let magnitude = fourier_chords.complex_buffer[i].norm();

        spectrum_data.frequency = frequency;
        spectrum_data.magnitude = magnitude;
        spectrum_data.index = i;
    }
}

// Function to identify notes in the spectrum
fn identify_notes(fourier_chords: &mut FourierChords) -> () {
    // Calculate local maxima of given spectrum data
    get_local_maxima(fourier_chords);

    // TODO: Finish peak picking algorithm process
    get_prominent_peaks(fourier_chords);

    // Initializes frequency to note hash map
    let freq_to_note = get_note_data();

    for value in &fourier_chords.prominent_peaks {
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

        if !fourier_chords.detected_notes.contains(&note.to_string()) {
            fourier_chords.detected_notes.push(note.clone()); // Clone the note string and add it to matched_notes
        }
    }
}

// TODO: Consider adding these as FFT Chord implementation functions; seems like that would be more
//  straightforward based on how many functions we're defining that depend on traits
// Calculate local maxima
fn get_local_maxima(fourier_chords: &mut FourierChords) -> () {
    // Calculate magnitude threshold and assign it. Play with this value to optimize execution time.
    // Currently, we're using a threshold of one third of the maximum magnitude.
    fourier_chords.magnitude_threshold = max_magnitude(&fourier_chords.spectrum_data) / 3.0;

    // Identifies local maxima and pushes them to maxima vector
    for i in 1..fourier_chords.spectrum_data.len() - 1 {
        if fourier_chords.spectrum_data[i].magnitude < fourier_chords.magnitude_threshold {
            continue;
        }
        if fourier_chords.spectrum_data[i].magnitude > fourier_chords.spectrum_data[i - 1].magnitude
            && fourier_chords.spectrum_data[i].magnitude > fourier_chords.spectrum_data[i + 1].magnitude
        {
            fourier_chords.local_maxima.push(fourier_chords.spectrum_data[i].clone());
        }
    }
}

// Identifies maximum magnitude from SpectrumData vector
// TODO: Error handling
fn max_magnitude(spectrum: &Vec<SpectrumData>) -> f32 {
    spectrum.iter()
        .map(|data| data.magnitude) // Extract the magnitude from each SpectrumData
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)) // Compare magnitudes
        .unwrap_or(10000.0) // Handle the case where spectrum_data is empty
}

// TODO: Implement prominent peak picking algorithm
fn get_prominent_peaks(fourier_chords: &mut FourierChords) -> () {
    fourier_chords.prominence_threshold = max_magnitude(&fourier_chords.local_maxima) / 4.0;

    for peak in &fourier_chords.local_maxima {
        fourier_chords.prominence = calculate_prominence(&fourier_chords.spectrum_data, peak.index);
        if fourier_chords.prominence >= fourier_chords.prominence_threshold {
            fourier_chords.prominent_peaks.push(peak.clone());
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
