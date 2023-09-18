use alsa::pcm::IO;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia_core::audio::{AudioBufferRef, Signal};

use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, ValueOr};

fn process_packets(mss: MediaSourceStream, io: &IO<'_, i16>) {
    // Create the media source stream.

    let mut hint = Hint::new();
    hint.with_extension("ogg");

    // Use the default options for metadata and format readers.
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    // Probe the media source.
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .expect("unsupported format");

    // Get the instantiated format reader.
    let mut format = probed.format;

    // Find the first audio track with a known (decodeable) codec.
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .expect("no supported audio tracks");

    // Use the default options for the decoder.
    let dec_opts: DecoderOptions = Default::default();

    // Create a decoder for the track.
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .expect("unsupported codec");

    // Store the track identifier, it will be used to filter packets.
    let track_id = track.id;
    loop {
        // Get the next packet from the media format.
        let res = format.next_packet();
        if res.is_err() {
            println!("Was err! {:?}", res.err());
            break;
        }
        let packet = format.next_packet().unwrap();

        // Consume any new metadata that has been read since the last packet.
        while !format.metadata().is_latest() {
            // Pop the old head of the metadata queue.
            format.metadata().pop();
            // Consume the new metadata at the head of the metadata queue.
        }

        // If the packet does not belong to the selected track, skip over it.
        if packet.track_id() != track_id {
            continue;
        }

        // Decode the packet into audio samples.
        match decoder.decode(&packet) {
            Ok(_decoded) => {
                // Consume the decoded audio samples (see below).
                match _decoded {
                    AudioBufferRef::F32(buf) => {
                        let chan = buf.chan(0);
                        play_pcm_reader(&io, chan);
                    }
                    _ => {
                        // Repeat for the different sample formats.
                        unimplemented!()
                    }
                }
            }
            Err(Error::IoError(_)) => {
                // The packet failed to decode due to an IO error, skip the packet.
                continue;
            }
            Err(Error::DecodeError(_)) => {
                // The packet failed to decode due to invalid data, skip the packet.
                continue;
            }
            Err(err) => {
                // An unrecoverable error occurred, halt decoding.
                panic!("{:?}", err);
            }
        }
    }
}

fn open_audio_device() -> PCM {
    // Open default playback device
    let pcm = PCM::new("default", Direction::Playback, false).unwrap();

    // Set hardware parameters: 16000 Hz / Mono / 16 bit
    {
        let hwp = HwParams::any(&pcm).unwrap();
        hwp.set_channels(1).unwrap();
        hwp.set_rate(16000, ValueOr::Nearest).unwrap();
        hwp.set_format(Format::s16()).unwrap();
        hwp.set_access(Access::RWInterleaved).unwrap();
        pcm.hw_params(&hwp).unwrap();
    }

    {
        let hwp = pcm.hw_params_current().unwrap();
        let swp = pcm.sw_params_current().unwrap();
        swp.set_start_threshold(hwp.get_buffer_size().unwrap())
            .unwrap();
        pcm.sw_params(&swp).unwrap();
    }
    pcm
}
fn main() {
    let src = std::fs::File::open("./valkyries.ogg").expect("failed to open media");
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let pcm = open_audio_device();
    let io = pcm.io_i16().unwrap();
    process_packets(mss, &io);
    pcm.drain().unwrap();
}

fn play_pcm_reader(io: &IO<'_, i16>, reader: &[f32]) {
    //let io = pcm.io_f32().unwrap();

    let mut converted: Vec<i16> = Vec::new();
    for sample in reader {
        let isample: i32 = (sample * 65535.0) as i32;
        // this is incorrect, but not sure what's a good way of converting F32->S16
        converted.push(isample as i16);
        converted.push(isample as i16);
        //
        //converted.push(((isample & 0x7fff0000) >> 16) as i16);
        //converted.push((isample & 0xffff) as i16);
        //converted.push((isample & 0xffff) as i16);
    }
    io.writei(&converted).unwrap();
}
