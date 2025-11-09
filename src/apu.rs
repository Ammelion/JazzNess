use std::collections::VecDeque;
use serde::{Serialize, Deserialize};

const CPU_CLOCK_HZ: f64 = 1_789_773.0;
const AUDIO_SAMPLE_RATE: f64 = 44100.0;
const CYCLES_PER_SAMPLE: f64 = CPU_CLOCK_HZ / AUDIO_SAMPLE_RATE;

const LENGTH_COUNTER_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const PULSE_DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

const TRIANGLE_WAVE_TABLE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
    12, 13, 14, 15,
];

const NOISE_PERIOD_TABLE: [u16; 16] =
    [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068];

#[derive(Default)]
struct Envelope {
    start: bool,
    loop_flag: bool,
    enabled: bool,
    period: u8,
    decay_level: u8,
    divider: u8,
    volume: u8,
}

#[derive(Serialize, Deserialize, Default)]
pub struct EnvelopeState {
    start: bool,
    loop_flag: bool,
    enabled: bool,
    period: u8,
    decay_level: u8,
    divider: u8,
    volume: u8,
}

impl Envelope {
    fn write(&mut self, data: u8) {
        self.loop_flag = (data & 0x20) != 0;
        self.enabled = (data & 0x10) == 0;
        self.period = data & 0x0F;
        self.volume = self.period;
        self.start = true;
    }

    fn clock(&mut self) {
        if self.start {
            self.start = false;
            self.decay_level = 15;
            self.divider = self.period;
        } else {
            if self.divider > 0 {
                self.divider -= 1;
            } else {
                self.divider = self.period;
                if self.decay_level > 0 {
                    self.decay_level -= 1;
                } else if self.loop_flag {
                    self.decay_level = 15;
                }
            }
        }
    }

    fn output(&self) -> u8 {
        if self.enabled {
            self.decay_level
        } else {
            self.volume
        }
    }

    fn save_state(&self) -> EnvelopeState {
        EnvelopeState {
            start: self.start,
            loop_flag: self.loop_flag,
            enabled: self.enabled,
            period: self.period,
            decay_level: self.decay_level,
            divider: self.divider,
            volume: self.volume,
        }
    }

    fn load_state(&mut self, state: &EnvelopeState) {
        self.start = state.start;
        self.loop_flag = state.loop_flag;
        self.enabled = state.enabled;
        self.period = state.period;
        self.decay_level = state.decay_level;
        self.divider = state.divider;
        self.volume = state.volume;
    }
}

#[derive(Default)]
struct Sweep {
    enabled: bool,
    negate: bool,
    reload: bool,
    period: u8,
    divider: u8,
    shift: u8,
}

#[derive(Serialize, Deserialize, Default)]
pub struct SweepState {
    enabled: bool,
    negate: bool,
    reload: bool,
    period: u8,
    divider: u8,
    shift: u8,
}

impl Sweep {
    fn write(&mut self, data: u8) {
        self.enabled = (data & 0x80) != 0;
        self.period = (data & 0x70) >> 4;
        self.negate = (data & 0x08) != 0;
        self.shift = data & 0x07;
        self.reload = true;
    }

    fn clock(&mut self, timer_period: &mut u16, channel_num: u8) -> bool {
        let mut muted = false;
        let change = *timer_period >> self.shift;

        if self.negate {
            let subtract_amount = change + if channel_num == 1 { 1 } else { 0 };
            if *timer_period < subtract_amount {
                muted = true;
            }
        }

        let target_period = if self.negate {
            timer_period.wrapping_sub(change + if channel_num == 1 { 1 } else { 0 })
        } else {
            *timer_period + change
        };

        if *timer_period < 8 || target_period > 0x7FF {
            muted = true;
        }

        if self.divider == 0 && self.enabled && self.shift > 0 && !muted {
            *timer_period = target_period;
        }

        if self.divider == 0 || self.reload {
            self.divider = self.period;
            self.reload = false;
        } else {
            self.divider -= 1;
        }
        muted
    }

    fn save_state(&self) -> SweepState {
        SweepState {
            enabled: self.enabled,
            negate: self.negate,
            reload: self.reload,
            period: self.period,
            divider: self.divider,
            shift: self.shift,
        }
    }

    fn load_state(&mut self, state: &SweepState) {
        self.enabled = state.enabled;
        self.negate = state.negate;
        self.reload = state.reload;
        self.period = state.period;
        self.divider = state.divider;
        self.shift = state.shift;
    }
}

