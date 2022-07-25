use bevy::prelude::*;

use crate::ClientGameState;

pub struct CustomUIPlugin;

impl Plugin for CustomUIPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(setup_ui)
            .add_startup_system(setup_button)
            .add_system(update_button);
    }
}

fn setup_ui(mut com: Commands) {
    com.spawn_bundle(UiCameraBundle::default());
}

struct SubmitButtonMarker;
pub fn setup_button(mut com: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    com.spawn_bundle(UiCameraBundle::default());
    com.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Px(150.), Val::Px(75.)),
            position: Rect {
                left: Val::Px(1500.),
                bottom: Val::Px(100.),
                ..Default::default()
            },
            ..Default::default()
        },
        material: materials.add(Color::rgba(0.65, 0.65, 0.65, 0.5).into()),
        ..Default::default()
    })
    .insert(SubmitButtonMarker);
}

fn update_button(
    mut old_state: Local<ClientGameState>,
    state: Res<State<ClientGameState>>,
    mut query: Query<&mut Handle<ColorMaterial>, With<SubmitButtonMarker>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if &*old_state != state.current() {
        *old_state = state.current().clone();

        //change button color accordingly
        let mut handle = query.single_mut().unwrap();
        *handle = match *old_state {
            ClientGameState::Waiting => materials.add(Color::rgba(0.65, 0.1, 0.1, 0.5).into()),
            ClientGameState::Playing => materials.add(Color::rgba(0.1, 0.4, 0.2, 0.5).into()),
            ClientGameState::Loading => materials.add(Color::rgba(0.1, 0.2, 0.6, 0.5).into()),
            _ => materials.add(Color::rgba(0.6, 0.6, 0.2, 0.5).into()),
        }

    }
}
