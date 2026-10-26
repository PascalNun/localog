//! A WAV file that is valid while it is still being written.
//!
//! The header carries two lengths that are only known when a recording ends, so
//! the usual arrangement is to patch them on close. This application cannot rely
//! on getting to close: `recording.rs` stops a recorder politely where it can and
//! kills it where it cannot, and on Windows there is no polite version — a stop
//! is `TerminateProcess`, with nothing running afterwards.
//!
//! So the lengths are rewritten every time samples are flushed. A file killed
//! mid-recording is then a complete WAV of everything up to the last flush,
//! rather than a header claiming zero bytes and a body nobody will play.

use std::fs::File;
use std::io::{BufWriter, Result, Seek, SeekFrom, Write};
use std::path::Path;

pub struct Wav {
    file: BufWriter<File>,
    data_bytes: u32,
}

impl Wav {
    pub fn create(path: &Path, sample_rate: u32, channels: u16) -> Result<Self> {
        let mut file = BufWriter::new(File::create(path)?);
        let bits = 16_u16;
        let block_align = channels * bits / 8;
        let byte_rate = sample_rate * u32::from(block_align);
        file.write_all(b"RIFF")?;
        file.write_all(&0_u32.to_le_bytes())?; // patched by `flush`
        file.write_all(b"WAVEfmt ")?;
        file.write_all(&16_u32.to_le_bytes())?;
        file.write_all(&1_u16.to_le_bytes())?; // PCM
        file.write_all(&channels.to_le_bytes())?;
        file.write_all(&sample_rate.to_le_bytes())?;
        file.write_all(&byte_rate.to_le_bytes())?;
        file.write_all(&block_align.to_le_bytes())?;
        file.write_all(&bits.to_le_bytes())?;
        file.write_all(b"data")?;
        file.write_all(&0_u32.to_le_bytes())?; // patched by `flush`
        Ok(Self {
            file,
            data_bytes: 0,
        })
    }

    pub fn write(&mut self, samples: &[i16]) -> Result<()> {
        for sample in samples {
            self.file.write_all(&sample.to_le_bytes())?;
        }
        self.data_bytes = self.data_bytes.saturating_add((samples.len() * 2) as u32);
        Ok(())
    }

    /// Push the samples to disk and correct the two lengths in the header.
    pub fn flush(&mut self) -> Result<()> {
        self.file.flush()?;
        let data = self.data_bytes;
        let inner = self.file.get_mut();
        inner.seek(SeekFrom::Start(4))?;
        inner.write_all(&(36 + data).to_le_bytes())?;
        inner.seek(SeekFrom::Start(40))?;
        inner.write_all(&data.to_le_bytes())?;
        inner.seek(SeekFrom::End(0))?;
        inner.flush()
    }

    pub fn bytes_written(&self) -> u32 {
        self.data_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_checkpoint_is_playable_and_later_samples_append_after_the_audio() {
        let directory = std::env::temp_dir().join(format!(
            "localog-wav-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("synthetic.wav");
        let mut wav = Wav::create(&path, 48_000, 2).unwrap();
        wav.write(&[123, -123]).unwrap();
        wav.flush().unwrap();
        let checkpoint = std::fs::read(&path).unwrap();
        assert_eq!(&checkpoint[..4], b"RIFF");
        assert_eq!(&checkpoint[8..16], b"WAVEfmt ");
        assert_eq!(u32::from_le_bytes(checkpoint[4..8].try_into().unwrap()), 40);
        assert_eq!(
            u32::from_le_bytes(checkpoint[40..44].try_into().unwrap()),
            4
        );
        assert_eq!(checkpoint.len(), 48);

        wav.write(&[456, -456]).unwrap();
        wav.flush().unwrap();
        let complete = std::fs::read(&path).unwrap();
        assert_eq!(u32::from_le_bytes(complete[4..8].try_into().unwrap()), 44);
        assert_eq!(u32::from_le_bytes(complete[40..44].try_into().unwrap()), 8);
        assert_eq!(&complete[44..48], &checkpoint[44..48]);
        assert_eq!(&complete[48..50], &456_i16.to_le_bytes());
        assert_eq!(&complete[50..52], &(-456_i16).to_le_bytes());
        drop(wav);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
