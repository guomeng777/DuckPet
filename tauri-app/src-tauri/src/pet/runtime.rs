use std::path::PathBuf;

use serde::Serialize;

use super::{
    expression::{evaluate_expression, ExpressionContext},
    model::{Animation, NextOnly, NextTransition, PetManifest, Sound, Spawn},
};

#[derive(Debug, Clone, Copy)]
pub struct SpriteDimensions {
    pub sheet_width: i32,
    pub sheet_height: i32,
    pub tile_width: i32,
    pub tile_height: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetFrame {
    pub animation_id: i32,
    pub animation_name: String,
    pub frame_index: i32,
    pub sequence_step: i32,
    pub total_steps: i32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub interval_ms: i32,
    pub offset_y: i32,
    pub opacity: f64,
    pub flipped: bool,
}

#[derive(Debug, Clone, Copy)]
struct EvaluatedMovement {
    x: i32,
    y: i32,
    interval_ms: i32,
    offset_y: i32,
    opacity: f64,
}

pub struct PetRuntime {
    source_path: PathBuf,
    manifest: PetManifest,
    dimensions: SpriteDimensions,
    context: ExpressionContext,
    area_width: i32,
    area_height: i32,
    x: i32,
    y: i32,
    current_animation_id: i32,
    sequence_step: i32,
    total_steps: i32,
    start: EvaluatedMovement,
    end: EvaluatedMovement,
    flipped: bool,
    random_state: u64,
}

impl PetRuntime {
    pub fn new(
        source_path: PathBuf,
        manifest: PetManifest,
        dimensions: SpriteDimensions,
        area_width: i32,
        area_height: i32,
    ) -> Result<Self, String> {
        let mut runtime = Self {
            source_path,
            manifest,
            dimensions,
            context: ExpressionContext::new(
                area_width,
                area_height,
                area_width,
                area_height,
                dimensions.tile_width,
                dimensions.tile_height,
            )
            .with_spawn_random(37),
            area_width,
            area_height,
            x: 0,
            y: 0,
            current_animation_id: 0,
            sequence_step: 0,
            total_steps: 1,
            start: EvaluatedMovement::default(),
            end: EvaluatedMovement::default(),
            flipped: false,
            random_state: 0x5EED_5EED,
        };

        runtime.start_from_spawn()?;
        Ok(runtime)
    }

    pub fn source_path(&self) -> &PathBuf {
        &self.source_path
    }

    pub fn set_area(&mut self, area_width: i32, area_height: i32) {
        self.area_width = area_width.max(self.dimensions.tile_width);
        self.area_height = area_height.max(self.dimensions.tile_height);
        self.context.screen_w = self.area_width;
        self.context.screen_h = self.area_height;
        self.context.area_w = self.area_width;
        self.context.area_h = self.area_height;
    }

    pub fn sound_for_animation(&mut self, animation_id: i32) -> Option<Sound> {
        let sounds = self
            .manifest
            .sounds
            .iter()
            .filter(|sound| sound.animation_id == animation_id)
            .cloned()
            .collect::<Vec<_>>();

        for sound in sounds {
            let probability = sound.probability.clamp(0, 100);

            if self.next_random_percent() < probability {
                return Some(sound);
            }
        }

        None
    }

    pub fn current_bounds(&self) -> (i32, i32, i32, i32) {
        (
            self.x,
            self.y,
            self.dimensions.tile_width,
            self.dimensions.tile_height,
        )
    }

    pub fn sprite_dimensions(&self) -> SpriteDimensions {
        self.dimensions
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x.clamp(0, self.area_width - self.dimensions.tile_width);
        self.y = y.clamp(0, self.area_height - self.dimensions.tile_height);
    }

