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
    artwork: Artwork,
    params: EmberFatParams,
    time: f64,
}

impl Harness {
    fn new() -> Self {
        let ctx = egui::Context::default();
        configure_style(&ctx);
        let artwork = load_artwork(&ctx);
        let result = Self {
            ctx,
            artwork,
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
                        &self.artwork,
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

// Optional headless export uses the same paint meshes and textures as the VST3 editor.
// Set EMBERFAT_PREVIEW_DIR to inspect each tiger stage and the bypass state.
#[test]
fn render_supplied_artwork_previews() {
    use std::fmt::Write as _;
    let Some(directory) = std::env::var_os("EMBERFAT_PREVIEW_DIR") else {
        return;
    };
    for (name, fat, bypassed, factor) in [
        ("default", 32.0, false, 1.0),
        ("stage-1", 0.0, false, 1.0),
        ("stage-2", 20.0, false, 1.0),
        ("stage-3", 40.0, false, 1.0),
        ("stage-4", 60.0, false, 1.0),
        ("stage-5", 80.0, false, 1.0),
        ("stage-6", 100.0, false, 1.0),
        ("bypassed-small", 100.0, true, 0.75),
    ] {
        let path = std::path::Path::new(&directory).join(name);
        std::fs::create_dir_all(&path).unwrap();
        let mut h = Harness::new();
        h.params.fat = FloatParam::new(
            "Fat",
            fat,
            FloatRange::Linear {
                min: 0.0,
                max: 100.0,
            },
        )
        .with_step_size(0.1);
        h.params.bypass = BoolParam::new("Bypass", bypassed);
        let mut texture_records = vec![];
        let mut last = None;
        for frame in 0..3 {
            let out = h.ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        PANEL_SIZE * factor,
                    )),
                    time: Some(frame as f64 * 0.1),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::new().fill(INK).inner_margin(0.0))
                        .show(ctx, |ui| {
                            draw_front_panel(ui, &h.artwork, &h.params, &ParamSetter::new(&h.host))
                        });
                },
            );
            for (id, delta) in &out.textures_delta.set {
                let egui::TextureId::Managed(id) = id else {
                    panic!("Unexpected user texture")
                };
                let im = match &delta.image {
                    egui::ImageData::Color(im) => (**im).clone(),
                    egui::ImageData::Font(im) => egui::ColorImage {
                        size: im.size,
                        pixels: im.srgba_pixels(None).collect(),
                    },
                };
                let data: Vec<u8> = im.pixels.iter().flat_map(|p| p.to_array()).collect();
                let file = format!("texture-{frame}-{id}.rgba");
                std::fs::write(path.join(&file), data).unwrap();
                let pos = delta
                    .pos
                    .map(|p| format!("[{},{}]", p[0], p[1]))
                    .unwrap_or("null".into());
                texture_records.push(format!(
                    "{{\"id\":{id},\"file\":\"{file}\",\"size\":[{},{}],\"pos\":{pos}}}",
                    im.size[0], im.size[1]
                ));
            }
            last = Some(out);
        }
        let out = last.unwrap();
        let meshes = h.ctx.tessellate(out.shapes, out.pixels_per_point);
        let mut records = vec![];
        for primitive in meshes {
            let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive else {
                panic!("Unexpected paint callback")
            };
            let egui::TextureId::Managed(id) = mesh.texture_id else {
                panic!("Unexpected user texture")
            };
            let mut vertices = String::new();
            for (i, v) in mesh.vertices.iter().enumerate() {
                assert!(v.pos.x.is_finite() && v.pos.y.is_finite());
                if i > 0 {
                    vertices.push(',');
                }
                let c = v.color.to_array();
                write!(
                    vertices,
                    "[{},{},{},{},{},{},{},{}]",
                    v.pos.x, v.pos.y, v.uv.x, v.uv.y, c[0], c[1], c[2], c[3]
                )
                .unwrap();
            }
            let indices = mesh
                .indices
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let clip = primitive.clip_rect;
            records.push(format!("{{\"texture\":{id},\"clip\":[{},{},{},{}],\"vertices\":[{vertices}],\"indices\":[{indices}]}}",clip.min.x,clip.min.y,clip.max.x,clip.max.y));
        }
        let json = format!(
            "{{\"size\":[{},{}],\"textures\":[{}],\"meshes\":[{}]}}",
            PANEL_SIZE.x * factor,
            PANEL_SIZE.y * factor,
            texture_records.join(","),
            records.join(",")
        );
        std::fs::write(path.join("paint.json"), json).unwrap();
    }
}
