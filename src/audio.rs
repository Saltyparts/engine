pub struct Instance {
    audio_driver: AudioDriver,
    sounds: Vec<Vec<u8>>,
}

pub type Sound = u32;

impl Instance {
    pub fn new() -> Instance {
        unimplemented!()
    }

    pub fn create_sound() -> Sound {
        unimplemented!()
    }

    pub fn unload_sounds() -> Sounds {
        unimplemented!()
    }

    pub fn play_sound(&self, sound: Sound) {
        unimplemented!()
    }

    pub fn stop_sound(&self) {
        unimplemented!()
    }
}
