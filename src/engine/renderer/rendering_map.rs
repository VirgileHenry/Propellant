use std::collections::{HashMap, BTreeMap};

use crate::{ProppellantResources, engine::{resources::mesh_library::LoadedMesh, consts::PROPELLANT_DEBUG_FEATURES}};

/// A rendering map builds the rendering commands from the scene.
pub struct RenderingMap {
    /// key : mesh id
    /// value : (instance count, map: entity id -> instance id)
    pub map: BTreeMap<u64, (u32, HashMap<usize, usize>)>,
}

impl RenderingMap {
    pub fn new() -> RenderingMap {
        RenderingMap {
            map: BTreeMap::new(),
        }
    }

    /// Creates a iterator over the meshes and the instances to draw.
    /// With the provided resources, zip the mesh id to the mesh and filters the non existing meshes.
    pub fn iter<'a>(&'a self, resources: &'a ProppellantResources) -> impl Iterator<Item = (&'a LoadedMesh, u32)> + '_ {
        self.map.iter().map(|(k, v)| {
            let mesh = resources.meshes().loaded_mesh(k);
            if PROPELLANT_DEBUG_FEATURES {
                if mesh.is_none() {
                    println!("[PROPELLANT DEBUG] Mesh not in mesh library (id {})", k);
                }
            }
            (mesh, v.0)
        }).filter(|(m, _)| m.is_some())
        .map(|(m, c)| (m.unwrap(), c))
    }

    /// Clears the rendering map, but keep the allocated memory for each hashmap.
    pub fn clear(&mut self) {
        for (_, (instance_count, map)) in self.map.iter_mut() {
            map.clear();
            *instance_count = 0;
        }
    }

    /// Adds an instance to the rendering map.
    pub fn add_instance(&mut self, mesh_id: u64, entity_id: usize) {
        let (instance_count, map) = self.map.entry(mesh_id).or_insert((0, HashMap::new()));
        map.insert(entity_id, *instance_count as usize);
        *instance_count += 1;
    }

    /// Get the buffer index of an entity.
    pub fn get_buffer_index(&self, mesh_id: u64, entity_id: usize) -> Option<usize> {
        self.map.get(&mesh_id).and_then(|(_, map)| map.get(&entity_id).copied())
    }

    /// Add offsets to the rendering map.
    /// This convert the maps from mapping to instance id for each mesh to mapping to buffer offset for each mesh.
    pub fn add_offsets(&mut self) {
        let mut offset = 0;
        for (_, (instance_count, map)) in self.map.iter_mut() {
            for (_, instance_id) in map.iter_mut() {
                *instance_id += offset;
            }
            offset += *instance_count as usize;
        }
    }

    /// Get the number of objects in the scene rendered by this pipeline.
    pub fn object_count(&self) -> usize {
        self.map.iter().map(|(_, (instance_count, _))| *instance_count as usize).sum()
    }

}

