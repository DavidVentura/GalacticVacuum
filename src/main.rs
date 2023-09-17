use itertools::Itertools;

use alsa::pcm::{Access, Format, HwParams, State, PCM};
use alsa::{Direction, ValueOr};

fn main() {
    let mut reader = hound::WavReader::open("./valkyries.wav").unwrap();
    // Open default playback device
    let pcm = PCM::new("default", Direction::Playback, false).unwrap();

    // Set hardware parameters: 44100 Hz / Mono / 16 bit
    let hwp = HwParams::any(&pcm).unwrap();
    hwp.set_channels(1).unwrap();
    hwp.set_rate(16000, ValueOr::Nearest).unwrap();
    hwp.set_format(Format::s16()).unwrap();
    hwp.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&hwp).unwrap();
    let io = pcm.io_i16().unwrap();

    // Make sure we don't start the stream too early
    let hwp = pcm.hw_params_current().unwrap();
    let swp = pcm.sw_params_current().unwrap();
    swp.set_start_threshold(hwp.get_buffer_size().unwrap())
        .unwrap();
    pcm.sw_params(&swp).unwrap();

    // Make a sine wave
    for _samples in reader.samples::<i16>().chunks(4096).into_iter() {
        let s: Result<Vec<i16>, _> = _samples.collect();
        io.writei(&s.unwrap()).unwrap();
    }

    // In case the buffer was larger than 2 seconds, start the stream manually.
    if pcm.state() != State::Running {
        pcm.start().unwrap()
    };
    // Wait for the stream to finish playback.
    pcm.drain().unwrap();
}
