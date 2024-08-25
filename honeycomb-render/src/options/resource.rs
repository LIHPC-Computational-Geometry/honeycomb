use bevy::prelude::*;
use bevy_egui::egui::Color32;

macro_rules! declare_newtype_resource {
    ($nam: ident, $inr: ty) => {
        #[derive(Resource)]
        pub struct $nam(pub $inr);
    };
    ($nam: ident, $inr: ty, $def: expr) => {
        #[derive(Resource)]
        pub struct $nam(pub $inr);

        impl Default for $nam {
            fn default() -> Self {
                Self($def)
            }
        }
    };
    ($nam: ident, $inr1: ty, $inr2: ty, $def: expr) => {
        #[derive(Resource)]
        pub struct $nam(pub $inr1, pub $inr2);

        impl Default for $nam {
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
declare_newtype_resource!(BetaRenderColor, bool, Color32, (false, Color32::TRANSPARENT));
declare_newtype_resource!(VertexRenderColor, bool, Color32, (true, Color32::GOLD));
declare_newtype_resource!(EdgeRenderColor, bool, Color32, (false, Color32::YELLOW));
declare_newtype_resource!(FaceRenderColor, bool, Color32, (false, Color32::RED));
declare_newtype_resource!(VolumeRenderColor, bool, Color32, (false, Color32::DARK_RED));

// -- material handle for objects of the given type; those exist for efficiency reasons

declare_newtype_resource!(DartMatHandle, Handle<StandardMaterial>);
declare_newtype_resource!(BetaMatHandle, Handle<StandardMaterial>);
declare_newtype_resource!(VertexMatHandle, Handle<StandardMaterial>);
declare_newtype_resource!(EdgeMatHandle, Handle<StandardMaterial>);
declare_newtype_resource!(FaceMatHandle, Handle<StandardMaterial>);
declare_newtype_resource!(VolumeMatHandle, Handle<StandardMaterial>);

// -- shrink factor for objects of the given type; these are only relevant to a subset of types

declare_newtype_resource!(DartShrink, f32, 0.0);
declare_newtype_resource!(FaceShrink, f32, 0.0);
declare_newtype_resource!(VolumeShrink, f32, 0.0);

// -- size for objects of the given type; these are only relevant to a subset of types

declare_newtype_resource!(DartWidth, f32);
declare_newtype_resource!(BetaWidth, f32);
declare_newtype_resource!(VertexWidth, f32);
declare_newtype_resource!(EdgeWidth, f32);

// -- more specific options

declare_newtype_resource!(VertexHandle, Handle<Mesh>);
declare_newtype_resource!(DartHeadHandle, Handle<Mesh>);
declare_newtype_resource!(DartHeadMul, f32, 2.0);
