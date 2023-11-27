use nih_plug::prelude::*;

pub struct CircleBuffer {
    sample_rate: f32,
    num_samples: usize,
    buffer: Vec<f32>,
    write_pos: usize,
}

impl CircleBuffer {
    pub fn new() -> Self {
        let buffer = Vec::new();
        let sample_rate = 1.0;
        CircleBuffer {
            sample_rate,
            num_samples: 0,
            buffer,
            write_pos: 0,
        }
    }

    pub fn resize(&mut self, sample_rate: f32, capacity_factor: usize) {
        nih_debug_assert!(sample_rate > 0.0);

        self.sample_rate = sample_rate;
        self.num_samples = ((sample_rate * capacity_factor as f32).ceil()) as usize;

        self.buffer.resize(self.num_samples, 0.0);
    }

    pub fn samples(&self) -> usize {
        self.num_samples
    }

    pub fn write(&mut self, sample: f32) {
        self.buffer[self.write_pos] = sample;
        self.write_pos += 1 as usize;
        self.write_pos %= self.num_samples;
    }

    pub fn read(&self, read_offset: usize) -> f32 {
        let read_pos = self._get_read_pos(read_offset);
        return self.buffer[read_pos];
    }

    fn _get_read_pos(&self, read_offset: usize) -> usize {
        let read_offset = if read_offset >= self.num_samples {
            dbg!("read position would loop the buffer, so is being capped to its max value, read_offset: {}, buffer_length: {}", read_offset, self.num_samples);
            0
        } else if read_offset < 0 as usize {
            dbg!("read position would loop the buffer, so is being capped to its min value, read_offset: {}, buffer_length: {}", read_offset, self.num_samples);
            self.num_samples
        } else {
            self.num_samples - read_offset
        };

        (self.write_pos + (self.num_samples - 1) + read_offset) % self.num_samples
    }
}
