use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::wlr_layer::{
        Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
        LayerSurfaceConfigure,
    },
    shell::WaylandSurface,
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use std::f64::consts::PI;
use pixelens_config::load_config;
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};

const SECTOR_COUNT: usize = 5;
const CENTER_CANCEL_RADIUS: f64 = 48.0;
const INACTIVE_TIMEOUT_SECS: u64 = 2;

#[derive(Clone, Copy)]
struct Cursor {
    x: f64,
    y: f64,
}

const PALETTE: [(u8,u8,u8); 5] = [
    (0x35, 0x84, 0xe4),
    (0x33, 0xd1, 0x7a),
    (0xf6, 0xd3, 0x2d),
    (0xf6, 0x61, 0x51),
    (0x91, 0x41, 0xac),
];

struct Spike {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    pool: Option<SlotPool>,
    width: u32,
    height: u32,
    surface: wl_surface::WlSurface,
    layer: LayerSurface,
    pointer: Option<wl_pointer::WlPointer>,
    cursor: Option<Cursor>,
    anchor: Option<Cursor>,
    opened_at: std::time::Instant,
    sectors: Vec<String>,
    last_activity: std::time::Instant,
    exit: bool,
}

impl Spike {
    fn sector_for(&self, anchor: Cursor, point: Cursor) -> Option<usize> {
        let dx = point.x - anchor.x;
        let dy = point.y - anchor.y;
        let distance = dx.hypot(dy);
        if distance <= CENTER_CANCEL_RADIUS {
            return None;
        }
        let mut angle = dy.atan2(dx);
        if angle < 0.0 {
            angle += 2.0 * PI;
        }
        let count = self.sectors.len().max(1);
        let sector_size = (2.0 * PI) / count as f64;
        let mut index = (angle / sector_size).floor() as usize;
        if index >= count {
            index = count - 1;
        }
        Some(index)
    }
}

// Trait wiring.
impl CompositorHandler for Spike {
    fn scale_factor_changed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _new_factor: i32) {}
    fn transform_changed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _new_transform: wl_output::Transform) {}
    fn frame(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _time: u32) {
        self.draw(qh);
    }
    fn surface_enter(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _output: &wl_output::WlOutput) {}
    fn surface_leave(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _output: &wl_output::WlOutput) {}
}

impl OutputHandler for Spike {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
}

impl LayerShellHandler for Spike {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }
    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if configure.new_size.0 == 0 || configure.new_size.1 == 0 {
            self.width = 1;
            self.height = 1;
        } else {
            self.width = configure.new_size.0;
            self.height = configure.new_size.1;
        }
        println!("overlay configured: {}x{} (center cancel radius {})", self.width, self.height, CENTER_CANCEL_RADIUS);
        if self.pool.is_none() {
            self.pool = Some(SlotPool::new((self.width * self.height * 4) as usize, &self.shm).unwrap());
        }
        self.last_activity = std::time::Instant::now();
        self.draw(qh);
    }
}

impl SeatHandler for Spike {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {}
    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_none() {
            self.pointer = Some(self.seat_state.get_pointer(qh, &seat).unwrap());
        }
    }
    fn remove_capability(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Pointer {
            self.pointer = None;
        }
    }
    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {}
}