#[derive(Default)]
struct Pulse {
    enabled: bool,
    envelope: Envelope,
    sweep: Sweep,
    duty_mode: u8,
    duty_step: u8,
    timer_period: u16,
    timer_value: u16,
    length_counter: u8,
    length_counter_halt: bool,
    sweep_muted: bool,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PulseState {
    enabled: bool,
    envelope: EnvelopeState,
    sweep: SweepState,
    duty_mode: u8,
    duty_step: u8,
    timer_period: u16,
    timer_value: u16,
    length_counter: u8,
    length_counter_halt: bool,
    sweep_muted: bool,
}

impl Pulse {
    fn new() -> Self {
        Self::default()
    }

    fn clock_timer(&mut self) {
        if self.timer_value > 0 {
            self.timer_value -= 1;
        } else {
            self.timer_value = self.timer_period;
            self.duty_step = (self.duty_step + 1) % 8;
        }
    }

    fn clock_length_counter(&mut self) {
        if !self.length_counter_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    fn clock_sweep(&mut self, channel_num: u8) {
        self.sweep_muted = self.sweep.clock(&mut self.timer_period, channel_num);
    }

    fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    fn output(&self) -> u8 {
        if !self.enabled
            || self.length_counter == 0
            || PULSE_DUTY_TABLE[self.duty_mode as usize][self.duty_step as usize] == 0
            || self.timer_period < 8
            || self.sweep_muted
        {
            0
        } else {
            self.envelope.output()
        }
    }

    fn write_ctrl(&mut self, data: u8) {
        self.duty_mode = (data & 0xC0) >> 6;
        self.length_counter_halt = (data & 0x20) != 0;
        self.envelope.write(data);
    }

    fn write_sweep(&mut self, data: u8) {
        self.sweep.write(data);
    }

    fn write_timer_lo(&mut self, data: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | (data as u16);
    }

    fn write_timer_hi(&mut self, data: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | (((data & 0x07) as u16) << 8);
        if self.enabled {
            self.length_counter = LENGTH_COUNTER_TABLE[(data >> 3) as usize];
        }
        self.timer_value = self.timer_period;
        self.envelope.start = true;
        self.duty_step = 0;
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }

    fn save_state(&self) -> PulseState {
        PulseState {
            enabled: self.enabled,
            envelope: self.envelope.save_state(),
            sweep: self.sweep.save_state(),
            duty_mode: self.duty_mode,
            duty_step: self.duty_step,
            timer_period: self.timer_period,
            timer_value: self.timer_value,
            length_counter: self.length_counter,
            length_counter_halt: self.length_counter_halt,
            sweep_muted: self.sweep_muted,
        }
    }

    fn load_state(&mut self, state: &PulseState) {
        self.enabled = state.enabled;
        self.envelope.load_state(&state.envelope);
        self.sweep.load_state(&state.sweep);
        self.duty_mode = state.duty_mode;
        self.duty_step = state.duty_step;
        self.timer_period = state.timer_period;
        self.timer_value = state.timer_value;
        self.length_counter = state.length_counter;
        self.length_counter_halt = state.length_counter_halt;
        self.sweep_muted = state.sweep_muted;
    }
}

#[derive(Default)]
struct Triangle {
    enabled: bool,
    timer_period: u16,
    timer_value: u16,
    duty_step: u8,
    length_counter: u8,
    length_counter_halt: bool,
    linear_counter: u8,
    linear_counter_period: u8,
    linear_counter_reload: bool,
}

#[derive(Serialize, Deserialize, Default)]
pub struct TriangleState {
    enabled: bool,
    timer_period: u16,
    timer_value: u16,
    duty_step: u8,
    length_counter: u8,
    length_counter_halt: bool,
    linear_counter: u8,
    linear_counter_period: u8,
    linear_counter_reload: bool,
}

impl Triangle {
    fn new() -> Self {
        Self::default()
    }

    fn clock_timer(&mut self) {
        if self.timer_value > 0 {
            self.timer_value -= 1;
        } else {
            self.timer_value = self.timer_period;
            if self.length_counter > 0 && self.linear_counter > 0 && self.timer_period > 1 {
                self.duty_step = (self.duty_step + 1) % 32;
            }
        }
    }

