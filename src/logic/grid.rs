use std::{cell::{Ref, RefCell}, collections::HashMap};
use super::{EntityId, Position};

pub struct EntityGrid {
    sorted : HashMap<(usize, usize), RefCell<Vec<EntityId>>>,
    locations : Vec<(usize, usize)>,
    scale_factor : f32,
}

impl EntityGrid {
    pub fn new(scale_factor : f32) -> Self {
        Self {
            scale_factor,
            sorted : HashMap::new(),
            locations : Vec::new(),
        }
    }

    fn get_location(&self, pos : &Position) -> (usize, usize) {
        ((pos.x/self.scale_factor) as usize, (pos.y/self.scale_factor) as usize)
    }

    fn fill_loc(&mut self, id : EntityId, loc : &(usize,usize)) {
        if let None = self.sorted.get(loc) {
            self.sorted.insert(*loc, RefCell::new(vec![id]));
        } else {
            self.sorted.get(loc).unwrap().borrow_mut().push(id);
        }
        if self.locations.len() <= id as usize {
            self.locations.push(*loc);
        } else {
            self.locations[id as usize] = *loc;
        }
    }

    /// The iterator given has to contain all positions.
    /// TODO: Deal with unpositioned entities
    pub fn sort<'a>(&mut self, positions : impl Iterator<Item = &'a Position>) {
        
        for (i, pos) in positions.enumerate() {
            let loc = self.get_location(pos);

            self.fill_loc(i as EntityId, &loc);
        }
    }

    pub fn sort_single(&mut self, id : EntityId, pos : &Position) {
        let loc = self.get_location(pos);
        if let Some(k) = self.locations.get(id as usize) {
            if *k != loc {
                {
                    let l = self.sorted.get(k).unwrap();
                    let mut l = l.borrow_mut();
                    let index = *(l.iter().find(|f| **f == id).unwrap());
                    l.remove(index as usize);
                }

                self.fill_loc(id, &loc);
            }
        }
    }

    pub fn find_nearby(&self, pos :  &Position) -> Option<Ref<Vec<u16>>> {
        let loc = self.get_location(pos);
        let result = self.sorted.get(&loc);
        match result {
            None => None,
            Some(rc) => Some(rc.borrow())
        }
    }
}