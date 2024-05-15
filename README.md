# Fourier Chords

Fourier Chords is a VST plugin that analyzes incoming audio, performs Fast Fourier Transform (FFT) on the signal, and visualizes the notes and chords contained within the signal to the user.

## Status
Fourier Chords is still under development, and there is still much to do. In its current iteration, the code can be compiled as an audio plugin and added to an audio track in a DAW. When audio passes through the plugin, it will show the notes that it hears in the incoming audio. For example, if the audio is composed of a Cmaj chord played on a piano, the plugin will identify notes C-E-G, which compose the Cmaj chord. 

The detection is still spotty, and there are many false positives, especially on layered, atmospheric sounds. In the future, I aim to:
- Visualize real-time audio frequencies
- Improve accuracy
- Reduce latency

## Features

- Real-time FFT analysis of incoming audio signal
- Note and chord detection
- Visual representation of notes and chords
- VST3 plugin compatible with various Digital Audio Workstations (DAWs)

## Getting Started

### Prerequisites

- A DAW that supports VST3 plugins
- Rust and Cargo for building the project

## Building
After installing [Rust](https://rustup.rs/), you can compile fourierchords as follows:
```shell
cargo xtask bundle fourierchords --release
```

### Usage
Build the release version of the project. 

Load the Fourier Chords plugin in your DAW, and route audio to it. The plugin will analyze the audio and display detected notes in real-time.

### Contributing
If you would like to contribute to the project, feel free to fork the repository, create a new branch for your work, and open a pull request.

### License
This project is licensed under the MIT License.