    pub fn enter_animation_by_name(&mut self, animation_name: &str) -> Result<bool, String> {
        let animation_id = self
            .manifest
            .animations
            .iter()
            .find(|animation| animation.name.eq_ignore_ascii_case(animation_name))
            .map(|animation| animation.id);

        if let Some(animation_id) = animation_id {
            self.enter_animation(animation_id)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn next_frame(&mut self) -> Result<PetFrame, String> {
        self.next_frame_with_window_floor(None)
    }

    pub fn next_frame_with_window_floor(
        &mut self,
        window_floor_y: Option<i32>,
    ) -> Result<PetFrame, String> {
        let animation = self.current_animation()?.clone();
        let frame_index = frame_for_step(&animation, self.sequence_step);
        let mut movement = self.interpolated_movement();

        if self.flipped {
            movement.x = -movement.x;
        }

        if let Some(condition) = self.border_condition(movement, window_floor_y) {
            self.clamp_to_border(condition, movement, window_floor_y);

            if let Some(next) =
                choose_next_transition(&animation.border_next, condition, &mut self.random_state)
            {
                self.enter_animation(next.animation_id)?;
                return self.first_frame();
            }
        } else if self.should_apply_gravity(movement, window_floor_y) {
            if let Some(next) = choose_next_transition(
                &animation.gravity_next,
                NextOnly::None,
                &mut self.random_state,
            ) {
                self.enter_animation(next.animation_id)?;
                return self.first_frame();
            }
        } else {
            self.x += movement.x;
            self.y += movement.y;
            self.snap_to_floor_if_close(window_floor_y);
        }

        let frame = PetFrame {
            animation_id: animation.id,
            animation_name: animation.name.clone(),
            frame_index,
            sequence_step: self.sequence_step,
            total_steps: self.total_steps,
            x: self.x,
            y: self.y - movement.offset_y,
            width: self.dimensions.tile_width,
            height: self.dimensions.tile_height,
            interval_ms: movement.interval_ms.max(16),
            offset_y: movement.offset_y,
            opacity: movement.opacity,
            flipped: self.flipped,
        };

        self.sequence_step += 1;

        if self.sequence_step >= self.total_steps {
            self.complete_current_sequence(&animation, window_floor_y)?;
        }

        Ok(frame)
    }

    fn first_frame(&mut self) -> Result<PetFrame, String> {
        let animation = self.current_animation()?.clone();
        let movement = self.interpolated_movement();
        let frame_index = frame_for_step(&animation, self.sequence_step);
        let frame = PetFrame {
            animation_id: animation.id,
            animation_name: animation.name,
            frame_index,
            sequence_step: self.sequence_step,
            total_steps: self.total_steps,
            x: self.x,
            y: self.y - movement.offset_y,
            width: self.dimensions.tile_width,
            height: self.dimensions.tile_height,
            interval_ms: movement.interval_ms.max(16),
            offset_y: movement.offset_y,
            opacity: movement.opacity,
            flipped: self.flipped,
        };

        self.sequence_step += 1;
        Ok(frame)
    }

    fn start_from_spawn(&mut self) -> Result<(), String> {
        let spawn = choose_weighted_owned(&self.manifest.spawns, &mut self.random_state)
            .ok_or_else(|| "pet manifest has no spawn entries".to_owned())?;

        self.context.rand_s = 10 + (self.next_random_percent() % 80);
        self.x = evaluate_expression(&spawn.x, &mut self.context)
            .map_err(|error| format!("failed to evaluate spawn x: {error}"))?;
        self.y = evaluate_expression(&spawn.y, &mut self.context)
            .map_err(|error| format!("failed to evaluate spawn y: {error}"))?;
        self.y = self.floor_y(self.y);

        let next_animation_id = spawn
            .next
            .as_ref()
            .map(|next| next.animation_id)
            .or_else(|| self.manifest.animations.first().map(|animation| animation.id))
            .ok_or_else(|| "pet manifest has no animations".to_owned())?;

        self.enter_animation(next_animation_id)
    }

    fn enter_animation(&mut self, animation_id: i32) -> Result<(), String> {
        let animation = self
            .manifest
            .animations
            .iter()
            .find(|animation| animation.id == animation_id)
            .cloned()
            .ok_or_else(|| format!("animation id {animation_id} does not exist"))?;

        self.start = self.evaluate_movement(&animation.start)?;
        self.end = self.evaluate_movement(&animation.end)?;
        self.total_steps = self.calculate_total_steps(&animation)?;
        self.current_animation_id = animation.id;
        self.sequence_step = 0;

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn enter_animation_for_test(&mut self, animation_id: i32) -> Result<(), String> {
        self.enter_animation(animation_id)
    }

    #[cfg(test)]
    pub(crate) fn set_position_for_test(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    #[cfg(test)]
    pub(crate) fn is_flipped_for_test(&self) -> bool {
        self.flipped
    }

    fn complete_current_sequence(
        &mut self,
        animation: &Animation,
        window_floor_y: Option<i32>,
    ) -> Result<(), String> {
        if animation.sequence.action.as_deref() == Some("flip") {
            self.flipped = !self.flipped;
        }

        let condition = if self.is_on_floor(window_floor_y) {
            self.floor_condition(window_floor_y)
        } else {
            NextOnly::None
        };
        let next_animation_id =
            choose_next_transition(&animation.sequence.next, condition, &mut self.random_state)
            .map(|next| next.animation_id)
            .or_else(|| self.manifest.animations.first().map(|animation| animation.id));

        if let Some(animation_id) = next_animation_id {
            self.enter_animation(animation_id)
        } else {
            self.start_from_spawn()
        }
    }

    fn evaluate_movement(
        &mut self,
        movement: &super::model::Movement,
    ) -> Result<EvaluatedMovement, String> {
        Ok(EvaluatedMovement {
            x: evaluate_expression(&movement.x, &mut self.context)
                .map_err(|error| format!("failed to evaluate movement x: {error}"))?,
            y: evaluate_expression(&movement.y, &mut self.context)
                .map_err(|error| format!("failed to evaluate movement y: {error}"))?,
            interval_ms: evaluate_expression(&movement.interval, &mut self.context)
                .map_err(|error| format!("failed to evaluate movement interval: {error}"))?,
            offset_y: movement.offset_y,
            opacity: movement.opacity,
        })
    }

    fn calculate_total_steps(&mut self, animation: &Animation) -> Result<i32, String> {
        if animation.sequence.frames.is_empty() {
            return Ok(1);
        }

        let repeat = evaluate_expression_or_default(&animation.sequence.repeat, &mut self.context, 0)
            .map_err(|error| format!("failed to evaluate sequence repeat: {error}"))?
            .max(0);
        let repeat_from = animation.sequence.repeat_from.max(0) as usize;
        let repeat_len = animation.sequence.frames.len().saturating_sub(repeat_from);
        let total_steps = animation.sequence.frames.len() as i32 + repeat_len as i32 * repeat;

        Ok(total_steps.max(1))
    }

    fn interpolated_movement(&self) -> EvaluatedMovement {
        let x = interpolate_i32(
            self.start.x,
            self.end.x,
            self.sequence_step,
            self.total_steps - 1,
        );
        let y = interpolate_i32(
            self.start.y,
            self.end.y,
            self.sequence_step,
            self.total_steps - 1,
        );
        let interval_ms = interpolate_i32(
            self.start.interval_ms,
            self.end.interval_ms,
            self.sequence_step,
            self.total_steps,
        );
        let offset_y = interpolate_i32(
            self.start.offset_y,
            self.end.offset_y,
            self.sequence_step,
            self.total_steps,
        );
        let opacity = interpolate_f64(
            self.start.opacity,
            self.end.opacity,
            self.sequence_step,
            self.total_steps,
        );

        EvaluatedMovement {
            x,
            y,
            interval_ms,
            offset_y,
            opacity,
        }
    }

    fn current_animation(&self) -> Result<&Animation, String> {
        self.manifest
            .animations
            .iter()
            .find(|animation| animation.id == self.current_animation_id)
            .ok_or_else(|| {
                format!(
                    "animation id {} does not exist",
                    self.current_animation_id
                )
            })
    }

    fn floor_y(&self, y: i32) -> i32 {
        y.min(self.area_height - self.dimensions.tile_height)
    }

    fn floor(&self) -> i32 {
        self.area_height - self.dimensions.tile_height
    }

    fn effective_floor(&self, window_floor_y: Option<i32>) -> i32 {
        let area_floor = self.floor();

        window_floor_y
            .map(|floor| floor - self.dimensions.tile_height)
            .filter(|floor| *floor >= 0 && *floor < area_floor)
            .unwrap_or(area_floor)
    }

    fn floor_condition(&self, window_floor_y: Option<i32>) -> NextOnly {
        if self.effective_floor(window_floor_y) < self.floor() {
            NextOnly::Window
        } else {
            NextOnly::Taskbar
        }
    }

    fn is_on_floor(&self, window_floor_y: Option<i32>) -> bool {
        self.y >= self.effective_floor(window_floor_y) - 2
    }

    fn snap_to_floor_if_close(&mut self, window_floor_y: Option<i32>) {
        let floor = self.effective_floor(window_floor_y);

        if self.y < floor && self.y + 3 >= floor {
            self.y = floor;
        } else {
            self.y = self.floor_y(self.y);
        }
    }

    fn should_apply_gravity(
        &self,
        movement: EvaluatedMovement,
        window_floor_y: Option<i32>,
    ) -> bool {
        !self.current_animation()
            .map(|animation| animation.gravity_next.is_empty())
            .unwrap_or(true)
            && self.y + movement.y < self.effective_floor(window_floor_y) - 3
    }

    fn border_condition(
        &self,
        movement: EvaluatedMovement,
        window_floor_y: Option<i32>,
    ) -> Option<NextOnly> {
        let next_x = self.x + movement.x;
        let next_y = self.y + movement.y;
        let floor = self.effective_floor(window_floor_y);

        if movement.x < 0 && next_x < 0 {
            Some(NextOnly::Vertical)
        } else if movement.x > 0 && next_x + self.dimensions.tile_width > self.area_width {
            Some(NextOnly::Vertical)
        } else if movement.y < 0 && next_y < 0 {
            Some(NextOnly::Horizontal)
        } else if movement.y > 0 && next_y > floor {
            Some(self.floor_condition(window_floor_y))
        } else {
            None
        }
    }

    fn clamp_to_border(
        &mut self,
        condition: NextOnly,
        movement: EvaluatedMovement,
        window_floor_y: Option<i32>,
    ) {
        match condition {
            NextOnly::Vertical => {
                self.x =
                    (self.x + movement.x).clamp(0, self.area_width - self.dimensions.tile_width);
            }
            NextOnly::Horizontal => {
                self.y = (self.y + movement.y).max(0);
            }
            NextOnly::Taskbar => {
                self.y = self.floor();
            }
            NextOnly::Window => {
                self.y = self.effective_floor(window_floor_y);
            }
            NextOnly::None | NextOnly::HorizontalPlus => {}
        }
    }

    fn next_random_percent(&mut self) -> i32 {
        next_random_percent(&mut self.random_state)
    }
}

impl Default for EvaluatedMovement {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            interval_ms: 100,
            offset_y: 0,
            opacity: 1.0,
        }
    }
}

fn frame_for_step(animation: &Animation, sequence_step: i32) -> i32 {
    if animation.sequence.frames.is_empty() {
        return 0;
    }

    let frame_count = animation.sequence.frames.len() as i32;

    if sequence_step < frame_count {
        return animation.sequence.frames[sequence_step as usize];
    }

    let repeat_from = animation
        .sequence
        .repeat_from
        .clamp(0, frame_count.saturating_sub(1));
    let repeat_len = frame_count - repeat_from;

    if repeat_len <= 0 {
        return animation.sequence.frames[0];
    }

    let frame_position = repeat_from + ((sequence_step - frame_count) % repeat_len);
    animation.sequence.frames[frame_position as usize]
}

fn choose_next_transition<'a>(
    transitions: &'a [NextTransition],
    condition: NextOnly,
    random_state: &mut u64,
) -> Option<&'a NextTransition> {
    let matching: Vec<&NextTransition> = transitions
        .iter()
        .filter(|transition| next_only_matches(transition.only, condition))
        .collect();

