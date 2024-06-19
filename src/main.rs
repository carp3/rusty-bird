use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
struct Background;

#[derive(Component)]
struct Base;

#[derive(Component)]
struct LowerPipes;

#[derive(Component)]
struct UpperPipes;

#[derive(Component)]
struct PressSpace;

#[derive(Component)]
struct ScoreDisplay;

#[derive(Resource, Default)]
struct Score {
    value: u32,
}

#[derive(Resource, Default)]
struct GameState {
    game_over: bool,
    first_start: bool,
}

#[derive(Component)]
struct GameOverDisplay;

#[derive(Component)]
struct BirdAnimationIndices {
    first: usize,
    last: usize,
    speed: f64,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct GravityTimer(Timer);

#[derive(Event)]
struct GameOverEvent();

const GRAVITY: f64 = -9.81 * 4.;

const WINDOW_Y: f32 = 512.;
const WINDOW_X: f32 = 800.;

fn main() {
    App::new()
        .init_resource::<Score>()
        .init_resource::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(Update, display_score)
        .add_systems(Update, jump)
        .add_systems(Update, animate_bird.run_if(game_is_active))
        .add_systems(Update, animate_press_space.run_if(game_is_not_active))
        .add_systems(Update, physics.run_if(game_is_active))
        .add_systems(Update, move_bg.run_if(game_is_active))
        .add_systems(Update, move_base.run_if(game_is_active))
        .add_systems(Update, move_pipes_and_game_logic.run_if(game_is_active))
        .add_event::<GameOverEvent>()
        .add_systems(Update, game_over_event)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rusty Bird".to_string(),
                resolution: (WINDOW_X, WINDOW_Y).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut game_over: ResMut<GameState>,
) {
    game_over.game_over = true;
    game_over.first_start = true;
    let background_image = asset_server.load("sprites/background-day.png");
    let number_image = asset_server.load("sprites/numbers.png");
    let base_image = asset_server.load("sprites/base.png");
    let game_over_image = asset_server.load("sprites/game-over.png");
    let pipe = asset_server.load("sprites/pipe.png");
    let bird = asset_server.load("sprites/bluebird2.png");
    let space = asset_server.load("sprites/space.png");

    let pos = Vec3::new(0., 0., 0.);
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            texture: background_image,
            transform: Transform::from_translation(pos),
            sprite: Sprite {
                custom_size: Some(Vec2::new(1400.0, 512.0)),
                ..default()
            },
            ..default()
        },
        ImageScaleMode::Tiled {
            tile_x: true,
            tile_y: false,
            stretch_value: 1.,
        },
        Background,
    ));

    commands.spawn((
        SpriteBundle {
            texture: base_image,
            transform: Transform::from_xyz(-0., -230., 5.),
            sprite: Sprite {
                custom_size: Some(Vec2::new(1400.0, 112.0)),
                ..default()
            },
            ..Default::default()
        },
        ImageScaleMode::Tiled {
            tile_x: true,
            tile_y: false,
            stretch_value: 1.,
        },
        Base,
    ));

    commands.spawn((
        SpriteBundle {
            texture: game_over_image,
            transform: Transform::from_xyz(0., 0., 10.),
            visibility: Visibility::Hidden,
            ..default()
        },
        GameOverDisplay,
    ));

    commands.spawn((
        SpriteBundle {
            texture: space,
            transform: Transform::from_xyz(0., -50., 10.),
            ..default()
        },
        PressSpace,
        AnimationTimer(Timer::from_seconds(0.75, TimerMode::Repeating)),
    ));

    let mut x = -250.;
    for _i in 0..=4 {
        commands.spawn((
            SpriteBundle {
                texture: number_image.clone(),
                transform: Transform::from_xyz(x, 220., 10.),
                sprite: Sprite {
                    rect: Some(Rect::new(0., 0., 24., 36.)),
                    ..default()
                },
                ..Default::default()
            },
            ScoreDisplay,
        ));

        x -= 26.;
    }

    let layout = TextureAtlasLayout::from_grid(Vec2::new(34.0, 24.0), 3, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_indices = BirdAnimationIndices {
        first: 0,
        last: 2,
        speed: 0.,
    };
    commands.spawn((
        SpriteSheetBundle {
            texture: bird,
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            },
            transform: Transform::from_xyz(0., 0., 4.),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.15, TimerMode::Repeating)),
        GravityTimer(Timer::from_seconds(0.02, TimerMode::Repeating)),
    ));

    x = 300.;
    for _ in 1..=5 {
        let mut transform = Transform::from_xyz(x, -100., 3.);
        commands.spawn((
            SpriteBundle {
                texture: pipe.clone(),
                transform: transform.clone(),
                ..default()
            },
            LowerPipes,
        ));

        transform.rotate_local_x(std::f32::consts::PI);
        transform.translation.y += 450.;
        commands.spawn((
            SpriteBundle {
                texture: pipe.clone(),
                transform,
                ..default()
            },
            UpperPipes,
        ));
        x += 200.;
    }
}

