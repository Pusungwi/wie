use alloc::rc::Rc;
use core::{cell::RefCell, fmt::Debug, num::NonZeroU32};

use softbuffer::{Context, Surface};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct Window {
    window: winit::window::Window,
    event_loop: Option<EventLoop<()>>,
    surface: Surface,
}

impl Window {
    pub fn new(width: u32, height: u32) -> Self {
        let event_loop = EventLoop::new();

        let size = PhysicalSize::new(width, height);

        let window = WindowBuilder::new().with_inner_size(size).with_title("WIPI").build(&event_loop).unwrap();
        let context = unsafe { Context::new(&window) }.unwrap();
        let mut surface = unsafe { Surface::new(&context, &window) }.unwrap();

        surface
            .resize(NonZeroU32::new(size.width).unwrap(), NonZeroU32::new(size.height).unwrap())
            .unwrap();

        Self {
            window,
            event_loop: Some(event_loop),
            surface,
        }
    }

    pub fn paint(&mut self, data: &[u32]) {
        let mut buffer = self.surface.buffer_mut().unwrap();
        buffer.copy_from_slice(data);

        buffer.present().unwrap();
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    fn callback<C, E>(event: wie_base::Event, control_flow: &mut ControlFlow, callback: &mut C)
    where
        C: FnMut(wie_base::Event) -> Result<(), E> + 'static,
        E: Debug,
    {
        let result = callback(event);
        if let Err(x) = result {
            tracing::error!(target: "wie", "{:?}", x);

            *control_flow = ControlFlow::Exit;
        }
    }

    pub fn run<C, E>(self_: Rc<RefCell<Self>>, mut callback: C) -> !
    where
        C: FnMut(wie_base::Event) -> Result<(), E> + 'static,
        E: Debug,
    {
        let event_loop = self_.borrow_mut().event_loop.take().unwrap();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            scancode,
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    Self::callback(wie_base::Event::Keydown(scancode), control_flow, &mut callback);
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            scancode,
                            state: ElementState::Released,
                            ..
                        },
                    ..
                } => {
                    Self::callback(wie_base::Event::Keyup(scancode), control_flow, &mut callback);
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                Self::callback(wie_base::Event::Update, control_flow, &mut callback);
            }
            Event::RedrawRequested(_) => {
                Self::callback(wie_base::Event::Redraw, control_flow, &mut callback);
            }

            _ => {}
        })
    }
}
