use super::*;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
enum HostCall {
    Begin,
    Value(f32),
    End,
}

#[derive(Default)]
struct Host {
    calls: Mutex<Vec<HostCall>>,
}

// Deliberately does not echo values back: a DAW may apply them on a later audio block.
impl GuiContext for Host {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Vst3
    }
    fn request_resize(&self) -> bool {
        true
    }
    unsafe fn raw_begin_set_parameter(&self, _: ParamPtr) {
        self.calls.lock().unwrap().push(HostCall::Begin);
    }
    unsafe fn raw_set_parameter_normalized(&self, _: ParamPtr, value: f32) {
        self.calls.lock().unwrap().push(HostCall::Value(value));
    }
    unsafe fn raw_end_set_parameter(&self, _: ParamPtr) {
        self.calls.lock().unwrap().push(HostCall::End);
    }
    fn get_state(&self) -> PluginState {
        unreachable!()
    }
    fn set_state(&self, _: PluginState) {
        unreachable!()
    }
}

struct Harness {
    ctx: egui::Context,
    host: Host,
    params: EmberFatParams,
    time: f64,
}

impl Harness {
    fn new() -> Self {
        let result = Self {
            ctx: egui::Context::default(),
            host: Host::default(),
            params: EmberFatParams::default(),
            time: 0.0,
        };
        configure_style(&result.ctx);
        result
    }

    fn frame(&mut self, events: Vec<egui::Event>, shift: bool) -> f32 {
        self.time += 0.05;
        let mut value = 0.0;
        let _ = self.ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, PANEL_SIZE)),
                time: Some(self.time),
                modifiers: if shift {
                    egui::Modifiers::SHIFT
                } else {
                    egui::Modifiers::NONE
                },
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let mut response = ui.interact(
                        egui::Rect::from_min_max(
                            egui::pos2(100.0, 100.0),
                            egui::pos2(200.0, 200.0),
                        ),
                        egui::Id::new("test-dial"),
                        egui::Sense::click_and_drag(),
                    );
                    value = interact_float_param(
                        ui,
                        &mut response,
                        &self.params.fat,
                        &ParamSetter::new(&self.host),
                        1.0,
                    );
                    draw_value(
                        ui,
                        egui::Rect::from_min_max(
                            egui::pos2(100.0, 250.0),
                            egui::pos2(200.0, 280.0),
                        ),
                        self.params.fat.value(),
                        &self.params.fat,
                        &ParamSetter::new(&self.host),
                        false,
                        1.0,
                        "typed-fat",
                    );
                    draw_bypass(
                        ui,
                        PanelLayout::new(egui::Rect::from_min_size(egui::Pos2::ZERO, PANEL_SIZE)),
                        &self.params.bypass,
                        &ParamSetter::new(&self.host),
                    );
                });
            },
        );
        value
    }

    fn begin_drag(&mut self, shift: bool) {
        self.frame(vec![], shift);
        self.frame(
            vec![
                egui::Event::PointerMoved(egui::pos2(150.0, 150.0)),
                pointer(150.0, 150.0, true, shift),
            ],
            shift,
        );
        self.frame(
            vec![egui::Event::PointerMoved(egui::pos2(150.0, 140.0))],
            shift,
        );
    }

    fn values(&self) -> Vec<f32> {
        self.host
            .calls
            .lock()
            .unwrap()
            .iter()
            .filter_map(|call| {
                if let HostCall::Value(v) = call {
                    Some(*v)
                } else {
                    None
                }
            })
            .collect()
    }

    fn click(&mut self, x: f32, y: f32) {
        self.frame(
            vec![
                egui::Event::PointerMoved(egui::pos2(x, y)),
                pointer(x, y, true, false),
            ],
            false,
        );
        self.frame(vec![pointer(x, y, false, false)], false);
    }

    fn type_value(&mut self, text: &str, accept: bool) {
        self.frame(vec![], false);
        self.click(150.0, 265.0);
        self.frame(vec![egui::Event::Text(text.into())], false);
        self.frame(
            vec![egui::Event::Key {
                key: if accept {
                    egui::Key::Enter
                } else {
                    egui::Key::Escape
                },
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            false,
        );
        self.frame(vec![], false);
    }

    fn assert_balanced(&self) {
        let calls = self.host.calls.lock().unwrap();
        let mut open = false;
        for call in calls.iter() {
            match call {
                HostCall::Begin => {
                    assert!(!open, "nested gesture: {calls:?}");
                    open = true;
                }
                HostCall::Value(v) => {
                    assert!(open);
                    assert!(v.is_finite() && (0.0..=1.0).contains(v));
                }
                HostCall::End => {
                    assert!(open, "unmatched end: {calls:?}");
                    open = false;
                }
            }
        }
        assert!(!open, "unfinished gesture: {calls:?}");
    }
}

fn pointer(x: f32, y: f32, pressed: bool, shift: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos: egui::pos2(x, y),
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: if shift {
            egui::Modifiers::SHIFT
        } else {
            egui::Modifiers::NONE
        },
    }
}

