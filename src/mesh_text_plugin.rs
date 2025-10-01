use crate::text_glyphs::TextGlyphs;
use crate::{HorizontalAnchorPoint, InputText, MeshTextError, VerticalAlign, VerticalAnchorPoint};
use crate::{MeshTextEntry, Parameters};
use bevy::prelude::*;
use cosmic_text::{FontSystem, Metrics};

pub struct MeshTextPlugin;

impl Plugin for MeshTextPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings {
            font_system: FontSystem::new(),
        });
    }
}

pub fn generate_meshes<M: Asset>(
    text: InputText<M>,
    fonts: &mut ResMut<Settings>,
    params: Parameters,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Result<(Vec<MeshTextEntry<M>>, Vec2), MeshTextError> {
    if !text.is_valid() {
        error!("Invalid text input");
        return Err(MeshTextError::InvalidInput);
    }

    let (materials, spans, default_attrs) = match text {
        InputText::Simple {
            material,
            ref text,
            attrs,
        } => (vec![material], vec![(text.as_str(), attrs.clone())], attrs),
        InputText::Rich {
            materials,
            ref words,
            attrs,
        } => (
            materials,
            words
                .iter()
                .map(|w| w.as_str())
                .zip(attrs.iter())
                .enumerate()
                .map(|(i, (word, attr))| (word, attr.clone().metadata(i)))
                .collect(),
            attrs[0].clone(),
        ),
    };

    let default_metrics = Metrics {
        font_size: params.font_size,
        line_height: params.line_height,
    };

    let mut tx = TextGlyphs::new(
        default_metrics,
        spans,
        &default_attrs,
        &mut fonts.font_system,
        params.alignment,
    );
    let (width, height) = tx.measure(params.max_width, params.max_height, &mut fonts.font_system);
    let processed_glyphs = tx.generate_mesh_glyphs(
        &mut fonts.font_system,
        params.extrusion_depth,
        meshes,
        &materials,
    );

    let mut meshes = Vec::new();

    for glyph_data in processed_glyphs {
        // Calculate the target world position for the glyph's visual center.
        // glyph_data.x, .y, .x_offset, .y_offset, .line_y are from CosmicText layout.
        // glyph_data.glyph_center_x_layout and .glyph_center_y_layout are the offsets
        // from the glyph's own layout origin to its visual center, in layout units.
        // (Layout units are scaled by font_size relative to font design units).

        let target_center_x_layout_units =
            glyph_data.x + glyph_data.x_offset + glyph_data.glyph_center_x_layout;

        // Calculate the Y position for the glyph's visual center in Bevy's Y-up world space.
        // 1. Sum CosmicText's Y-down layout components:
        //    line_y: baseline position (Y increases downwards from top of text buffer).
        //    glyph.y: glyph's offset from baseline (Y increases downwards if positive).
        //    glyph.y_offset: additional Y offset in the same system.
        let sum_y_components_layout_down = glyph_data.line_y + glyph_data.y + glyph_data.y_offset;

        // 2. Convert the sum to a Y-up Bevy coordinate. If CosmicText Y=0 (top) is Bevy Y=H,
        //    and CosmicText Y=H (bottom) is Bevy Y=0, this would be (H_text_block - sum_y_components_layout_down).
        //    Simpler: if mapping Cosmic Y=0 to Bevy Y=0 and flipping axis: Bevy_Y_up = -Cosmic_Y_down.
        let glyph_origin_y_layout_bevy_up = -sum_y_components_layout_down;

        // 3. Add the glyph's intrinsic Y-up center offset.
        //    glyph_data.glyph_center_y_layout is the Y-up distance from the glyph's font origin to its visual center.
        let target_center_y_layout_units_bevy_up =
            glyph_origin_y_layout_bevy_up + glyph_data.glyph_center_y_layout;
        let aligned_center_y_units = match (params.max_height, params.vertical_alignment) {
            (_, Some(VerticalAlign::Top)) | (_, None) | (None, _) => {
                target_center_y_layout_units_bevy_up
            }
            (Some(max_height), Some(VerticalAlign::Middle)) => {
                target_center_y_layout_units_bevy_up - ((max_height - height) / 2.0)
            }
            (Some(max_height), Some(VerticalAlign::Bottom)) => {
                target_center_y_layout_units_bevy_up - (max_height - height)
            }
        };
        let width = params.max_width.unwrap_or(width);
        let height = params.max_height.unwrap_or(height);
        let anchored_x = match params.horizontal_anchor_point {
            HorizontalAnchorPoint::Left => target_center_x_layout_units,
            HorizontalAnchorPoint::Middle => target_center_x_layout_units - (width / 2.0),
            HorizontalAnchorPoint::Right => target_center_x_layout_units - width,
        };
        let anchored_y = match params.vertical_anchor_point {
            VerticalAnchorPoint::Top => aligned_center_y_units,
            VerticalAnchorPoint::Middle => aligned_center_y_units + (height / 2.0),
            VerticalAnchorPoint::Bottom => aligned_center_y_units + height,
        };
        let world_x = anchored_x;
        let world_y = anchored_y; // Use the new Y-up calculation

        meshes.push(MeshTextEntry {
            mesh: glyph_data.mesh,
            material: glyph_data.material,
            transform: Transform::from_xyz(world_x, world_y, 0.0),
        });
    }

    Ok((meshes, vec2(width, height)))
}

#[derive(Resource)]
pub struct Settings {
    pub font_system: FontSystem,
}