    fn clock_length_counter(&mut self) {
        if !self.length_counter_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    fn clock_linear_counter(&mut self) {
        if self.linear_counter_reload {
            self.linear_counter = self.linear_counter_period;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }
        if !self.length_counter_halt {
            self.linear_counter_reload = false;
        }
    }

    fn output(&self) -> u8 {
        if !self.enabled || self.length_counter == 0 || self.linear_counter == 0 {
            return 0;
        }
        TRIANGLE_WAVE_TABLE[self.duty_step as usize]
    }

    fn write_ctrl(&mut self, data: u8) {
        self.length_counter_halt = (data & 0x80) != 0;
        self.linear_counter_period = data & 0x7F;
    }

    fn write_timer_lo(&mut self, data: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | (data as u16);
    }

    fn write_timer_hi(&mut self, data: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | (((data & 0x07) as u16) << 8);
        if self.enabled {
            self.length_counter = LENGTH_COUNTER_TABLE[(data >> 3) as usize];
        }
        self.linear_counter_reload = true;
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }

    fn save_state(&self) -> TriangleState {
        TriangleState {
            enabled: self.enabled,
            timer_period: self.timer_period,
            timer_value: self.timer_value,
            duty_step: self.duty_step,
            length_counter: self.length_counter,
            length_counter_halt: self.length_counter_halt,
            linear_counter: self.linear_counter,
            linear_counter_period: self.linear_counter_period,
            linear_counter_reload: self.linear_counter_reload,
        }
    }

    fn load_state(&mut self, state: &TriangleState) {
        self.enabled = state.enabled;
        self.timer_period = state.timer_period;
        self.timer_value = state.timer_value;
        self.duty_step = state.duty_step;
        self.length_counter = state.length_counter;
        self.length_counter_halt = state.length_counter_halt;
        self.linear_counter = state.linear_counter;
        self.linear_counter_period = state.linear_counter_period;
        self.linear_counter_reload = state.linear_counter_reload;
    }
}

#[derive(Default)]
struct Noise {
    enabled: bool,
    envelope: Envelope,
    timer_period: u16,
    timer_value: u16,
    length_counter: u8,
    length_counter_halt: bool,
    mode: bool,
    shift_register: u16,
}

#[derive(Serialize, Deserialize, Default)]
pub struct NoiseState {
    enabled: bool,
    envelope: EnvelopeState,
    timer_period: u16,
    timer_value: u16,
    length_counter: u8,
    length_counter_halt: bool,
    mode: bool,
    shift_register: u16,
}

impl Noise {
    fn new() -> Self {
        let mut noise = Self::default();
        noise.shift_register = 1;
        noise
    }

    fn clock_timer(&mut self) {
        if self.timer_value > 0 {
            self.timer_value -= 1;
        } else {
            self.timer_value = self.timer_period;
            let bit0 = self.shift_register & 1;
            let bit_cmp = if self.mode {
                (self.shift_register >> 6) & 1
            } else {
                (self.shift_register >> 1) & 1
            };
            let feedback = bit0 ^ bit_cmp;
            self.shift_register >>= 1;
            self.shift_register |= feedback << 14;
        }
    }

    fn clock_length_counter(&mut self) {
        if !self.length_counter_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    fn output(&self) -> u8 {
        if !self.enabled || self.length_counter == 0 || (self.shift_register & 1) == 1 {
            0
        } else {
            self.envelope.output()
        }
    }

    fn write_ctrl(&mut self, data: u8) {
        self.length_counter_halt = (data & 0x20) != 0;
        self.envelope.write(data);
    }

    fn write_period(&mut self, data: u8) {
        self.mode = (data & 0x80) != 0;
        self.timer_period = NOISE_PERIOD_TABLE[(data & 0x0F) as usize];
    }

    fn write_length(&mut self, data: u8) {
        if self.enabled {
            self.length_counter = LENGTH_COUNTER_TABLE[(data >> 3) as usize];
        }
        self.envelope.start = true;
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }

    fn save_state(&self) -> NoiseState {
        NoiseState {
            enabled: self.enabled,
            envelope: self.envelope.save_state(),
            timer_period: self.timer_period,
            timer_value: self.timer_value,
            length_counter: self.length_counter,
            length_counter_halt: self.length_counter_halt,
            mode: self.mode,
            shift_register: self.shift_register,
        }
    }

    fn load_state(&mut self, state: &NoiseState) {
        self.enabled = state.enabled;
        self.envelope.load_state(&state.envelope);
        self.timer_period = state.timer_period;
        self.timer_value = state.timer_value;
        self.length_counter = state.length_counter;
        self.length_counter_halt = state.length_counter_halt;
        self.mode = state.mode;
        self.shift_register = state.shift_register;
    }
}

#[derive(PartialEq, Copy, Clone)]
enum FrameCounterMode {
    Step4,
    Step5,
}

pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc_enabled: bool,
    sample_accumulator: f64,
    cpu_cycle_counter: u64,
    sample_buffer: VecDeque<f32>,
    last_input_sample: f32,
    last_output_sample: f32,
    frame_counter_cycle: u32,
    frame_counter_mode: FrameCounterMode,
    interrupt_inhibit: bool,
    frame_interrupt: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ApuState {
    pulse1: PulseState,
    pulse2: PulseState,
    triangle: TriangleState,
    noise: NoiseState,
    dmc_enabled: bool,
    sample_accumulator: f64,
    cpu_cycle_counter: u64,
    last_input_sample: f32,
    last_output_sample: f32,
    frame_counter_cycle: u32,
    frame_counter_mode: u8,
    interrupt_inhibit: bool,
    frame_interrupt: bool,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            pulse1: Pulse::new(),
            pulse2: Pulse::new(),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc_enabled: false,
            sample_accumulator: 0.0,
            last_input_sample: 0.0,
            last_output_sample: 0.0,
            cpu_cycle_counter: 0,
            sample_buffer: VecDeque::with_capacity(4096),
            frame_counter_cycle: 0,
            frame_counter_mode: FrameCounterMode::Step4,
            interrupt_inhibit: false,
            frame_interrupt: false,
        }
    }

