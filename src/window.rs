

use std::sync::mpsc::Receiver;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use crate::graphics::Renderer;
use crate::logic::{Asset, Position};

pub struct Window {
    event_loop: Option<glutin::event_loop::EventLoop<()>>,
    context: glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>,
    receiver : Receiver<Vec<(Asset, Position)>>,
    renderer : Renderer,
}

pub type EventHandler = Box<dyn FnMut(glutin::event::Event<()>) + Send + 'static>;

impl Window {
    ///
    /// unsafe, since calling twice on the same thread is likely to lead to serious trouble.
    /// Also, extremely stateful.
    pub unsafe fn new(receiver : Receiver<Vec<(Asset, Position)>>) -> Window {
        let el = glutin::event_loop::EventLoop::new();
        let wb = glutin::window::WindowBuilder::new()
            .with_title("Hello world!")
            .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
        let windowed_context = glutin::ContextBuilder::new()
            .build_windowed(wb, &el)
            .unwrap();

        let windowed_context = windowed_context.make_current().unwrap();

        println!("Loading gl!");
        gl::load_with(|s| windowed_context.get_proc_address(s) as *const _);

        gl::ClearColor(0.3, 0.3, 0.5, 1.0);

        //TODO: DEPTH TESTING IS OPTIONAL, NOT REQUIRED!
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        gl::ClearColor(0.6, 0.6, 0.6, 1.0);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);

        gl::PointSize(5.0);

        let screen_dimensions = (
            windowed_context.window().inner_size().width,
            windowed_context.window().inner_size().height,
        );

        let res = Window {
            event_loop: Some(el),
            context: windowed_context,
            renderer : Renderer::new(screen_dimensions),
            receiver
        };

        res
    }

    unsafe fn render(&mut self) {
        let rec = match self.receiver.try_recv() {
            Ok(o) => o,
            Err(_) => return
        };

        self.renderer.render(&rec);
        self.context.swap_buffers().unwrap();
    }

    unsafe fn update_screen_dimensions(&mut self, screen_dimensions: (u32, u32)) {
        gl::Viewport(0, 0, screen_dimensions.0 as i32, screen_dimensions.1 as i32);
    }

    ///
    /// Starts this graphics object
    ///
    /// NOTE: THE EXECUTION GOES TO THE GRAPHICS OBJECT WHEN THIS IS CALLED!
    ///
    pub unsafe fn run(mut self, mut event_handler: EventHandler) {
        //Ignore the result from the function.
        let _ = self.context.window().set_cursor_grab(true);
        self.context.window().set_cursor_visible(false);
        if let Some(el) = self.event_loop.take() {
            el.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Poll;

                match event {
                    Event::LoopDestroyed => {} //TODO: HANDLE LOOP DESTROYED
                    Event::WindowEvent { window_id, event } => match event {
                        WindowEvent::Resized(physical_size) => {
                            self.context.resize(physical_size);
                            self.update_screen_dimensions((
                                physical_size.width,
                                physical_size.height,
                            ));
                        }
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        _ => event_handler(Event::WindowEvent { window_id, event }),
                    },
                    Event::RedrawRequested(_) => {}
                    Event::NewEvents(cs) => match cs {
                        glutin::event::StartCause::Poll => {
                            // Perform a render
                            self.render();
                        }
                        _ => event_handler(Event::NewEvents(cs)),
                    },
                    _ => event_handler(event),
                }
            });
        } else {
            panic!("Graphics object was told to run, but the event loop is already consumed!");
        }
    }

    //pub fn message(&mut self, messages: RenderPack<T>) {
    //    self.render_messages = Some(messages);
    //}
}
