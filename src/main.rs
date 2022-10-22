use std::f32::consts::{ PI, TAU };
use std::io::Result;
use std::path::Path;

use wave_stream::{read_wav_from_file_path, write_wav_to_file_path};
use wave_stream::open_wav::OpenWav;
use wave_stream::wave_header::{SampleFormat, WavHeader};
use wave_stream::wave_reader::{RandomAccessOpenWavReader, StreamOpenWavReader};

fn main() {
    let open_wav = read_wav_from_file_path(Path::new("some.wav")).unwrap();

    // Inspect metadata
    // ******************************
    println!("Number of channels: {0}, samples per second: {1}, bits per sample: {2}, length in samples: {3}",
        open_wav.channels(),
        open_wav.bits_per_sample(),
        open_wav.sample_rate(),
        open_wav.len_samples());

    // Read via random access
    // ******************************
    let mut random_access_wave_reader = open_wav.get_random_access_f32_reader().unwrap();
    let first_sample = random_access_wave_reader.read_sample(0, 0).unwrap();
    println!("First sample, channel 0: {0}", first_sample);

    // Read via an enumerable: Find the loudest sample in the wave file
    // ******************************
    let open_wav = read_wav_from_file_path(Path::new("some.wav")).unwrap();
    let mut loudest_sample = f32::MIN;

    // Note that the wave is read as f32 values in this example.
    // Reading as 8-bit, (i8) 16-bit, (i16) and 24-bit (i32) is also supported.
    // Upsampling during reads is supported. (You can read an 8-bit wav file as f32)
    // Downsampling during reads isn't supported. (You can't read a floating point wav file as i8)
    //
    // In general:
    // - For audio manipulation: f32 is *strongly* reccomended
    // - Only use i16, i32, (or i8), when cutting existing audio without manipulation
    let iterator = open_wav.get_stream_f32_reader().unwrap().into_iter();

    for samples_result in iterator {
        let samples = samples_result.unwrap();

        for sample in samples {
            loudest_sample = f32::max(loudest_sample, sample);
        }
    }

    println!("Loudest sample: {0}", loudest_sample);

    // Write via random access
    // ******************************
    let sample_rate = 96000;
    let header = WavHeader {
        sample_format: SampleFormat::Float,
        channels: 1,
        sample_rate,
    };

    let open_wav = write_wav_to_file_path(Path::new("ramp.wav"), header).unwrap();
    let mut random_access_wave_writer = open_wav.get_random_access_f32_writer().unwrap();

    let samples_in_ramp = 2000;
    let samples_in_ramp_f32 = samples_in_ramp as f32;
    for sample in 0u32..(sample_rate * 3u32) { // Write 3 seconds of samples
        let modulo = (sample % samples_in_ramp) as f32;
        let sample_value = (2f32 * modulo / samples_in_ramp_f32) - 1f32;
        random_access_wave_writer.write_sample(sample, 0, sample_value).unwrap();
    }

    random_access_wave_writer.flush().unwrap();

    // Write via iterator
    // ******************************
    let header = WavHeader {
        sample_format: SampleFormat::Float,
        channels: 1,
        sample_rate,
    };

    let open_wav = write_wav_to_file_path(Path::new("sine.wav"), header).unwrap();
    let sine_iterator = SineIterator {
        period: (sample_rate / 60) as f32,
        current_sample: PI // Start at 0 crossing
    };
    let sine_iterator_three_seconds = sine_iterator.take((sample_rate * 3u32) as usize); // Write 3 seconds
    open_wav.write_all_f32(sine_iterator_three_seconds).unwrap();
}

// Used when writing via iterator
struct SineIterator {
    period: f32,
    current_sample: f32
}

// Used when writing via iterator
impl Iterator for SineIterator {
    type Item = Result<Vec<f32>>;

    fn next(&mut self) -> Option<Result<Vec<f32>>> {
        let result = (self.current_sample / self.period * TAU).sin();
        self.current_sample += 1f32;

        if self.current_sample > self.period {
            self.current_sample = 0f32;
        }

        return Some(Ok(vec![result]));
    }
}
