use bevy::prelude::*;

pub mod resource;
pub mod tab;

pub struct OptionsPlugin;

impl Plugin for OptionsPlugin {
    fn build(&self, app: &mut App) {
        // render color
        app.insert_resource(resource::DartRenderColor::default())
            .insert_resource(resource::BetaRenderColor::default())
            .insert_resource(resource::VertexRenderColor::default())
            .insert_resource(resource::EdgeRenderColor::default())
            .insert_resource(resource::FaceRenderColor::default())
            .insert_resource(resource::VolumeRenderColor::default());
        // mat handle
        // ...
        // shrink
        app.insert_resource(resource::DartShrink(-0.2))
            .insert_resource(resource::FaceShrink::default())
            .insert_resource(resource::VolumeShrink::default());
        // width
        app.insert_resource(resource::DartWidth(0.05))
            .insert_resource(resource::BetaWidth(0.05))
            .insert_resource(resource::VertexWidth(0.075))
            .insert_resource(resource::EdgeWidth(0.05));
        // dart stuff
        app.insert_resource(resource::DartHeadMul::default());
    }
}
