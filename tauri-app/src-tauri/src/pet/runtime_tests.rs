use std::path::PathBuf;

use super::{
    model::{
        Animation, Header, Movement, NextOnly, NextTransition, PetManifest, Sequence, Spawn, Sound,
        SpriteSheet,
    },
    runtime::{PetRuntime, SpriteDimensions},
};

fn runtime() -> PetRuntime {
    PetRuntime::new(
        PathBuf::from("synthetic.xml"),
        manifest(),
        SpriteDimensions {
            sheet_width: 100,
            sheet_height: 10,
            tile_width: 10,
            tile_height: 10,
        },
        100,
        80,
    )
    .expect("runtime")
}

#[test]
fn border_transition_uses_vertical_condition_on_left_and_right_edges() {
    let mut runtime = runtime();
    runtime.enter_animation_for_test(1).expect("walk left");
    runtime.set_position_for_test(2, 70);

    let left = runtime.next_frame().expect("left border frame");

    assert_eq!(2, left.animation_id);
    assert_eq!("turn", left.animation_name);
    assert_eq!(0, left.x);

    runtime.enter_animation_for_test(6).expect("walk right");
    runtime.set_position_for_test(88, 70);

    let right = runtime.next_frame().expect("right border frame");

    assert_eq!(2, right.animation_id);
    assert_eq!(90, right.x);
}

#[test]
fn gravity_transition_starts_fall_when_pet_is_above_floor() {
    let mut runtime = runtime();
    runtime.enter_animation_for_test(1).expect("walk left");
    runtime.set_position_for_test(50, 20);

    let frame = runtime.next_frame().expect("gravity frame");

    assert_eq!(3, frame.animation_id);
    assert_eq!("fall", frame.animation_name);
}

#[test]
fn floor_border_uses_taskbar_condition() {
    let mut runtime = runtime();
    runtime.enter_animation_for_test(3).expect("fall");
    runtime.set_position_for_test(50, 66);

    let frame = runtime.next_frame().expect("floor border frame");

    assert_eq!(4, frame.animation_id);
    assert_eq!("land", frame.animation_name);
    assert_eq!(70, frame.y);
}

#[test]
fn sequence_next_can_use_taskbar_condition() {
    let mut runtime = runtime();
    runtime.enter_animation_for_test(7).expect("taskbar sequence");
    runtime.set_position_for_test(50, 70);

    let first = runtime.next_frame().expect("sequence frame");
    let next = runtime.next_frame().expect("next sequence frame");

    assert_eq!(7, first.animation_id);
    assert_eq!(8, next.animation_id);
    assert_eq!("taskbar-next", next.animation_name);
}

#[test]
fn falling_pet_uses_window_collision_condition() {
    let mut runtime = runtime();
    runtime.enter_animation_for_test(9).expect("window fall");
    runtime.set_position_for_test(50, 45);

    let frame = runtime
        .next_frame_with_window_floor(Some(60))
        .expect("window collision frame");

    assert_eq!(10, frame.animation_id);
    assert_eq!("window-land", frame.animation_name);
    assert_eq!(50, frame.y);
}

#[test]
fn sequence_next_can_use_window_condition() {
    let mut runtime = runtime();
    runtime.enter_animation_for_test(11).expect("window sequence");
    runtime.set_position_for_test(50, 50);

    let first = runtime
        .next_frame_with_window_floor(Some(60))
        .expect("sequence frame");
    let next = runtime
        .next_frame_with_window_floor(Some(60))
        .expect("next sequence frame");

    assert_eq!(11, first.animation_id);
    assert_eq!(12, next.animation_id);
    assert_eq!("window-next", next.animation_name);
}

#[test]
fn action_flip_changes_future_x_movement() {
    let mut runtime = runtime();
    runtime.enter_animation_for_test(5).expect("flip");
    runtime.set_position_for_test(50, 70);

    let flip = runtime.next_frame().expect("flip frame");
    let moved = runtime.next_frame().expect("moved frame");

    assert_eq!(5, flip.animation_id);
    assert!(runtime.is_flipped_for_test());
    assert_eq!(1, moved.animation_id);
    assert_eq!(55, moved.x);
}

#[test]
fn enters_animation_by_name_when_present() {
    let mut runtime = runtime();
    let entered = runtime
        .enter_animation_by_name("drag")
        .expect("named animation lookup");

    let frame = runtime.next_frame().expect("drag frame");

    assert!(entered);
    assert_eq!(13, frame.animation_id);
    assert_eq!("drag", frame.animation_name);
}

#[test]
fn reports_missing_named_animation_without_changing_animation() {
    let mut runtime = runtime();
    runtime.enter_animation_for_test(1).expect("walk left");

    let entered = runtime
        .enter_animation_by_name("missing")
        .expect("named animation lookup");
    let frame = runtime.next_frame().expect("walk frame");

    assert!(!entered);
    assert_eq!(1, frame.animation_id);
}

