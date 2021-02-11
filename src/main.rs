mod audio;
mod graphics;

use winit::{
   event::{Event, WindowEvent},
   event_loop::{ControlFlow, EventLoop},
   window::WindowBuilder,
};

const RESOLUTION: [u32; 2] = [1280, 720];

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(RESOLUTION[0], RESOLUTION[1]))
        .build(&event_loop)?;

    let mut graphics_instance = graphics::Instance::new(&window, RESOLUTION).await?;
    let barbarian = graphics::Model::new("content/barbarian.obj")?;
    let view_matrix = [
        1., 0., 0., 0.,
        0., 1., 0., 0.,
        0., 0., 1., 0.,
        0., 0., 0., 1.,
    ];

    let audio_instance = audio::Instance::new();
    let sink = audio_instance.create_sink();
    //let sound = audio::Sound::new("some/path/to/file.ogg")?;
    //sink.queue_sound(&sound);
    //sink.set_volume(0.5);
    //sink.stop_sound();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => graphics_instance.resize([size.width, size.height]),
            Event::WindowEvent { event: WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size }, .. } => {
                graphics_instance.resize([new_inner_size.width, new_inner_size.height])
            },
            Event::MainEventsCleared => graphics_instance.render(&view_matrix, &[&barbarian]).unwrap(),
            _ => (),
        }
    });
}
