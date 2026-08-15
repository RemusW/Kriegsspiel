use std::{
    collections::HashMap,
    default,
    marker::{self, PhantomData},
};

use macroquad::{prelude::*, texture, ui::widgets::Texture};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::World;

#[derive(Clone, Debug)]
pub struct Asset<T: Clone> {
    pub value: T,
    pub key: String,
    pub uuid: Uuid,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handle<T> {
    pub uuid: Uuid,
    pub name: String,
    #[serde(skip)]
    _marker: marker::PhantomData<T>,
}

impl<T: Clone + 'static> From<&Asset<T>> for Handle<T> {
    fn from(asset: &Asset<T>) -> Self {
        Handle {
            uuid: asset.uuid.clone(),
            name: asset.key.clone(),
            _marker: PhantomData,
        }
    }
}

struct PendingLoad {
    name: String,
    uuid: Uuid,
    path: String,
}

impl PendingLoad {
    pub fn new(name: &str, uuid: Uuid, path: &str) -> Self {
        PendingLoad {
            name: name.to_string(),
            uuid: uuid,
            path: path.to_string(),
        }
    }
}

pub struct AssetStore {
    textures: HashMap<String, Asset<Texture2D>>,
    pending: Vec<PendingLoad>,
    manifest: HashMap<String, String>,
}

impl AssetStore {
    pub fn get_manifest(&self) -> HashMap<String, String> {
        self.manifest.clone()
    }

    pub fn load_texture(&mut self, name: &str, path: &str) -> Handle<Texture2D> {
        let uuid = Uuid::new_v4();
        let handle: Handle<Texture2D> = Handle {
            name: name.to_string(),
            uuid: uuid,
            _marker: PhantomData,
        };
        let pending = PendingLoad::new(name, uuid, path);
        self.pending.push(pending);
        handle
    }

    pub fn get_texture(&self, name: &str) -> Texture2D {
        if let Some(t) = self.textures.get(name) {
            return t.value.clone();
        }
        self.textures["debug"].value.clone()
    }

    pub fn get_asset_texture(&self, key: &str) -> Asset<Texture2D> {
        if let Some(tex_asset) = self.textures.get(key) {
            return tex_asset.clone();
        } else {
            return self.textures["debug"].clone();
        }
    }
    // pub fn get_asset_path<T>(&self, key: &str) -> String {
    //     self.registry[key].clone()
    // }

    pub fn get_handle_from(&self, name: &str) -> Handle<Texture2D> {
        self.get_asset_texture(name);
        Handle::from(self.textures.get(name).unwrap())
    }

    pub async fn process_pending(&mut self) {
        let pending = std::mem::take(&mut self.pending);
        for p in pending {
            let tex = load_texture(&p.path).await.unwrap();
            let asset: Asset<Texture2D> = Asset {
                value: tex.clone(),
                uuid: Uuid::new_v4(),
                key: p.name.to_string(),
                source_path: p.path.to_string(),
            };
            self.manifest
                .insert(p.name.clone(), asset.source_path.clone());
            self.textures.insert(p.name.to_string(), asset);
        }
    }

    pub fn load_manifest(&mut self, manifest: HashMap<String, String>) {
        for (name, path) in manifest.iter() {
            self.load_texture(name, path);
            self.manifest.insert(name.to_string(), path.to_string());
        }
    }
}

impl Default for AssetStore {
    fn default() -> Self {
        let mut asset_store = Self {
            textures: HashMap::new(),
            pending: Vec::new(),
            manifest: HashMap::new(),
        };
        asset_store.load_texture("debug", "assets/test_uv.png");
        asset_store
    }
}
