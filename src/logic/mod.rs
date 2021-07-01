mod grid;


use std::{cell::RefCell, sync::mpsc::{SyncSender}};

use bitflags::bitflags;

use self::grid::EntityGrid;
type EntityId = u16;


bitflags! {
    pub struct CompFlag : u32 {
        const Pos = 0b1;
        const Vel = 0b10;
        const Ass = 0b100;
    }    
}

#[derive(Default, Debug, Clone)]
pub struct Position {
    pub x : f32,
    pub y : f32
}

#[derive(Default, Clone)]
pub struct Velocity {
    pub x : f32,
    pub y : f32
}

#[derive(Default, Clone)]
pub struct Asset {
    texture : String
}

pub struct Entity {
    id : EntityId,
    components : CompFlag
}

pub struct Game {
    pub entities : Vec<Entity>,
    pub positions : Vec<Position>,
    pub velocities : Vec<Velocity>,
    pub collision_buffer_pos : Vec<Position>,
    pub collision_buffer_vel : Vec<Velocity>,
    pub assets : Vec<Asset>,
    pub spacially_sorted : EntityGrid,
}

impl Game {
    // Dense database, CompFlag indicates if the entity has the corresponding component
    pub fn new() -> Self {
        Self {
            entities : Vec::new(),
            positions : Vec::new(),
            velocities : Vec::new(),
            collision_buffer_pos : Vec::new(),
            collision_buffer_vel : Vec::new(),
            assets : Vec::new(),
            spacially_sorted : EntityGrid::new(1.0),
        }
    }

    pub fn add_entity(&mut self, components : CompFlag) -> EntityId {
        let id = self.entities.len() as EntityId;
        self.entities.push(Entity {
            id,
            components
        });
        self.positions.push(Position::default());
        self.velocities.push(Velocity::default());
        self.assets.push(Asset::default());
        self.collision_buffer_pos.resize(self.positions.len(), Position::default());
        self.collision_buffer_vel.resize(self.positions.len(), Velocity::default());

        id
    }

    fn apply_veloc<'a>(physics_entities : impl Iterator<Item = (&'a Position, &'a Velocity)>) -> Vec<Position>{
        physics_entities.map(|(pos, vel)| Position{
            x : pos.x+vel.x, y : pos.y + vel.y
        }).collect()
    }

    fn collide<'a>(physics_entities : impl Iterator<Item = EntityId>, spacially_sorted : &EntityGrid, positions : &[Position], velocities : &[Velocity], pos_buffer : &mut [Position], vel_buffer : &mut [Velocity]) {
        
        let iterator = physics_entities.map(|i| {
            let pos = &positions[i as usize];
            let vel = &velocities[i as usize];

            let nearby_entities = spacially_sorted.find_nearby(&pos);
            match nearby_entities {
                None => (),
                Some(r) => (
                    for nearby_entity in r.iter() {
                        
                    }
                     //TODO: Collision detection
                )
            }

            (pos.clone(), vel.clone())
        });
        for (i,j) in iterator.enumerate() {
            pos_buffer[i] = j.0;
            vel_buffer[i] = j.1;
        }
    }

    pub fn update(&mut self, wd_sender : &mut SyncSender<Vec<(Asset, Position)>>) {
        let physics_entities = self.entities.iter()
            .filter(|x| x.components.contains(CompFlag::Pos | CompFlag::Vel))
            .map(|x| &x.id);

        let b = physics_entities.clone().map(|i| (&self.positions[*i as usize], &self.velocities[*i as usize]));

        let new_positions = Game::apply_veloc(b);
        for (i, j) in physics_entities.zip(new_positions) {
            self.positions[*i as usize] = j
        }

        
        let physics_entities = self.entities.iter()
            .filter(|x| x.components.contains(CompFlag::Pos | CompFlag::Vel))
            .map(|x| x.id);

        Game::collide(physics_entities.clone(), &self.spacially_sorted, &self.positions, &self.velocities, &mut self.collision_buffer_pos, &mut self.collision_buffer_vel);


        for (i, j) in physics_entities.zip(&self.collision_buffer_pos) {
            self.spacially_sorted.sort_single(i, &j);
        }
        std::mem::swap(&mut self.positions, &mut self.collision_buffer_pos);
        std::mem::swap(&mut self.velocities, &mut self.collision_buffer_vel);
        

        let shown_entities = self.entities.iter()
            .filter(|x| x.components.contains(CompFlag::Pos | CompFlag::Ass))
            .map(|x| (self.assets[x.id as usize].clone(), self.positions[x.id as usize].clone()))
            .collect();
        let _ = wd_sender.try_send(shown_entities);

    }
}

