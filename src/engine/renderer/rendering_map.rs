use std::collections::BTreeMap;

use crate::{
    PropellantResources,
    engine::{
        resources::mesh_library::LoadedMesh,
        consts::PROPELLANT_DEBUG_FEATURES
    }
};

pub(crate) mod hasher;

/// A rendering map builds the rendering commands from the scene.
pub struct RenderingMap {
    /// key : mesh id
    /// value : number of instances, total offset, temp counter
    pub map: BTreeMap<u64, (usize, usize, usize)>,
}

impl RenderingMap {
    pub fn new() -> RenderingMap {
        RenderingMap {
            map: BTreeMap::new(),
        }
    }

    /// Creates a iterator over the meshes and the instances to draw.
    /// With the provided resources, zip the mesh id to the mesh and filters the non existing meshes.
    pub fn iter<'a>(&'a self, resources: &'a PropellantResources) -> impl Iterator<Item = (&'a LoadedMesh, usize)> + '_ {
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

    pub fn map_mut(&mut self) -> &mut BTreeMap<u64, (usize, usize, usize)> {
        &mut self.map
    }

    /// Get the number of objects in the scene rendered by this pipeline.
    pub fn object_count(&self) -> usize {
        self.map.iter().map(|(_, (instance_count, _, _))| *instance_count).sum()
    }

}