    pub fn take_samples(&mut self) -> Vec<f32> {
        self.sample_buffer.drain(..).collect()
    }

    pub fn poll_frame_interrupt(&mut self) -> bool {
        let occurred = self.frame_interrupt;
        self.frame_interrupt = false;
        occurred
    }

    fn clock_frame_counter_step(&mut self) {
        const STEP1: u32 = 7457;
        const STEP2: u32 = 14913;
        const STEP3: u32 = 22371;
        const STEP4_4STEP: u32 = 29781;
        const STEP4_5STEP: u32 = 29781;
        const STEP5_5STEP: u32 = 37281;

        match self.frame_counter_mode {
            FrameCounterMode::Step4 => match self.frame_counter_cycle {
                STEP1 => self.clock_quarter_frame(),
                STEP2 => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
                STEP3 => self.clock_quarter_frame(),
                STEP4_4STEP => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                    if !self.interrupt_inhibit {
                        self.frame_interrupt = true;
                    }
                }
                _ => {}
            },
            FrameCounterMode::Step5 => match self.frame_counter_cycle {
                STEP1 => self.clock_quarter_frame(),
                STEP2 => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
                STEP3 => self.clock_quarter_frame(),
                STEP4_5STEP => {}
                STEP5_5STEP => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
                _ => {}
            },
        }
    }

    fn clock_quarter_frame(&mut self) {
        self.pulse1.clock_envelope();
        self.pulse2.clock_envelope();
        self.noise.clock_envelope();
        self.triangle.clock_linear_counter();
    }

    fn clock_half_frame(&mut self) {
        self.pulse1.clock_length_counter();
        self.pulse2.clock_length_counter();
        self.triangle.clock_length_counter();
        self.noise.clock_length_counter();
        self.pulse1.clock_sweep(1);
        self.pulse2.clock_sweep(2);
    }

