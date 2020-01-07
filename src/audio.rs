use amethyst::{
    assets::{Loader, AssetStorage},
    audio::{OggFormat, SourceHandle, output::Output, Source},
    ecs::{World, WorldExt},
};

pub struct Sounds {
    score_sfx: SourceHandle,
    bounce_sfx: SourceHandle,
}

fn load_audio(loader: &Loader, world: &World, file: &str) -> SourceHandle {
    loader.load(file, OggFormat, (), &world.read_resource())
}

pub fn initialize_sound(world: &mut World) {
    let sounds = {
        let loader = world.read_resource::<Loader>();

        let sounds = Sounds {
            score_sfx: load_audio(&loader, world, "audio/score.ogg"),
            bounce_sfx: load_audio(&loader, world, "audio/bounce.ogg"),
        };
        sounds
    };

    world.insert(sounds);
}

impl Sounds {
    pub fn play_bounce_sound(&self, storage: &AssetStorage<Source>, output: Option<&Output>) {
        if let Some(output) = output {
            if let Some(sound) = storage.get(&self.bounce_sfx) {
                output.play_once(sound, 1.0)
            }
        }
    }

    pub fn play_score_sound(&self, storage: &AssetStorage<Source>, output: Option<&Output>) {
        if let Some(output) = output {
            if let Some(sound) = storage.get(&self.score_sfx) {
                output.play_once(sound, 1.0)
            }
        }
    }
}