fn game_is_active(game_over: Res<GameState>) -> bool {
    return !game_over.game_over;
}

fn game_is_not_active(game_over: Res<GameState>) -> bool {
    return game_over.game_over;
}

fn animate_bird(
    time: Res<Time>,
    mut query: Query<(
        &mut BirdAnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlas,
    )>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}

fn animate_press_space(
    time: Res<Time>,
    mut query: Query<(&mut Visibility, &mut AnimationTimer), With<PressSpace>>,
) {
    for (mut vis, mut timer) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            if matches!(*vis, Visibility::Visible) {
                *vis = Visibility::Hidden;
            } else {
                *vis = Visibility::Visible;
            }
        }
    }
}

fn physics(
    time: Res<Time>,
    mut query: Query<(&mut BirdAnimationIndices, &mut GravityTimer, &mut Transform)>,
    mut ev_game_over: EventWriter<GameOverEvent>,
) {
    for (mut indices, mut timer, mut transform) in &mut query {
        let delta = time.delta();
        timer.tick(delta);
        if timer.just_finished() {
            let t = delta.as_secs_f64();
            transform.translation.y += (indices.speed + (0.5 * GRAVITY * t * t)) as f32;
            indices.speed += GRAVITY * t;
            transform.rotation = Quat::from_rotation_z((indices.speed.max(0.).abs() / 50.) as f32);
            if transform.translation.y < -174. {
                transform.translation.y = -174.;
                ev_game_over.send(GameOverEvent());
            }
        }
    }
}

fn game_over_event(
    mut game_over: ResMut<GameState>,
    mut game_over_and_space_query: Query<
        &mut Visibility,
        Or<(With<GameOverDisplay>, With<PressSpace>)>,
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_game_over: EventReader<GameOverEvent>,
) {
    for _ in ev_game_over.read() {
        game_over.game_over = true;
        commands.spawn(AudioBundle {
            source: asset_server.load("audio/hit.ogg"),
            settings: PlaybackSettings::DESPAWN,
            ..default()
        });
        for mut vis in game_over_and_space_query.iter_mut() {
            *vis = Visibility::Visible;
        }
    }
}

fn display_score(score: Res<Score>, mut sprites: Query<&mut Sprite, With<ScoreDisplay>>) {
    if !score.is_changed() {
        return;
    }

    let mut digits = Vec::new();
    let mut current_score = score.value.min(99999);
    if current_score == 0 {
        for _i in 0..=4 {
            digits.push(0);
        }
    } else {
        while current_score > 0 {
            let digit = current_score % 10;
            digits.push(digit);
            current_score /= 10;
        }
    }
    for (i, mut sprite) in sprites.iter_mut().enumerate() {
        if let Some(digit) = digits.get(i) {
            sprite.rect = Some(Rect::new(
                0.,
                (*digit as f32) * 36.,
                24.,
                ((*digit as f32) + 1.) * 36.,
            ));
        }
    }
}

fn jump(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut score: ResMut<Score>,
    asset_server: Res<AssetServer>,
    mut game_over: ResMut<GameState>,
    mut upper_pipe_query: Query<&mut Transform, (Without<LowerPipes>, With<UpperPipes>)>,
    mut lower_pipe_query: Query<&mut Transform, (Without<UpperPipes>, With<LowerPipes>)>,
    mut bird_query: Query<
        (&mut BirdAnimationIndices, &mut Transform),
        (
            With<BirdAnimationIndices>,
            Without<UpperPipes>,
            Without<LowerPipes>,
        ),
    >,
    mut game_over_and_space_query: Query<
        &mut Visibility,
        Or<(With<GameOverDisplay>, With<PressSpace>)>,
    >,
) {
    if input.just_pressed(KeyCode::Space) {
        if !game_over.game_over {
            let (mut indices, transform) = bird_query.single_mut();
            if transform.translation.y < WINDOW_Y / 2. {
                if indices.speed > 2. {
                    indices.speed = 12.;
                } else {
                    indices.speed = 8.;
                }
            }
            commands.spawn(AudioBundle {
                source: asset_server.load("audio/wing.ogg"),
                settings: PlaybackSettings::DESPAWN,
                ..default()
            });
        } else {
            if game_over.first_start {
                game_over.first_start = false;
            } else {
                let (mut indices, mut bird) = bird_query.single_mut();
                bird.translation.y = 0.;
                indices.speed = 0.;
                bird.rotation = Quat::from_rotation_x(0.);
                let mut x = 300.;
                for mut lower_t in lower_pipe_query.iter_mut() {
                    lower_t.translation.x = x;
                    x += 200.;
                }
                x = 300.;
                for mut upper_t in upper_pipe_query.iter_mut() {
                    upper_t.translation.x = x;
                    x += 200.;
                }
            }

            for mut vis in game_over_and_space_query.iter_mut() {
                *vis = Visibility::Hidden;
            }
            game_over.game_over = false;
            score.value = 0;
        }
    }
}

