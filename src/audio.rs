use std::{
    fs::File,
    io::{
        Cursor,
        Read,
    },
    path::Path,
    sync::Arc,
};

pub struct Instance {
    // stream: rodio::OutputStream,
    // stream_handle: rodio::OutputStreamHandle,
}

impl Instance {
    pub fn new() -> Instance {
        Instance {
            // fields go here
        } // <-- no semicolon is the same as return
    }

    pub fn create_sink(&self) -> Sink {
        Sink {
            // fields go here
        }
    }
}

pub struct Sink {
    // sink: rodio::Sink,
}

impl Sink {
    pub fn queue_sound(&self, sound: &Sound) {
        unimplemented!()
    }

    pub fn stop_sound(&self) {
        unimplemented!()
    }

    pub fn set_volume(&self, volume: f32) {
        unimplemented!()
    }
}

pub struct Sound(Arc<Vec<u8>>);

impl Sound {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Sound> {
        let mut buf = Vec::new();
        let mut file = File::open(path)?;
        file.read_to_end(&mut buf)?;
        Ok(Sound(Arc::new(buf)))
    }

    fn as_cursor(&self) -> Cursor<Sound> {
        Cursor::new(Sound(self.0.clone()))
    }

    fn as_decoder(&self) -> rodio::Decoder<Cursor<Sound>> {
        rodio::Decoder::new(self.as_cursor()).unwrap()
    }
}

impl AsRef<[u8]> for Sound {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
