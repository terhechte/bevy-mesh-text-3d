use bevy::prelude::*;
use cosmic_text::{Attrs, Metrics, Style, Weight};

use bevy_mesh_text_3d::{
    HorizontalAnchorPoint, InputText, MeshTextPlugin, Parameters, Settings, VerticalAnchorPoint,
    generate_meshes,
};

// Rotation speed in radians per second
const ROTATION_SPEED: f32 = 2.0;

// Constants for MovingText
const MOVE_AMPLITUDE: f32 = 0.3; // Max displacement from the center y-position
const MOVE_PERIOD_SECONDS: f32 = 3.0; // How long one full up-and-down cycle takes
const MOVE_FREQUENCY: f32 = 2.0 * std::f32::consts::PI / MOVE_PERIOD_SECONDS; // Angular frequency for sine wave

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MeshTextPlugin)
        .add_systems(Update, keyboard_input)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 900.,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Startup, spawn_text)
        .add_systems(Update, rotate_text) // Add the rotation system
        .add_systems(Update, move_text_vertically) // Add the vertical movement system
        .run();
}

fn keyboard_input(keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) {
        std::process::exit(0);
    }
}

/// Component to mark text that should rotate
#[derive(Component)]
pub struct RotatingText {
    pub speed: f32,
}

impl Default for RotatingText {
    fn default() -> Self {
        Self {
            speed: ROTATION_SPEED,
        }
    }
}

/// Component to mark text that should move vertically and control its movement.
#[derive(Component)]
pub struct MovingText {
    amplitude: f32,
    frequency: f32,
    initial_y_center: f32, // The y-coordinate around which the text oscillates
    phase_offset: f32,     // Phase offset in radians to adjust starting position in cycle
}

impl MovingText {
    /// Creates a new `MovingText` component.
    ///
    /// # Arguments
    /// * `initial_y_offset`: The desired starting offset from `initial_y_center`.
    ///   A value of 0.0 starts at the center. `MOVE_AMPLITUDE` starts at the top.
    ///   Values are clamped to `[-MOVE_AMPLITUDE, MOVE_AMPLITUDE]`.
    /// * `initial_y_center`: The central y-coordinate around which the text will oscillate.
    pub fn new(initial_y_offset: f32, initial_y_center: f32) -> Self {
        // Normalize the initial offset to [-1, 1] range relative to amplitude
        let clamped_offset_ratio = (initial_y_offset / MOVE_AMPLITUDE).clamp(-1.0, 1.0);
        // Calculate phase offset using arcsin to start at the desired point in the sine wave
        Self {
            amplitude: MOVE_AMPLITUDE,
            frequency: MOVE_FREQUENCY,
            initial_y_center,
            phase_offset: clamped_offset_ratio.asin(),
        }
    }
}

fn rotate_text(mut query: Query<(&mut Transform, &RotatingText)>, time: Res<Time>) {
    for (mut transform, rotating) in &mut query {
        // Rotate around the Y axis
        transform.rotate_y(rotating.speed * time.delta_secs());
    }
}

/// System that moves text entities with `MovingText` component vertically.
fn move_text_vertically(
    mut query: Query<(&mut Transform, &MovingText)>,
    time: Res<Time>, // Access global time resource
) {
    for (mut transform, moving_text) in &mut query {
        // Calculate current time elapsed since app start
        let current_time = time.elapsed().as_secs_f32();
        // Calculate vertical displacement using a sine wave
        let displacement = moving_text.amplitude
            * (moving_text.frequency * current_time + moving_text.phase_offset).sin();
        // Update the y-coordinate of the transform
        transform.translation.y = moving_text.initial_y_center + displacement;
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_text(
    mut commands: Commands,
    mut fonts: ResMut<Settings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let default_metrics = Metrics {
        font_size: 0.14,
        line_height: 0.16,
    };

    let blue_material = materials.add(StandardMaterial {
        base_color: Color::Srgba(Srgba::BLUE),
        cull_mode: None,
        ..default()
    });

    let red_material = materials.add(StandardMaterial {
        base_color: Color::Srgba(Srgba::RED),
        cull_mode: None,
        ..default()
    });

    let attrs1 = Attrs::new().weight(Weight::BLACK);

    let attrs2 = Attrs::new().style(Style::Italic);

    let (meshes, _) = generate_meshes(
        InputText::Rich {
            words: vec!["Hello".to_string(), "World".to_string()],
            materials: vec![blue_material, red_material],
            attrs: vec![attrs1, attrs2],
        },
        &mut fonts,
        Parameters {
            extrusion_depth: 0.03,
            font_size: default_metrics.font_size,
            line_height: default_metrics.line_height,
            alignment: None,
            max_width: None,
            max_height: None,
            vertical_alignment: None,
            horizontal_anchor_point: HorizontalAnchorPoint::Middle,
            vertical_anchor_point: VerticalAnchorPoint::Middle,
        },
        &mut meshes,
    )
    .unwrap();

    let mut idx = 1;
    for mesh in meshes {
        // Calculate the final spawn transform first, including any horizontal adjustments
        let spawn_transform = mesh.transform.with_translation(Vec3::new(
            mesh.transform.translation.x, // Apply horizontal shift
            mesh.transform.translation.y,          // This y is the base for vertical oscillation
            mesh.transform.translation.z,
        ));

        // Determine the center y-coordinate for the MovingText component based on the spawn position
        let initial_y_center_for_movement = spawn_transform.translation.y;

        // Set the desired initial vertical offset from the center y-coordinate.
        // For example:
        // 0.0 will start the oscillation at the central y-position.
        // `MOVE_AMPLITUDE` will start at the top of the oscillation.
        // `-MOVE_AMPLITUDE` will start at the bottom.
        // `25.0` would start 25 units above the center (if `MOVE_AMPLITUDE` is >= 25).
        let desired_initial_y_offset = idx as f32 * 0.05;

        commands.spawn((
            Mesh3d(mesh.mesh),
            MeshMaterial3d(mesh.material),
            spawn_transform,
            RotatingText::default(),
            // Add the MovingText component, initialized with the desired offset and center
            MovingText::new(desired_initial_y_offset, initial_y_center_for_movement),
        ));
        idx += 1;
    }
}
