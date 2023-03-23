use imgui::Ui;

pub struct CpuWindow {}

impl CpuWindow {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, ui: &Ui) {
        ui.window("CPU")
            .size([100.0, 100.0], imgui::Condition::FirstUseEver)
            .build(|| {});
    }
}
