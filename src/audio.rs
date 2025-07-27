use cpal::traits::{HostTrait, StreamTrait, DeviceTrait};
use std::{error::Error, fs, thread::spawn, sync::{Arc, mpsc::{self, Sender}, Mutex}};
use symphonia::core::{codecs::DecoderOptions, formats::FormatOptions, io::MediaSourceStream, probe::Hint, audio::Signal};
use std::sync::atomic::{AtomicU32, Ordering};

enum PlayerCommand {
    Play(AudioData),
    SetVolume(u32),
    Stop,
}

#[derive(Clone)]
struct AudioData {
    data: Vec<f32>,
    sample_rate: u32,
    channels: u16,
    position: f64,
}

struct AudioPlayer {
    host: cpal::Host,
    device: cpal::Device,
    volume: Arc<AtomicU32>,
    audio: Arc<Mutex<Option<AudioData>>>,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device available");
        AudioPlayer {
            host,
            device,
            volume: Arc::new(AtomicU32::new(0_500)),
            audio: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for AudioData {
    fn default() -> Self {
        AudioData {
            data: Vec::new(),
            sample_rate: 44100,
            channels: 2,
            position: 0.0,
        }
    }
}

impl AudioPlayer {
    fn new() -> Self {
        AudioPlayer::default()
    }

    fn start(&mut self) -> Sender<PlayerCommand> {
        let (tx, rx) = mpsc::channel();
        let volume = Arc::clone(&self.volume); // Клонируем Arc для потока
        let audio = Arc::clone(&self.audio);
        let device = self.device.clone();

        spawn(move || {
            let mut stream: Option<cpal::Stream> = None;
            for command in rx {
                match command {
                    PlayerCommand::Play(audio_data) => {
                        *audio.lock().unwrap() = Some(audio_data.clone());
                        let mut audio_data = audio_data;
                        let volume_clone = Arc::clone(&volume); // Клонируем Arc для замыкания

                        let mut supported_configs = device.supported_output_configs().unwrap();
                        let config_range = supported_configs
                            .find(|c| c.sample_format() == cpal::SampleFormat::F32)
                            .unwrap();
                        let config = config_range.with_max_sample_rate();

                        let stream_config = cpal::StreamConfig {
                            channels: config.channels(),
                            sample_rate: config.sample_rate(),
                            buffer_size: cpal::BufferSize::Default,
                        };

                        let new_stream = device
                            .build_output_stream(&stream_config,  move |data: &mut [f32], _| {
                                for sample in data.iter_mut() {
                                    if audio_data.position < audio_data.data.len() as f64 {
                                        let idx = audio_data.position as usize;
                                        let vol = volume_clone.load(Ordering::Relaxed) as f32 / 1_000.0;
                                        *sample = audio_data.data[idx] * vol;
                                        audio_data.position += 0.5;
                                    } else {
                                        *sample = 0.0;
                                    }
                                }
                            }, |err| eprintln!("Error occurred on stream: {}", err), None)
                            .unwrap();

                        let _ = new_stream.play();
                        stream = Some(new_stream);
                    }
                    PlayerCommand::SetVolume(value) => {
                        volume.store(value, Ordering::Relaxed);
                    }
                    PlayerCommand::Stop => {
                        if let Some(s) = stream.take() {
                            let _ = s.pause();
                        }
                        break;
                    }
                }
            }
        });

        tx
    }

    fn byte_array_conversion(&self, file: &[u8]) -> Result<AudioData, Box<dyn Error>> {
        let buffer = file.to_vec();
        let src = std::io::Cursor::new(buffer);
        let mss = MediaSourceStream::new(Box::new(src), Default::default());
        let hint = Hint::new();
        let probed = symphonia::default::get_probe().format(&hint, mss, &FormatOptions::default(), &Default::default())?;
        
        let mut reader = probed.format;
        let track = reader.default_track().ok_or("No track found")?;
        let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

        let mut samples = Vec::new();
        let mut sample_rate = 0;
        let mut channels = 0;

        loop {
            match reader.next_packet() {
                Ok(packet) => {
                    let decoded = decoder.decode(&packet)?;
                    if let symphonia::core::audio::AudioBufferRef::F32(buf) = decoded {
                        samples.extend_from_slice(buf.chan(0));
                        sample_rate = buf.spec().rate;
                        channels = buf.spec().channels.count() as u16;
                    }
                }
                Err(symphonia::core::errors::Error::IoError(_)) => break,
                Err(e) => return Err(Box::new(e)),
            }
        }

        Ok(AudioData {
            data: samples,
            sample_rate,
            channels,
            ..Default::default()
        })
    }

    fn set_volume(&self, value: u32) {
        self.volume.store(value, Ordering::Relaxed);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = "C:/Users/shulg/Downloads/test.mp3";
    let file_data = fs::read(path)?;
    let mut audio_player = AudioPlayer::new();
    let tx = audio_player.start();
    
    let data = audio_player.byte_array_conversion(&file_data)?;
    tx.send(PlayerCommand::Play(data))?;

    std::thread::sleep(std::time::Duration::from_secs(2));
    tx.send(PlayerCommand::SetVolume(100))?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    tx.send(PlayerCommand::SetVolume(300))?;


    Ok(())
}