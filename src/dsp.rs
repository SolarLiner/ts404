use valib::dsp::blocks::Series;
use valib::dsp::DSP;
use valib::filters::biquad::Biquad;
use valib::filters::statespace::StateSpace;
use valib::saturators::{Linear, Slew};
use valib::Scalar;
use valib::simd::AutoSimd;

#[derive(Debug, Copy, Clone)]
pub struct InputStage<T: Scalar> {
    pub gain: T,
    state_space: StateSpace<T, 1, 1, 1>,
}

impl<T: Scalar> DSP<1, 1> for InputStage<T> {
    type Sample = T;

    fn process(&mut self, [x]: [Self::Sample; 1]) -> [Self::Sample; 1] {
        self.state_space.process([x*self.gain])
    }

    fn latency(&self) -> usize {
        self.state_space.latency()
    }

    fn reset(&mut self) {
        self.state_space.reset()
    }
}

impl<T: Scalar> InputStage<T> {
    pub fn new(samplerate: T, gain: T) -> Self {
        Self {
            gain,
            state_space: crate::gen::input(samplerate.simd_recip()),
        }
    }

    pub fn set_samplerate(&mut self, samplerate: T) {
        self.state_space.update_matrices(&crate::gen::input(samplerate.simd_recip()));
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ClipperStage<T: Scalar>(StateSpace<T, 1, 3, 1>, Slew<T>);

impl<T: Scalar> DSP<1, 1> for ClipperStage<T> {
    type Sample = T;

    fn process(&mut self, x: [Self::Sample; 1]) -> [Self::Sample; 1] {
        let [y] = self.0.process(x);
        self.1.process([y.simd_asinh()])
    }

    fn latency(&self) -> usize {
        self.0.latency() + self.1.latency()
    }

    fn reset(&mut self) {
        self.0.reset();
        self.1.reset();
    }
}

impl<T: Scalar> ClipperStage<T> {
    pub fn new(samplerate: T, dist: T) -> Self {
        let dt = samplerate.simd_recip();
        Self(crate::gen::clipper(dist, dt), Slew::new(T::from_f64(1e4) * dt))
    }

    pub fn set_params(&mut self, samplerate: T, dist: T) {
        let dt = samplerate.simd_recip();
        self.0.update_matrices(&crate::gen::clipper(dist, dt));
        self.1.set_max_diff(T::from_f64(1e5), dt);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ToneStage<T: Scalar>(StateSpace<T, 1, 4, 1>);

impl<T: Scalar> DSP<1, 1> for ToneStage<T> {
    type Sample = T;

    fn process(&mut self, x: [Self::Sample; 1]) -> [Self::Sample; 1] {
        self.0.process(x)
    }

    fn latency(&self) -> usize {
        self.0.latency()
    }

    fn reset(&mut self) {
        self.0.reset();
    }
}

impl<T: Scalar> ToneStage<T> {
    pub fn new(samplerate: T, tone: T) -> Self {
        Self(crate::gen::tone(tone, samplerate.simd_recip()))
    }

    pub fn update_params(&mut self, samplerate: T, tone: T) {
        self.0.update_matrices(&crate::gen::tone(tone, samplerate.simd_recip()));
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OutputStage<T: Scalar> {
    pub inner: StateSpace<T, 1, 2, 1>,
    pub gain: T,
}

impl<T: Scalar> OutputStage<T> {
    pub fn new(samplerate: T, gain: T) -> Self {
        Self {
            inner: crate::gen::output(samplerate.simd_recip()),
            gain,
        }
    }
    pub fn set_samplerate(&mut self, samplerate: T) {
        self.inner.update_matrices(&crate::gen::output(samplerate.simd_recip()));
    }
}

impl<T: Scalar> DSP<1, 1> for OutputStage<T> {
    type Sample = T;

    fn process(&mut self, x: [Self::Sample; 1]) -> [Self::Sample; 1] {
        let [y] = self.inner.process(x);
        [y*self.gain]
    }

    fn latency(&self) -> usize {
        DSP::latency(&self.inner)
    }

    fn reset(&mut self) {
        DSP::reset(&mut self.inner)
    }
}