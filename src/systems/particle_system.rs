use rltk::{Rltk, RGB};
use specs::prelude::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use specs_derive::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::map_builders::{common::xy_idx, map::Map};

#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Copy, Clone, Debug)]
pub enum OnDeathAction {
    RevealMap,
    NoAction
}

#[derive(ConvertSaveload, Clone)]
pub struct OnDeath {
    pub action: OnDeathAction,
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct Particle {
    action: OnDeathAction,
    pub pos: (i32, i32),
    pub fg: RGB,
    pub bg: RGB,
    pub glyph: rltk::FontCharType,
    pub render_order: i32,
    lifetime: f32,
}

#[derive(Default, Component, Serialize, Deserialize, Clone)]
pub struct Particles {
    pub particle: HashMap<String, Particle>,
}

pub struct ParticleSpawnSystem {}

impl<'a> System<'a> for ParticleSpawnSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Particles>,
        WriteExpect<'a, ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut particles,
            mut particle_builder,
        ) = data;
        for particle in particles.particle.values_mut() {
            particle.render_order = (particle.render_order + 1) % 4;
        }

        for new_particle in particle_builder.requests.iter() {
            let p = Particle {
                action: new_particle.on_death,
                pos: (new_particle.x, new_particle.y),
                fg: new_particle.fg,
                bg: new_particle.bg,
                glyph: new_particle.glyph,
                render_order: 0,
                lifetime: new_particle.lifetime,

            };
            let idx = xy_idx(new_particle.x, new_particle.y);
            let key = format!("{} @ {} for {}", p.glyph, idx, p.lifetime);
            particles.particle.insert(key, p);
        }
        particle_builder.requests.clear();
    }
}

struct ParticleRequest {
    x: i32,
    y: i32,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
    lifetime: f32,
    on_death: OnDeathAction,
}

pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ParticleBuilder {
        ParticleBuilder {
            requests: Vec::new(),
        }
    }

    pub fn request(
        &mut self,
        x: i32,
        y: i32,
        fg: RGB,
        bg: RGB,
        glyph: rltk::FontCharType,
        lifetime: f32,
        on_death: OnDeathAction,
    ) {
        self.requests.push(ParticleRequest {
            x,
            y,
            fg,
            bg,
            glyph,
            lifetime,
            on_death,
        });
    }
}

pub fn cull_dead_particles(ecs: &mut World, ctx: &Rltk) {
    let mut particles = ecs.fetch_mut::<Particles>();
    let mut dead_particles = Vec::new();
    {
        // Age out particles
        let mut map = ecs.fetch_mut::<Map>();
        let mut particle_builder = ecs.fetch_mut::<ParticleBuilder>();
        for (key, particle) in particles.particle.iter_mut() {
            particle.lifetime -= ctx.frame_time_ms;
            if particle.lifetime > 0.0 {
                match particle.action {
                    OnDeathAction::RevealMap => { 
                        let (x, y) = particle.pos;
                        map.reveal_me (x, y); 
                        for delta_x in -1..=1 {
                            for delta_y in -1..=1 {
                                if x + delta_x >= 0 && x + delta_x <= map.width-1 && y + delta_y >= 0 && y + delta_y <= map.height - 1 {
                                    if !map.is_magic_mapped (x + delta_x, y + delta_y) {
                                        particle_builder.request( x+delta_x, y+delta_y, RGB::named(rltk::MAGENTA), RGB::named(rltk::ORANGE), rltk::to_cp437('*'), 20.0, OnDeathAction::RevealMap);
                                    }
                                }
                            }
                        }
                    }
                    _ => { }
                }
            } else {
                dead_particles.push(key.to_owned());
            }
        }
    }
    for key in dead_particles.iter() {
        particles.particle.remove(key);
    }
    dead_particles.clear();

}
