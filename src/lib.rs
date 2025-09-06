use bevy::{
    asset::{Asset, Handle},
    render::mesh::Mesh,
    transform::components::Transform,
};
pub use cosmic_text::{
    Align, Attrs, CacheKeyFlags, CacheMetrics, Color, Family, Feature, FeatureTag, FontFeatures,
    LetterSpacing, Stretch, Style, Weight, fontdb::ID,
};

pub mod command_encoder;
pub mod extrude_glyph;
pub mod mesh_text_plugin;
pub mod text_glyphs;

pub use mesh_text_plugin::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MeshTextError {
    #[error("Tessellation process failed")]
    TessellationFailed,

    #[error("Path Building Failed")]
    PathBuildingFailed,

    #[error("The input provided was invalid")]
    InvalidInput,
}

/// A extruded glyph mesh.
#[derive(Debug)]
pub struct MeshGlyph<M: Asset> {
    pub glyph_id: u16,
    pub font_id: Option<ID>,
    pub x: f32,
    pub y: f32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub line_y: f32,
    pub glyph_center_x_layout: f32,
    pub glyph_center_y_layout: f32,
    pub height: f32,
    pub mesh: Handle<Mesh>,
    pub material: Handle<M>,
}

/// The text input for glyph mesh generation.
pub enum InputText<'a, M: Asset> {
    /// A simple text with a string and a single material
    Simple {
        text: String,
        material: Handle<M>,
        attrs: Attrs<'a>,
    },
    /// A rich text with a vector of words and materials.
    /// The three Vecs must be the same length.
    Rich {
        words: Vec<String>,
        materials: Vec<Handle<M>>,
        attrs: Vec<Attrs<'a>>,
    },
}

impl<M: Asset> InputText<'_, M> {
    pub fn is_valid(&self) -> bool {
        match self {
            InputText::Simple { text, .. } => !text.is_empty(),
            InputText::Rich {
                words,
                materials,
                attrs,
            } => !words.is_empty() && words.len() == materials.len() && attrs.len() == words.len(),
        }
    }
}

/// A extruded glyph mesh including the transform and material.
pub struct MeshTextEntry<M: Asset> {
    /// The mesh handle of the glyph. Similar glyphes share the same mesh handle
    pub mesh: Handle<Mesh>,
    /// The transform for this glyph in the provided sentence / text
    pub transform: Transform,
    /// The material of this glyph
    pub material: Handle<M>,
}

/// Describes the anchor point relative to the text
#[derive(Clone, Copy,Debug)]
pub enum VerticalAnchorPoint {
    Top,
    Middle,
    Bottom,
}
/// Describes the anchor point relative to the text
#[derive(Clone, Copy,Debug)]
pub enum HorizontalAnchorPoint {
    Left,
    Middle,
    Right,
}
#[derive(Clone, Copy,Debug)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

pub struct Parameters {
    /// Extrusion depth
    pub extrusion_depth: f32,
    /// Font size
    pub font_size: f32,
    /// Line height
    pub line_height: f32,
    /// Alignment
    pub alignment: Option<Align>,
    /// Vertical Alignment
    pub vertical_alignment: Option<VerticalAlign>,
    /// Describes the anchor point relative to the text
    pub horizontal_anchor_point: HorizontalAnchorPoint,
    /// Describes the anchor point relative to the text
    pub vertical_anchor_point: VerticalAnchorPoint,
    /// Maximum width of the textbox. Beyond this width, the text will wrap.
    pub max_width: Option<f32>,
    /// Maximum height of the textbox.
    pub max_height: Option<f32>,
}
