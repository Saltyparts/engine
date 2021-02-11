 mod audio;
 mod graphics;

fn main() -> anyhow::Result<()> {
    let instance = audio::Instance::new();
    instance.play_sound(0);
    instance.stop_sound();

    Ok(())
}
