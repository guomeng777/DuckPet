use std::path::Path;

use super::{
    model::NextOnly,
    xml::{parse_pet_manifest, parse_pet_manifest_file},
};

fn esheep_manifest_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("Pets")
        .join("esheep64")
        .join("animations.xml")
}

fn blue_sheep_manifest_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("Pets")
        .join("blue_sheep")
        .join("animations.xml")
}

#[test]
fn parses_esheep64_manifest_counts_and_header() {
    let manifest = parse_pet_manifest_file(esheep_manifest_path()).expect("parse esheep64 XML");

    assert_eq!(manifest.header.author, "Adriano");
    assert_eq!(manifest.header.title, "eSheep 64bit");
    assert_eq!(manifest.header.petname, "eSheep");
    assert_eq!(manifest.header.version, "1.8");
    assert_eq!(manifest.header.application, "1");
    assert!(!manifest.header.icon_base64.is_empty());

    assert_eq!(manifest.image.tiles_x, 16);
    assert_eq!(manifest.image.tiles_y, 11);
    assert_eq!(manifest.image.transparency, "Magenta");
    assert!(manifest.image.png_base64.starts_with("iVBORw0KGgo"));

    assert_eq!(manifest.spawns.len(), 4);
    assert_eq!(manifest.animations.len(), 54);
    assert_eq!(manifest.sounds.len(), 0);
}

#[test]
fn parses_real_manifest_sound_entries() {
    let manifest = parse_pet_manifest_file(blue_sheep_manifest_path()).expect("parse sound XML");

    let sound = manifest
        .sounds
        .iter()
        .find(|sound| sound.animation_id == 61)
        .expect("animation 61 sound exists");

    assert_eq!(sound.probability, 100);
    assert_eq!(sound.loop_count, 0);
    assert!(sound.base64.starts_with("//uQx"));
}

#[test]
fn keeps_spawn_expressions_as_strings() {
    let manifest = parse_pet_manifest_file(esheep_manifest_path()).expect("parse esheep64 XML");

    let spawn_one = manifest
        .spawns
        .iter()
        .find(|spawn| spawn.id == 1)
        .expect("spawn 1 exists");
    assert_eq!(spawn_one.probability, 20);
    assert_eq!(spawn_one.x, "screenW+10");
    assert_eq!(spawn_one.y, "areaH-imageH");
    assert_eq!(spawn_one.next.as_ref().map(|next| next.animation_id), Some(1));

    let spawn_two = manifest
        .spawns
        .iter()
        .find(|spawn| spawn.id == 2)
        .expect("spawn 2 exists");
    assert_eq!(spawn_two.x, "random*(screenW-imageW-50)/100+25");
    assert_eq!(spawn_two.y, "-imageH-20");
}

#[test]
fn parses_animation_sequence_border_and_gravity_transitions() {
    let manifest = parse_pet_manifest_file(esheep_manifest_path()).expect("parse esheep64 XML");

    let walk = manifest
        .animations
        .iter()
        .find(|animation| animation.id == 1)
        .expect("walk animation exists");

    assert_eq!(walk.name, "walk");
    assert_eq!(walk.start.x, "-2");
    assert_eq!(walk.start.y, "0");
    assert_eq!(walk.start.interval, "200");
    assert_eq!(walk.end.x, "-2");
    assert_eq!(walk.sequence.repeat, "20");
    assert_eq!(walk.sequence.repeat_from, 0);
    assert_eq!(walk.sequence.frames, vec![2, 3]);
    assert_eq!(walk.sequence.next.len(), 6);
    assert_eq!(walk.sequence.next[0].animation_id, 11);
    assert_eq!(walk.sequence.next[0].probability, 2);
    assert_eq!(walk.sequence.next[0].only, NextOnly::Window);

    assert_eq!(walk.border_next.len(), 3);
    assert_eq!(walk.border_next[1].animation_id, 37);
    assert_eq!(walk.border_next[1].only, NextOnly::Vertical);

    assert_eq!(walk.gravity_next.len(), 1);
    assert_eq!(walk.gravity_next[0].animation_id, 5);
    assert_eq!(walk.gravity_next[0].only, NextOnly::None);
}

#[test]
fn parses_sequence_action_and_optional_defaults() {
    let manifest = parse_pet_manifest_file(esheep_manifest_path()).expect("parse esheep64 XML");

    let rotate = manifest
        .animations
        .iter()
        .find(|animation| animation.id == 2)
        .expect("rotate animation exists");

    assert_eq!(rotate.name, "rotate1a");
    assert_eq!(rotate.sequence.frames, vec![3, 9, 10]);
    assert_eq!(rotate.sequence.action.as_deref(), Some("flip"));
    assert_eq!(rotate.start.offset_y, 0);
    assert_eq!(rotate.start.opacity, 1.0);
}

#[test]
fn parses_minimal_legacy_fragment() {
    let xml = r#"
        <animations xmlns="https://esheep.petrucci.ch/">
          <header>
            <author>A</author>
            <title>T</title>
            <petname>P</petname>
            <version>1</version>
            <info>I</info>
            <application>1</application>
            <icon>icon</icon>
          </header>
          <image>
            <tilesx>1</tilesx>
            <tilesy>1</tilesy>
            <png>png</png>
            <transparency>Magenta</transparency>
          </image>
          <spawns>
            <spawn id="1" probability="100">
              <x>screenW+10</x>
              <y>areaH-imageH</y>
              <next probability="100">1</next>
            </spawn>
          </spawns>
          <animations>
            <animation id="1">
              <name>walk</name>
              <start>
                <x>-2</x>
                <y>0</y>
                <interval>200</interval>
              </start>
              <sequence repeat="0" repeatfrom="0">
                <frame>2</frame>
                <next probability="100" only="none">1</next>
              </sequence>
            </animation>
          </animations>
          <sounds>
            <sound animationid="1">
              <probability>75</probability>
              <loop>2</loop>
              <base64>data:audio/mpeg;base64,AQIDBA==</base64>
            </sound>
          </sounds>
        </animations>
    "#;

    let manifest = parse_pet_manifest(xml).expect("parse minimal XML");
    assert_eq!(manifest.header.petname, "P");
    assert_eq!(manifest.spawns[0].x, "screenW+10");
    assert_eq!(manifest.animations[0].end, manifest.animations[0].start);
    assert_eq!(manifest.animations[0].sequence.frames, vec![2]);
    assert_eq!(manifest.sounds.len(), 1);
    assert_eq!(manifest.sounds[0].animation_id, 1);
    assert_eq!(manifest.sounds[0].probability, 75);
    assert_eq!(manifest.sounds[0].loop_count, 2);
    assert_eq!(manifest.sounds[0].base64, "data:audio/mpeg;base64,AQIDBA==");
}
