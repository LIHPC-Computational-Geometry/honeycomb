use bevy::prelude::*;
use bevy_egui::egui::Color32;

use crate::resources::{DartGizmos, EdgeGizmos};

/// Plugin handling rendering options.
pub struct OptionsPlugin;

impl Plugin for OptionsPlugin {
    fn build(&self, app: &mut App) {
        // render color
        app.insert_resource(DartRenderColor::default())
            .insert_resource(VertexRenderColor::default())
            .insert_resource(EdgeRenderColor::default())
            .insert_resource(FaceRenderColor::default())
            .insert_resource(VolumeRenderColor::default());
        // shrink
        app.insert_resource(DartShrink::default())
            .insert_resource(FaceShrink::default())
            .insert_resource(VolumeShrink::default());
        // width
        app.insert_resource(DartWidth(2.0))
            .insert_resource(VertexWidth(0.1))
            .insert_resource(EdgeWidth(1.0));
        // dart stuff
        app.insert_resource(DartHeadMul::default());
        // option update
        app.add_systems(
            Update,
            update_config
                .run_if(resource_changed::<DartWidth>.and_then(not(resource_added::<DartWidth>)))
                .run_if(
                    resource_changed::<VertexWidth>.and_then(not(resource_added::<VertexWidth>)),
                )
                .run_if(resource_changed::<EdgeWidth>.and_then(not(resource_added::<EdgeWidth>))),
        );
    }
}

pub fn update_config(
    mut config_store: ResMut<GizmoConfigStore>,
    dw: ResMut<DartWidth>,
    _v_width: ResMut<VertexWidth>,
    edw: ResMut<EdgeWidth>,
) {
    let (dart_config, _) = config_store.config_mut::<DartGizmos>();
    dart_config.line_width = dw.0;
    let (edge_config, _) = config_store.config_mut::<EdgeGizmos>();
    edge_config.line_width = edw.0;
}

macro_rules! declare_newtype_resource {
    ($name: ident, $inr: ty) => {
        /// Rendering option as a resource.
        #[derive(Resource)]
        pub struct $name(pub $inr);
    };
    ($name: ident, $inr: ty, $def: expr) => {
        /// Rendering option as a resource.
        #[derive(Resource)]
        pub struct $name(pub $inr);

        impl Default for $name {
            fn default() -> Self {
                Self($def)
            }
        }
    };
    ($name: ident, $inr1: ty, $inr2: ty, $def: expr) => {
        /// Rendering option as a resource.
        #[derive(Resource)]
        pub struct $name(pub $inr1, pub $inr2);

        impl Default for $name {
            fn default() -> Self {
                Self($def.0, $def.1)
            }
        }
    };
}

// --- NewType parameters
// fine granulation of parameters allow lighter rendering update logic

// -- indicate if objects of the given type should be rendered, & what color should be used

declare_newtype_resource!(DartRenderColor, bool, Color32, (true, Color32::BLACK));
#[rustfmt::skip]
declare_newtype_resource!(VertexRenderColor, bool, Color32, (true, Color32::GOLD));
declare_newtype_resource!(EdgeRenderColor, bool, Color32, (false, Color32::YELLOW));
declare_newtype_resource!(FaceRenderColor, bool, Color32, (false, Color32::RED));
declare_newtype_resource!(VolumeRenderColor, bool, Color32, (false, Color32::DARK_RED));

// -- shrink factor for objects of the given type; these are only relevant to a subset of types

declare_newtype_resource!(DartShrink, f32, -0.1);
declare_newtype_resource!(FaceShrink, f32, -0.2);
declare_newtype_resource!(VolumeShrink, f32, 0.0);

// -- size for objects of the given type; these are only relevant to a subset of types

declare_newtype_resource!(DartWidth, f32);
declare_newtype_resource!(VertexWidth, f32);
declare_newtype_resource!(EdgeWidth, f32);

// -- more specific options

declare_newtype_resource!(DartHeadMul, f32, 0.05);
