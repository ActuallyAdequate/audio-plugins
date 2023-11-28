use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};

use std::sync::Arc;

use crate::BorderSecurityPluginParams;

use widgets::time_slider::*;

#[derive(Lens)]
struct Data {
    params: Arc<BorderSecurityPluginParams>,
}

impl Model for Data {}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (600, 400))
}

pub(crate) fn create(
    params: Arc<BorderSecurityPluginParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        Data {
            params: params.clone(),
        }
        .build(cx);

        ResizeHandle::new(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, "Border Security GUI")
                .font_family(vec![FamilyOwned::Name(String::from(
                    assets::NOTO_SANS_THIN,
                ))])
                .font_size(30.0)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));
            Label::new(cx, "Departure Time");
            for i in 0..params.delay_params.len() {
                HStack::new(cx, |cx| {
                    ParamSlider::new(cx, Data::params, move |params| {
                        &params.delay_params[i].threshold
                    });
                    TimeSlider::new(cx, Data::params, move |params| {
                        &params.delay_params[i].delay
                    })
                    .set_style(TimeSliderStyle::CurrentStep { even: true })
                    .background_color(Color::rgb(120, 86, 28))
                    .color(Color::rgb(212, 214, 77))
                    .border_color(Color::rgb(28, 32, 46));
                    ParamSlider::new(cx, Data::params, move |params| {
                        &params.delay_params[i].capacity
                    });
                    ParamSlider::new(cx, Data::params, move |params| {
                        &params.delay_params[i].factor
                    });
                })
                .height(Auto);
            }
        });
    })
}
