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
    last_activity: std::time::Instant,
    exit: bool,
}

impl Spike {
    fn sector_for(&self, x: f64, y: f64) -> Option<usize> {
        let cx = self.width as f64 / 2.0;
        let cy = self.height as f64 / 2.0;
        let dx = x - cx;
        let dy = y - cy;
        let distance = dx.hypot(dy);
        if distance <= CENTER_CANCEL_RADIUS {
            return None;
        }
        let mut angle = dy.atan2(dx);
        if angle < 0.0 {
            angle += 2.0 * PI;
        }
        let sector_size = (2.0 * PI) / SECTOR_COUNT as f64;
        let mut index = (angle / sector_size).floor() as usize;
        if index >= SECTOR_COUNT {
            index = SECTOR_COUNT - 1;
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
                    self.cursor = Some(Cursor { x: ev.position.0, y: ev.position.1 });
                    self.last_activity = std::time::Instant::now();
                }
                PointerEventKind::Press { .. } => {
                    self.last_activity = std::time::Instant::now();
                    if let Some(cursor) = self.cursor {
                        match self.sector_for(cursor.x, cursor.y) {
                            None => {
                                println!("CANCEL (center)");
                                self.exit = true;
                            }
                            Some(index) => {
                                println!("SELECTED SECTOR {}", index);
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
    fn draw(&mut self, _qh: &QueueHandle<Self>) {
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
            for b in canvas.iter_mut() {
                *b = 0;
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

