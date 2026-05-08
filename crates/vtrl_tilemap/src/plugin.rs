use std::sync::Arc;

use vtrl_common::prelude::*;
use vtrl_ecs::prelude::*;
use vtrl_render::prelude::*;
use vtrl_scene::SceneManager;

use crate::atlas::*;
use crate::renderer::*;
use crate::tilemap::*;

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, world: &mut World, _: &mut AssetManager) {
        world.add_resource(TileAtlas::default());
        world.add_resource(TilemapRenderer::default());

        world.add_system(ScheduleSlot::PreRender, |w, mgr| {
            // Drain freshly-loaded tilesets from the scene manager
            let mut loaded = match w.get_resource_mut::<SceneManager>() {
                Some(mut s) if !s.just_loaded.is_empty() => std::mem::take(&mut s.just_loaded),
                _ => return,
            };

            {
                let mut atlas = w.get_resource_mut::<TileAtlas>().unwrap();
                loaded.retain(|(ty, sym)| {
                    if ty == "TileSet" {
                        if let Some(set) = mgr.get::<TileSet>(*sym) {
                            let _ = atlas.push(AssetHandle::from(*sym), set.clone());
                        }
                        false
                    } else {
                        true
                    }
                })
            }

            // Put back anything we didn't drain so other consumers can.
            if !loaded.is_empty() {
                w.get_resource_mut::<SceneManager>()
                    .unwrap()
                    .just_loaded
                    .append(&mut loaded);
            }
        });

        world.add_system(ScheduleSlot::Render, |w, mgr| {
            let mut cb = w.get_resource_mut::<CommandBuffer>().unwrap();
            let atlas = w.get_resource::<TileAtlas>().unwrap();

            let viewport = w.get_resource_mut::<Viewport>().unwrap();
            let view_projection = w.view::<(Camera, Transform), ()> ()
                .iter()
                .find(|(_, (cam, _))| cam.primary)
                .map(|(_, (cam, xform))| {
                    cam.view_projection(xform.position, xform.rotation, Vec2::new(viewport.width as f32, viewport.height as f32))
                });

            let view = w.view::<TileMap, ()>();
            let background_tilemap = view
                .iter()
                .find(|(_, tm)| tm.layer == TileLayer::Background);

            if let Some((entity, tm)) = background_tilemap {
                let matrix = view_projection.unwrap();
                let offset = match w.get_component::<Transform>(entity) {
                    Some(xform) => xform.position,
                    None => Vec2::zero(),
                };
                let tileset = mgr.get::<TileSet>(tm.tileset).unwrap();
                let tile_size = tileset.tile_size;
                let row_count = tileset.row_count;
                let column_count = tileset.column_count;
                let atlas_layer = atlas.get_texture_id(tm.tileset).unwrap_or(&0).to_owned();

                let mut instances: Vec<TileInstance> = Vec::new();
                for (index, tile) in tm.grid.rows.iter().enumerate() {
                    let y = index as u32 / tm.grid.width;
                    let x = index as u32 % tm.grid.width;
                    instances.push(TileInstance {
                        grid_position: Vec2::new(x as f32, y as f32),
                        tile_id: *tile,
                    });
                }

                cb.push(RenderCommand::BeginPass {
                    name: "tile-background",
                    target: RenderTarget::Screen,
                    clear: None,
                    blend_mode: Some(BlendMode::Alpha),
                    view_projection,
                });


                let command = move |w: &World| {
                    let r = w.get_resource::<TilemapRenderer>().unwrap();
                    let tile_atlas = w.get_resource::<TileAtlas>().unwrap();
                    r.draw_tiles(
                        matrix,
                        offset,
                        tile_size as f32,
                        column_count,
                        row_count,
                        atlas_layer as f32,
                        &tile_atlas,
                        &instances,
                    );
                };
                cb.push(RenderCommand::Complex(Arc::new(command)));
            }

            let view = w.view::<TileMap, ()>();
            let foreground_tilemap = view
                .iter()
                .find(|(_, tm)| tm.layer == TileLayer::Foreground);

            if let Some((entity, tm)) = foreground_tilemap {
                let matrix = view_projection.unwrap();
                let offset = match w.get_component::<Transform>(entity) {
                    Some(xform) => xform.position,
                    None => Vec2::zero(),
                };
                let tileset = mgr.get::<TileSet>(tm.tileset).unwrap();
                let tile_size = tileset.tile_size;
                let row_count = tileset.row_count;
                let column_count = tileset.column_count;

                let atlas_layer = atlas.get_texture_id(tm.tileset).unwrap_or(&0).to_owned();

                let mut instances: Vec<TileInstance> = Vec::new();
                for (index, tile) in tm.grid.rows.iter().enumerate() {
                    let y = index as u32 / tm.grid.width;
                    let x = index as u32 % tm.grid.width;
                    instances.push(TileInstance {
                        grid_position: Vec2::new(x as f32, y as f32),
                        tile_id: *tile,
                    });
                }

                cb.push(RenderCommand::BeginPass {
                    name: "tile-foreground",
                    target: RenderTarget::Screen,
                    clear: None,
                    blend_mode: Some(BlendMode::Alpha),
                    view_projection,
                });

                let command = move |w: &World| {
                    let r = w.get_resource::<TilemapRenderer>().unwrap();
                    let tile_atlas = w.get_resource::<TileAtlas>().unwrap();
                    r.draw_tiles(
                        matrix,
                        offset,
                        tile_size as f32,
                        column_count,
                        row_count,
                        atlas_layer as f32,
                        &tile_atlas,
                        &instances,
                    );
                };
                cb.push(RenderCommand::Complex(Arc::new(command)));
            }
        });
    }
}
