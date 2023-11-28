use array_init::array_init;
use circular_buffer::CircleBuffer;
use nih_plug::{params::Param, prelude::*};
use nih_plug_vizia::ViziaState;
use std::{cell::RefCell, sync::Arc};

mod circular_buffer;
mod editor;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

const MAX_DELAY: usize = 2;
const BUCKETS: usize = 2;

pub struct BorderSecurityPlugin {
    params: Arc<BorderSecurityPluginParams>,
    //Channel - Buckets - Delay Buffer
    delay_buffers: Vec<RefCell<CircleBuffer>>,
}

#[derive(Params)]
pub struct BorderSecurityPluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[nested(array, group = "Delay Parameters")]
    pub delay_params: [DelayParam; BUCKETS],
}

#[derive(Params)]
pub struct DelayParam {
    /// This parameter's ID will get a `_1`, `_2`, and a `_3` suffix because of how it's used in
    /// `array_params` above.
    #[id = "delay"]
    pub delay: FloatParam,

    #[id = "threshold"]
    pub threshold: FloatParam,

    #[id = "capacity"]
    pub capacity: FloatParam,

    #[id = "factor"]
    pub factor: FloatParam,
}

impl Default for BorderSecurityPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(BorderSecurityPluginParams::default()),
            delay_buffers: Vec::new(),
        }
    }
}

impl Default for BorderSecurityPluginParams {
    fn default() -> Self {
        let delay_params: [DelayParam; BUCKETS] = array_init(|index| DelayParam {
            delay: FloatParam::new(
                format!("Delay {index}"),
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: MAX_DELAY as f32,
                },
            ),
            threshold: FloatParam::new(
                format!("Threshold {index}"),
                util::db_to_gain(-30.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            ),
            capacity: FloatParam::new(
                format!("Threshold {index}"),
                util::db_to_gain(30.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            ),
            factor: FloatParam::new(
                format!("Threshold {index}"),
                util::db_to_gain(0.0),
                FloatRange::Linear {
                    min: util::db_to_gain(-1.0),
                    max: util::db_to_gain(1.0),
                },
            ),
        });
        Self {
            editor_state: editor::default_state(),
            delay_params,
        }
    }
}

impl Plugin for BorderSecurityPlugin {
    const NAME: &'static str = "Border Security Plugin";
    const VENDOR: &'static str = "ActuallyAdequate";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "actuallyadequate@gmail.com";

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
    const HARD_REALTIME_ONLY: bool = false;

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

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        let output_channels = _audio_io_layout
            .main_output_channels
            .expect("Plugin does not have main output channels!")
            .get() as usize;

        self.delay_buffers.resize_with(output_channels, || {
            let mut delay_buffer = CircleBuffer::new();
            delay_buffer.resize(_buffer_config.sample_rate, MAX_DELAY);
            RefCell::new(delay_buffer)
        });

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
        for (i, channel) in buffer.as_slice().iter_mut().enumerate() {
            let mut delay_buffer = self.delay_buffers[i].borrow_mut();

            for sample in channel.iter_mut() {
                let crossfade_factor = 0.5;
                let mut wet_sample = 0.0;

                delay_buffer.write(*sample);

                for j in 0..BUCKETS {
                    let delay_length = self.params.delay_params[j].delay.value();
                    let threshold = self.params.delay_params[j].threshold.value();
                    let capacity = self.params.delay_params[j].capacity.value();
                    let factor = self.params.delay_params[j].factor.value();

                    let read_offset = (delay_length / MAX_DELAY as f32
                        * ((delay_buffer.samples() - 1) as f32))
                        as usize;
                    let delayed_sample = delay_buffer.read(read_offset);
                    let delay_factor = if delayed_sample > threshold && delayed_sample < capacity {
                        factor
                    } else {
                        0.0
                    };
                    wet_sample += delayed_sample * delay_factor;
                }
                *sample = *sample * (1.0 - crossfade_factor) + wet_sample * crossfade_factor;
            }
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.params.editor_state.clone())
    }

    fn task_executor(&mut self) -> TaskExecutor<Self> {
        // In the default implementation we can simply ignore the value
        Box::new(|_| ())
    }

    fn filter_state(state: &mut PluginState) {}

    fn deactivate(&mut self) {}
}

impl ClapPlugin for BorderSecurityPlugin {
    const CLAP_ID: &'static str = "com.actuallyadequate.audio.border-security-plugin";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A Gain Delay Plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];

    const CLAP_POLY_MODULATION_CONFIG: Option<PolyModulationConfig> = None;

    fn remote_controls(&self, context: &mut impl RemoteControlsContext) {}
}

impl Vst3Plugin for BorderSecurityPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"actlyadqteborsec";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];

    const PLATFORM_VST3_CLASS_ID: [u8; 16] = Self::VST3_CLASS_ID;
}

nih_export_clap!(BorderSecurityPlugin);
nih_export_vst3!(BorderSecurityPlugin);
