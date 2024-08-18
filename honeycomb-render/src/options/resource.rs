use bevy::prelude::*;

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
}

// -- if enabled, render objects of the given type

declare_newtype_resource!(DartRender, bool, true);
declare_newtype_resource!(BetaRender, bool, false);
declare_newtype_resource!(VertexRender, bool, true);
declare_newtype_resource!(EdgeRender, bool, false);
declare_newtype_resource!(FaceRender, bool, false);
declare_newtype_resource!(VolumeRender, bool, false);

// -- rendered color for objects of the given type