    choose_weighted_refs(matching, random_state)
}

fn next_only_matches(only: NextOnly, condition: NextOnly) -> bool {
    match only {
        NextOnly::None => true,
        NextOnly::HorizontalPlus => {
            condition == NextOnly::Horizontal || condition == NextOnly::Window
        }
        _ => only == condition,
    }
}

fn choose_weighted_owned(items: &[Spawn], random_state: &mut u64) -> Option<Spawn> {
    let total_probability: i32 = items
        .iter()
        .map(|item| item.probability.max(0))
        .sum();

    if total_probability <= 0 {
        return items.first().cloned();
    }

    let roll = next_random_percent(random_state) % total_probability;
    let mut cursor = 0;

    for item in items {
        cursor += item.probability.max(0);

        if roll < cursor {
            return Some(item.clone());
        }
    }

    items.last().cloned()
}

fn choose_weighted_refs<'a>(
    items: Vec<&'a NextTransition>,
    random_state: &mut u64,
) -> Option<&'a NextTransition> {
    let total_probability: i32 = items
        .iter()
        .map(|item| item.probability.max(0))
        .sum();

    if total_probability <= 0 {
        return items.first().copied();
    }

    let roll = next_random_percent(random_state) % total_probability;
    let mut cursor = 0;

    for item in &items {
        cursor += item.probability.max(0);

        if roll < cursor {
            return Some(*item);
        }
    }

    items.last().copied()
}