#[test]
fn dragging_accumulates_when_the_host_has_not_echoed_the_value() {
    let mut h = Harness::new();
    h.begin_drag(false);
    let first = h.frame(
        vec![egui::Event::PointerMoved(egui::pos2(150.0, 120.0))],
        false,
    );
    let next = h.frame(
        vec![egui::Event::PointerMoved(egui::pos2(150.0, 100.0))],
        false,
    );
    assert!((next - first - 12.5).abs() < 0.001, "{first} -> {next}");
    h.frame(vec![pointer(150.0, 100.0, false, false)], false);
    assert_eq!(
        h.host
            .calls
            .lock()
            .unwrap()
            .iter()
            .filter(|c| **c == HostCall::Begin)
            .count(),
        1
    );
    h.assert_balanced();
}

#[test]
fn fine_drag_retains_movements_smaller_than_parameter_quantization() {
    let mut h = Harness::new();
    h.begin_drag(true);
    let before = *h.values().last().unwrap();
    for i in 1..=40 {
        h.frame(
            vec![egui::Event::PointerMoved(egui::pos2(
                150.0,
                140.0 - i as f32 * 0.08,
            ))],
            true,
        );
    }
    let after = *h.values().last().unwrap();
    assert!(
        (after - before - 0.002).abs() < 0.001,
        "{before} -> {after}"
    );
    h.frame(vec![pointer(150.0, 136.8, false, true)], true);
    h.assert_balanced();
}

#[test]
fn dragging_outside_the_control_clamps_and_closes_the_host_gesture() {
    let mut h = Harness::new();
    h.begin_drag(false);
    let value = h.frame(
        vec![egui::Event::PointerMoved(egui::pos2(150.0, -800.0))],
        false,
    );
    assert_eq!(value, 100.0);
    h.frame(vec![pointer(150.0, -800.0, false, false)], false);
    h.assert_balanced();
}

#[test]
fn shift_arrow_changes_the_focused_control_by_one_tenth() {
    let mut h = Harness::new();
    h.frame(vec![], false);
    h.ctx
        .memory_mut(|m| m.request_focus(egui::Id::new("test-dial")));
    let value = h.frame(
        vec![egui::Event::Key {
            key: egui::Key::ArrowUp,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::SHIFT,
        }],
        true,
    );
    assert!((value - 32.1).abs() < 0.001, "{value}");
    assert_eq!(h.values().len(), 1);
    h.assert_balanced();
}

#[test]
fn typing_a_value_commits_and_escape_cancels() {
    let mut accepted = Harness::new();
    accepted.type_value("67.5", true);
    assert!((accepted.values().last().unwrap() - 0.675).abs() < 0.001);
    accepted.assert_balanced();
    let mut cancelled = Harness::new();
    cancelled.type_value("67.5", false);
    assert!(cancelled.values().is_empty(), "{:?}", cancelled.values());
    cancelled.assert_balanced();
}

#[test]
fn typed_values_are_bounded_and_non_finite_input_is_rejected() {
    let mut bounded = Harness::new();
    bounded.type_value("250", true);
    assert_eq!(bounded.values().last(), Some(&1.0));
    bounded.assert_balanced();
    let mut invalid = Harness::new();
    invalid.type_value("NaN", true);
    assert!(invalid.values().is_empty(), "{:?}", invalid.values());
    invalid.assert_balanced();
}

#[test]
fn rapid_bypass_clicks_toggle_even_before_host_feedback() {
    let mut h = Harness::new();
    h.frame(vec![], false);
    h.click(376.0, 457.0);
    h.click(376.0, 457.0);
    assert_eq!(h.values(), vec![1.0, 0.0]);
    h.assert_balanced();
}