    pub fn tick(&mut self, cpu_cycles: usize) {
        for _ in 0..cpu_cycles {
            self.cpu_cycle_counter += 1;

            if self.cpu_cycle_counter % 2 == 0 {
                self.pulse1.clock_timer();
                self.pulse2.clock_timer();
                self.noise.clock_timer();
            }
            self.triangle.clock_timer();

            self.clock_frame_counter_step();
            self.frame_counter_cycle += 1;

            let reset_cycle = match self.frame_counter_mode {
                FrameCounterMode::Step4 => 29781,
                FrameCounterMode::Step5 => 37282,
            };
            if self.frame_counter_cycle >= reset_cycle {
                self.frame_counter_cycle = 0;
                if self.frame_counter_mode == FrameCounterMode::Step5 {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
            }

            self.sample_accumulator += 1.0;
            while self.sample_accumulator >= CYCLES_PER_SAMPLE {
                self.sample_accumulator -= CYCLES_PER_SAMPLE;

                let pulse1_out = self.pulse1.output() as f32;
                let pulse2_out = self.pulse2.output() as f32;
                let triangle_out = self.triangle.output() as f32;
                let noise_out = self.noise.output() as f32;
                let dmc_out = 0.0;

                let pulse_mix = if pulse1_out == 0.0 && pulse2_out == 0.0 {
                    0.0
                } else {
                    95.88 / ((8128.0 / (pulse1_out + pulse2_out)) + 100.0)
                };
                let tnd_mix = if triangle_out == 0.0 && noise_out == 0.0 && dmc_out == 0.0 {
                    0.0
                } else {
                    159.79
                        / ((1.0
                            / (triangle_out / 8227.0 + noise_out / 12241.0 + dmc_out / 22638.0))
                            + 100.0)
                };
                let output_sample_raw = pulse_mix + tnd_mix;
                let output_sample_scaled = (output_sample_raw * 0.7) - 0.35;

                let alpha = 0.99;
                let filtered_output = alpha
                    * (self.last_output_sample + output_sample_scaled - self.last_input_sample);
                self.last_input_sample = output_sample_scaled;
                self.last_output_sample = filtered_output;

                self.sample_buffer.push_back(filtered_output);
            }
        }
    }

    pub fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x4015 => {
                let mut status = 0u8;
                if self.pulse1.length_counter > 0 {
                    status |= 0x01;
                }
                if self.pulse2.length_counter > 0 {
                    status |= 0x02;
                }
                if self.triangle.length_counter > 0 {
                    status |= 0x04;
                }
                if self.noise.length_counter > 0 {
                    status |= 0x08;
                }
                if self.dmc_enabled {
                    status |= 0x10;
                }
                if self.frame_interrupt {
                    status |= 0x40;
                }
                self.frame_interrupt = false;
                status
            }
            _ => 0,
        }
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000 => self.pulse1.write_ctrl(data),
            0x4001 => self.pulse1.write_sweep(data),
            0x4002 => self.pulse1.write_timer_lo(data),
            0x4003 => self.pulse1.write_timer_hi(data),
            0x4004 => self.pulse2.write_ctrl(data),
            0x4005 => self.pulse2.write_sweep(data),
            0x4006 => self.pulse2.write_timer_lo(data),
            0x4007 => self.pulse2.write_timer_hi(data),
            0x4008 => self.triangle.write_ctrl(data),
            0x4009 => {}
            0x400A => self.triangle.write_timer_lo(data),
            0x400B => self.triangle.write_timer_hi(data),
            0x400C => self.noise.write_ctrl(data),
            0x400D => {}
            0x400E => self.noise.write_period(data),
            0x400F => self.noise.write_length(data),
            0x4010 => {}
            0x4011 => {}
            0x4012 => {}
            0x4013 => {}
            0x4015 => {
                self.pulse1.set_enabled((data & 0x01) != 0);
                self.pulse2.set_enabled((data & 0x02) != 0);
                self.triangle.set_enabled((data & 0x04) != 0);
                self.noise.set_enabled((data & 0x08) != 0);
                self.dmc_enabled = (data & 0x10) != 0;
            }
            0x4017 => {
                self.frame_counter_mode = if (data & 0x80) != 0 {
                    FrameCounterMode::Step5
                } else {
                    FrameCounterMode::Step4
                };
                self.interrupt_inhibit = (data & 0x40) != 0;
                if self.interrupt_inhibit {
                    self.frame_interrupt = false;
                }

                self.frame_counter_cycle = 0;

                if self.frame_counter_mode == FrameCounterMode::Step5 {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
            }
            _ => {}
        }
    }

    pub fn save_state(&self) -> ApuState {
        ApuState {
            pulse1: self.pulse1.save_state(),
            pulse2: self.pulse2.save_state(),
            triangle: self.triangle.save_state(),
            noise: self.noise.save_state(),
            dmc_enabled: self.dmc_enabled,
            sample_accumulator: self.sample_accumulator,
            cpu_cycle_counter: self.cpu_cycle_counter,
            last_input_sample: self.last_input_sample,
            last_output_sample: self.last_output_sample,
            frame_counter_cycle: self.frame_counter_cycle,
            frame_counter_mode: match self.frame_counter_mode {
                FrameCounterMode::Step4 => 0,
                FrameCounterMode::Step5 => 1,
            },
            interrupt_inhibit: self.interrupt_inhibit,
            frame_interrupt: self.frame_interrupt,
        }
    }

    pub fn load_state(&mut self, state: &ApuState) {
        self.pulse1.load_state(&state.pulse1);
        self.pulse2.load_state(&state.pulse2);
        self.triangle.load_state(&state.triangle);
        self.noise.load_state(&state.noise);
        self.dmc_enabled = state.dmc_enabled;
        self.sample_accumulator = state.sample_accumulator;
        self.cpu_cycle_counter = state.cpu_cycle_counter;
        self.last_input_sample = state.last_input_sample;
        self.last_output_sample = state.last_output_sample;
        self.frame_counter_cycle = state.frame_counter_cycle;
        self.frame_counter_mode = match state.frame_counter_mode {
            0 => FrameCounterMode::Step4,
            _ => FrameCounterMode::Step5,
        };
        self.interrupt_inhibit = state.interrupt_inhibit;
        self.frame_interrupt = state.frame_interrupt;
        self.sample_buffer.clear();
    }
}