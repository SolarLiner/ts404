use std::collections::HashMap;
use std::sync::Arc;

use nih_plug::prelude::*;
use nih_plug_slint::editor::SlintEditor;
use nih_plug_slint::plugin_component_handle::{
    PluginComponentHandle, PluginComponentHandleParameterEvents,
};
use nih_plug_slint::{LogicalSize, WindowAttributes};
use slint::{SharedString, Window};

use crate::Ts404Params;

slint::include_modules!();

pub struct PluginComponent {
    component: PluginWindow,
    param_map: HashMap<SharedString, ParamPtr>,
}

impl PluginComponent {
    pub fn new(params: &Ts404Params) -> Self {
        let component = PluginWindow::new().unwrap();
        let param_map = params
            .param_map()
            .into_iter()
            .map(|(name, param_ptr, _)| (name.into(), param_ptr))
            .collect();
        Self {
            component,
            param_map,
        }
    }

    fn convert_parameter(&self, id: &str) -> PluginParameter {
        let param_ptr = self.param_map[id];
        let name = unsafe { param_ptr.name().into() };
        let value = unsafe { param_ptr.unmodulated_normalized_value() };
        let default_value = unsafe { param_ptr.default_normalized_value() };
        let display_value = unsafe { param_ptr.normalized_value_to_string(value, true) }.into();
        let modulated_value = unsafe { param_ptr.modulated_normalized_value() };

        PluginParameter {
            id: id.into(),
            name,
            value,
            default_value,
            display_value,
            modulated_value,
        }
    }

    fn set_parameter(&self, id: &str, parameter: PluginParameter) {
        match id {
            "drive" => self.component.set_drive(parameter),
            "dist" => self.component.set_distortion(parameter),
            "tone" => self.component.set_tone(parameter),
            "level" => self.component.set_output_volume(parameter),
            _ => unimplemented!(),
        }
    }
}

impl PluginComponentHandle for PluginComponent {
    fn window(&self) -> &Window {
        self.component.window()
    }

    fn param_map(&self) -> &HashMap<SharedString, ParamPtr> {
        &self.param_map
    }

    fn update_parameter_value(&self, id: &str) {
        let parameter = self.convert_parameter(id);
        self.set_parameter(id, parameter);
    }

    fn update_parameter_modulation(&self, id: &str) {
        self.update_parameter_value(id);
    }

    fn update_all_parameters(&self) {
        for id in self.param_map.keys() {
            self.update_parameter_value(id);
        }
    }
}

impl PluginComponentHandleParameterEvents for PluginComponent {
    fn on_start_parameter_change(&self, mut f: impl FnMut(slint::SharedString) + 'static) {
        self.component
            .on_start_change(move |parameter| f(parameter.id.into()));
    }

    fn on_parameter_changed(&self, mut f: impl FnMut(slint::SharedString, f32) + 'static) {
        self.component
            .on_changed(move |parameter, value| f(parameter.id.into(), value));
    }

    fn on_end_parameter_change(&self, mut f: impl FnMut(slint::SharedString) + 'static) {
        self.component
            .on_end_change(move |parameter| f(parameter.id.into()));
    }

    fn on_set_parameter_string(
        &self,
        mut f: impl FnMut(slint::SharedString, slint::SharedString) + 'static,
    ) {
        self.component
            .on_set_string(move |parameter, string| f(parameter.id.into(), string));
    }
}

pub fn create_editor(params: Arc<Ts404Params>) -> impl Editor {
    let window_attributes = WindowAttributes::new(LogicalSize::new(500.0, 150.0), 1.0);
    let editor = SlintEditor::new(window_attributes, move |_, _| PluginComponent::new(&params));
    editor
}