fn manifest() -> PetManifest {
    PetManifest {
        header: Header {
            author: "test".to_owned(),
            title: "test".to_owned(),
            petname: "test".to_owned(),
            version: "1".to_owned(),
            info: String::new(),
            application: "1".to_owned(),
            icon_base64: String::new(),
        },
        image: SpriteSheet {
            tiles_x: 10,
            tiles_y: 1,
            png_base64: String::new(),
            transparency: "Magenta".to_owned(),
        },
        spawns: vec![Spawn {
            id: 1,
            probability: 100,
            x: "50".to_owned(),
            y: "70".to_owned(),
            next: Some(next(1, 100, NextOnly::None)),
        }],
        sounds: vec![Sound {
            animation_id: 2,
            probability: 100,
            loop_count: 0,
            base64: "AQIDBA==".to_owned(),
        }],
        animations: vec![
            animation(
                1,
                "walk-left",
                movement("-5", "0"),
                vec![1],
                "100",
                vec![next(1, 100, NextOnly::None)],
                vec![next(2, 100, NextOnly::Vertical)],
                vec![next(3, 100, NextOnly::None)],
                None,
            ),
            animation(
                2,
                "turn",
                movement("0", "0"),
                vec![2],
                "0",
                vec![next(1, 100, NextOnly::None)],
                Vec::new(),
                Vec::new(),
                None,
            ),
            animation(
                3,
                "fall",
                movement("0", "10"),
                vec![3],
                "100",
                vec![next(3, 100, NextOnly::None)],
                vec![next(4, 100, NextOnly::Taskbar)],
                Vec::new(),
                None,
            ),
            animation(
                4,
                "land",
                movement("0", "0"),
                vec![4],
                "0",
                vec![next(1, 100, NextOnly::None)],
                Vec::new(),
                Vec::new(),
                None,
            ),
            animation(
                5,
                "flip",
                movement("0", "0"),
                vec![5],
                "0",
                vec![next(1, 100, NextOnly::None)],
                Vec::new(),
                Vec::new(),
                Some("flip"),
            ),
            animation(
                6,
                "walk-right",
                movement("5", "0"),
                vec![6],
                "100",
                vec![next(6, 100, NextOnly::None)],
                vec![next(2, 100, NextOnly::Vertical)],
                Vec::new(),
                None,
            ),
            animation(
                7,
                "taskbar-sequence",
                movement("0", "0"),
                vec![7],
                "0",
                vec![next(8, 100, NextOnly::Taskbar)],
                Vec::new(),
                Vec::new(),
                None,
            ),
            animation(
                8,
                "taskbar-next",
                movement("0", "0"),
                vec![8],
                "0",
                vec![next(8, 100, NextOnly::None)],
                Vec::new(),
                Vec::new(),
                None,
            ),
            animation(
                9,
                "window-fall",
                movement("0", "10"),
                vec![9],
                "100",
                vec![next(9, 100, NextOnly::None)],
                vec![next(10, 100, NextOnly::Window)],
                Vec::new(),
                None,
            ),
            animation(
                10,
                "window-land",
                movement("0", "0"),
                vec![10],
                "0",
                vec![next(1, 100, NextOnly::None)],
                Vec::new(),
                Vec::new(),
                None,
            ),
            animation(
                11,
                "window-sequence",
                movement("0", "0"),
                vec![11],
                "0",
                vec![next(12, 100, NextOnly::Window)],
                Vec::new(),
                Vec::new(),
                None,
            ),
            animation(
                12,
                "window-next",
                movement("0", "0"),
                vec![12],
                "0",
                vec![next(12, 100, NextOnly::None)],
                Vec::new(),
                Vec::new(),
                None,
            ),
            animation(
                13,
                "drag",
                movement("0", "0"),
                vec![13],
                "100",
                vec![next(13, 100, NextOnly::None)],
                Vec::new(),
                Vec::new(),
                None,
            ),
        ],
    }
}

#[test]
fn sound_for_animation_respects_animation_id_and_probability() {
    let mut runtime = runtime();

    let sound = runtime
        .sound_for_animation(2)
        .expect("animation 2 should have sound");

    assert_eq!(2, sound.animation_id);
    assert_eq!("AQIDBA==", sound.base64);
    assert!(runtime.sound_for_animation(1).is_none());
}


fn animation(
    id: i32,
    name: &str,
    movement: Movement,
    frames: Vec<i32>,
    repeat: &str,
    sequence_next: Vec<NextTransition>,
    border_next: Vec<NextTransition>,
    gravity_next: Vec<NextTransition>,
    action: Option<&str>,
) -> Animation {
    Animation {
        id,
        name: name.to_owned(),
        start: movement.clone(),
        end: movement,
        sequence: Sequence {
            repeat: repeat.to_owned(),
            repeat_from: 0,
            frames,
            next: sequence_next,
            action: action.map(str::to_owned),
        },
        border_next,
        gravity_next,
    }
}

fn movement(x: &str, y: &str) -> Movement {
    Movement {
        x: x.to_owned(),
        y: y.to_owned(),
        interval: "100".to_owned(),
        offset_y: 0,
        opacity: 1.0,
    }
}

fn next(animation_id: i32, probability: i32, only: NextOnly) -> NextTransition {
    NextTransition {
        animation_id,
        probability,
        only,
    }
}