fn next_random_percent(random_state: &mut u64) -> i32 {
    *random_state = random_state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1);
    ((*random_state >> 32) % 100) as i32
}

fn evaluate_expression_or_default(
    expression: &str,
    context: &mut ExpressionContext,
    default: i32,
) -> Result<i32, super::expression::ExpressionError> {
    if expression.trim().is_empty() {
        Ok(default)
    } else {
        evaluate_expression(expression, context)
    }
}

fn interpolate_i32(start: i32, end: i32, step: i32, denominator: i32) -> i32 {
    if denominator <= 0 {
        return start;
    }

    start + ((end - start) * step / denominator)
}

fn interpolate_f64(start: f64, end: f64, step: i32, denominator: i32) -> f64 {
    if denominator <= 0 {
        return start;
    }

    start + (end - start) * step as f64 / denominator as f64
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::pet::xml::parse_pet_manifest_file;

    use super::{PetRuntime, SpriteDimensions};

    fn esheep_manifest_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("Pets")
            .join("esheep64")
            .join("animations.xml")
    }

    #[test]
    fn emits_frames_from_legacy_sequence() {
        let manifest = parse_pet_manifest_file(esheep_manifest_path()).expect("parse manifest");
        let dimensions = SpriteDimensions {
            sheet_width: 640,
            sheet_height: 440,
            tile_width: 40,
            tile_height: 40,
        };
        let mut runtime =
            PetRuntime::new(esheep_manifest_path(), manifest, dimensions, 200, 120)
                .expect("runtime");

        runtime.enter_animation(1).expect("walk animation");
        runtime.x = 120;
        runtime.y = 80;

        let first = runtime.next_frame().expect("first frame");
        let second = runtime.next_frame().expect("second frame");

        assert_eq!(1, first.animation_id);
        assert_eq!("walk", first.animation_name);
        assert_eq!(2, first.frame_index);
        assert_eq!(3, second.frame_index);
        assert!(second.x < first.x);
    }

    #[test]
    fn clamps_to_area_floor() {
        let manifest = parse_pet_manifest_file(esheep_manifest_path()).expect("parse manifest");
        let dimensions = SpriteDimensions {
            sheet_width: 640,
            sheet_height: 440,
            tile_width: 40,
            tile_height: 40,
        };
        let mut runtime =
            PetRuntime::new(esheep_manifest_path(), manifest, dimensions, 200, 120)
                .expect("runtime");

        runtime.enter_animation(5).expect("fall animation");
        runtime.y = 200;

        let frame = runtime.next_frame().expect("fall frame");

        assert!(frame.y <= 80);
        assert_ne!(5, frame.animation_id);
    }
}