impl PointerHandler for Spike {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for ev in events {
            match ev.kind {
                PointerEventKind::Motion { .. } => {
                    let cursor = Cursor { x: ev.position.0, y: ev.position.1 };
                    if self.anchor.is_none() {
                        self.anchor = Some(cursor);
                    }
                    self.cursor = Some(cursor);
                    self.last_activity = std::time::Instant::now();
                }
                PointerEventKind::Press { .. } => {
                    self.last_activity = std::time::Instant::now();
                    let anchor = self.anchor.or(self.cursor);
                    if let (Some(anchor), Some(cursor)) = (anchor, self.cursor) {
                        match self.sector_for(anchor, cursor) {
                            None => {
                                println!("CANCEL (center)");
                                self.exit = true;
                            }
                            Some(index) => {
                                let tag = self
                                    .sectors
                                    .get(index)
                                    .cloned()
                                    .unwrap_or_else(|| "grab".to_string());
                                println!("SELECTED SECTOR {} ({})", index, tag);
                                let _ = std::process::Command::new("pixelens").arg(&tag).spawn();
                                self.exit = true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl ShmHandler for Spike {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl smithay_client_toolkit::registry::ProvidesRegistryState for Spike {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(Spike);
delegate_output!(Spike);
delegate_layer!(Spike);
delegate_seat!(Spike);
delegate_pointer!(Spike);
delegate_registry!(Spike);
delegate_shm!(Spike);

impl Spike {
    fn draw(&mut self, qh: &QueueHandle<Self>) {
            if let Some(pool) = &mut self.pool {
                if self.width == 0 || self.height == 0 {
                    return;
                }
                let (buffer, canvas) = pool
                    .create_buffer(
                        self.width as i32,
                        self.height as i32,
                        (self.width * 4) as i32,
                        wl_shm::Format::Argb8888,
                    )
                    .unwrap();
                let center = self.anchor.or(self.cursor).unwrap_or(Cursor {
                    x: self.width as f64 / 2.0,
                    y: self.height as f64 / 2.0,
                });
                let count = self.sectors.len().max(1);
                let radius = 120.0;
                let hover_sector = match (self.anchor, self.cursor) {
                    (Some(anchor), Some(cursor)) => {
                        let dx = cursor.x - anchor.x;
                        let dy = cursor.y - anchor.y;
                        let dist = dx.hypot(dy);
                        if dist <= CENTER_CANCEL_RADIUS {
                            None
                        } else {
                            let mut angle = dy.atan2(dx);
                            if angle < 0.0 {
                                angle += 2.0 * PI;
                            }
                            Some(((angle / (2.0 * PI / count as f64)).floor() as usize).min(count - 1))
                        }
                    }
                    _ => None,
                };
                let intro = (self.opened_at.elapsed().as_secs_f64() / 0.12).clamp(0.0, 1.0);
                for y in 0..self.height {
                    for x in 0..self.width {
                        let idx = ((y * self.width + x) * 4) as usize;
                        let dx = x as f64 - center.x;
                        let dy = y as f64 - center.y;
                        let distance = dx.hypot(dy);
                        if distance > radius {
                            canvas[idx] = 0;
                            canvas[idx + 1] = 0;
                            canvas[idx + 2] = 0;
                            canvas[idx + 3] = 0;
                            continue;
                        }
                        if distance <= CENTER_CANCEL_RADIUS {
                            let v = (0x28 as f64 * intro) as u8;
                            canvas[idx] = v;
                            canvas[idx + 1] = v;
                            canvas[idx + 2] = v;
                            canvas[idx + 3] = (0x80 as f64 * intro) as u8;
                            continue;
                        }
                        let mut angle = dy.atan2(dx);
                        if angle < 0.0 {
                            angle += 2.0 * PI;
                        }
                        let sector = ((angle / (2.0 * PI / count as f64)).floor() as usize).min(count - 1);
                        let (r, g, b) = PALETTE[sector % PALETTE.len()];
                        let alpha = if Some(sector) == hover_sector { 0xdd } else { 0x70 };
                        canvas[idx] = b;
                        canvas[idx + 1] = g;
                        canvas[idx + 2] = r;
                        canvas[idx + 3] = ((alpha as f64) * intro) as u8;
                    }
                }
                self.surface.attach(Some(buffer.wl_buffer()), 0, 0);
                self.surface.damage_buffer(0, 0, self.width as i32, self.height as i32);
                self.surface.commit();
            }
        }
}

fn main() {
    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor missing");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell missing");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm missing");

    let surface = compositor.create_surface(&qh);
    let layer = layer_shell.create_layer_surface(&qh, surface.clone(), Layer::Overlay, Some("pixelens_spike"), None);
    layer.set_anchor(Anchor::all());
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.set_exclusive_zone(-1);
    layer.set_size(0, 0);
    WaylandSurface::commit(&layer);

    let config = load_config().unwrap_or_default();
    let sectors = if config.gui.sectors.is_empty() { vec!["copy".into(), "search".into(), "ai".into(), "translate".into(), "image".into()] } else { config.gui.sectors };

    let mut spike = Spike {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        pool: None,
        width: 1,
        height: 1,
        surface,
        layer,
        pointer: None,
        cursor: None,
        anchor: None,
        sectors,
        opened_at: std::time::Instant::now(),
        last_activity: std::time::Instant::now(),
        exit: false,
    };

    let timeout = std::time::Instant::now() + std::time::Duration::from_secs(INACTIVE_TIMEOUT_SECS as u64 + 2);
    while !spike.exit && std::time::Instant::now() < timeout {
        if spike.last_activity.elapsed().as_secs_f64() > INACTIVE_TIMEOUT_SECS as f64 {
            println!("TIMEOUT (no activity)");
            break;
        }
        event_queue.blocking_dispatch(&mut spike).unwrap();
    }
    println!("spike finished");
}