fn move_bg(
    time: Res<Time>,
    score: Res<Score>,
    mut bg_query: Query<&mut Transform, With<Background>>,
) {
    let delta_seconds = time.delta_seconds();
    for mut transform in bg_query.iter_mut() {
        transform.translation.x -= delta_seconds * (100. + score.value.min(100) as f32);
        if transform.translation.x <= -288. {
            transform.translation.x = 0.0;
        }
    }
}

fn move_base(
    time: Res<Time>,
    score: Res<Score>,
    mut base_query: Query<&mut Transform, With<Base>>,
) {
    let delta_seconds = time.delta_seconds();
    for mut transform in base_query.iter_mut() {
        transform.translation.x -= delta_seconds * 2. * (100. + score.value.min(100) as f32);
        if transform.translation.x <= -288. {
            transform.translation.x = 0.0;
        }
    }
}

fn move_pipes_and_game_logic(
    time: Res<Time>,
    mut upper_pipe_query: Query<&mut Transform, (Without<LowerPipes>, With<UpperPipes>)>,
    mut lower_pipe_query: Query<&mut Transform, (Without<UpperPipes>, With<LowerPipes>)>,
    bird_query: Query<
        &Transform,
        (
            With<BirdAnimationIndices>,
            Without<UpperPipes>,
            Without<LowerPipes>,
        ),
    >,
    mut ev_game_over: EventWriter<GameOverEvent>,
    mut score: ResMut<Score>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let delta_seconds = time.delta_seconds();
    let bird = bird_query.single();
    let mut rand = [0; 5];
    let mut rng = rand::thread_rng();
    for i in 0..rand.len() {
        rand[i] = rng.gen_range(-200..75);
    }
    for (i, mut lower_t) in lower_pipe_query.iter_mut().enumerate() {
        let initial_x = lower_t.translation.x;
        if check_collision(
            bird.translation,
            Vec2::new(20., 32.),
            lower_t.translation,
            Vec2::new(48., 316.),
        ) {
            ev_game_over.send(GameOverEvent());
        }
        lower_t.translation.x -= delta_seconds * 2. * (100. + score.value.min(100) as f32);
        if lower_t.translation.x <= -500. {
            lower_t.translation.x = 500.0;
            lower_t.translation.y = -100. + rand[i] as f32;
        }
        if initial_x > 0. && lower_t.translation.x <= 0. {
            score.value += 1;
            commands.spawn(AudioBundle {
                source: asset_server.load("audio/point.ogg"),
                settings: PlaybackSettings::DESPAWN,
                ..default()
            });
        }
    }

    for (i, mut upper_t) in upper_pipe_query.iter_mut().enumerate() {
        if check_collision(
            bird.translation,
            Vec2::new(20., 32.),
            upper_t.translation,
            Vec2::new(48., 316.),
        ) {
            ev_game_over.send(GameOverEvent());
        }
        upper_t.translation.x -= delta_seconds * 2. * (100. + score.value.min(100) as f32);
        if upper_t.translation.x <= -500. {
            upper_t.translation.x = 500.0;
            upper_t.translation.y = -100. + 450. - score.value.min(100) as f32 + rand[i] as f32;
        }
    }
}

fn check_collision(pos1: Vec3, size1: Vec2, pos2: Vec3, size2: Vec2) -> bool {
    let half_width1 = size1.x / 2.;
    let half_height1 = size1.y / 2.;
    let half_width2 = size2.x / 2.;
    let half_height2 = size2.y / 2.;

    let x1_min = pos1.x - half_width1;
    let x1_max = pos1.x + half_width1;
    let y1_min = pos1.y - half_height1;
    let y1_max = pos1.y + half_height1;

    let x2_min = pos2.x - half_width2;
    let x2_max = pos2.x + half_width2;
    let y2_min = pos2.y - half_height2;
    let y2_max = pos2.y + half_height2;

    let collision_x = x1_max >= x2_min && x2_max >= x1_min;
    let collision_y = y1_max >= y2_min && y2_max >= y1_min;

    collision_x && collision_y
}
