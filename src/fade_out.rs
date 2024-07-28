use bevy::{
    log,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::GameplaySet;

#[derive(Debug, Component)]
pub struct FadeOutThisEnt(pub Color);

#[derive(Debug, Component)]
struct FadeOut(Color, Timer);

pub fn fade_out_plugin(app: &mut App) {
    app.observe(on_fade_out_added)
        .add_systems(Update, fade_out_run.in_set(GameplaySet::After));
}

fn on_fade_out_added(
    trigger: Trigger<OnAdd, FadeOutThisEnt>,
    mut commands: Commands,

    fade_out: Query<(
        &Transform,
        &Mesh2dHandle,
        &Handle<ColorMaterial>,
        &FadeOutThisEnt,
    )>,
) {
    commands.entity(trigger.entity()).despawn_recursive();

    if let Ok((transform, mesh, mat, fade_out)) = fade_out.get(trigger.entity()) {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: mesh.clone(),
                material: mat.clone(),
                transform: *transform,
                ..default()
            },
            FadeOut(fade_out.0, Timer::from_seconds(0.5, TimerMode::Once)),
        ));
    } else {
        log::warn!("ent({}) is in fade out query", trigger.entity());
    }
}

fn fade_out_run(
    mut commands: Commands,
    mut fade_out: Query<(&Handle<ColorMaterial>, &mut FadeOut, Entity)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    for (mat, mut fade_out, ent) in fade_out.iter_mut() {
        fade_out.1.tick(time.delta());

        if fade_out.1.times_finished_this_tick() != 0 {
            commands.entity(ent).despawn_recursive();
        }

        if let Some(color) = materials.get_mut(mat.id()) {
            color.color = Hsva::from_vec3(
                Hsva::from(fade_out.0.to_srgba())
                    .to_vec3()
                    .lerp(Hsva::WHITE.to_vec3(), fade_out.1.elapsed_secs()),
            )
            .into();
        }
    }
}
