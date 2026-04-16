#![allow(dead_code)]
// Bat_OS — CSS Flexbox Layout Engine
// Implements CSS Flexible Box Layout (Level 1).
// Handles: flex-direction, justify-content, align-items, flex-grow/shrink.

use super::LayoutBox;

/// Flexbox properties
#[derive(Clone, Copy, PartialEq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Clone, Copy, PartialEq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
}

#[derive(Clone, Copy)]
pub struct FlexStyle {
    pub direction: FlexDirection,
    pub justify: JustifyContent,
    pub align: AlignItems,
    pub gap: i32,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: i32, // -1 = auto
}

impl FlexStyle {
    pub const fn default() -> Self {
        FlexStyle {
            direction: FlexDirection::Row,
            justify: JustifyContent::FlexStart,
            align: AlignItems::Stretch,
            gap: 0,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: -1,
        }
    }
}

/// Perform flex layout on children within a container.
/// `container` = the flex container box
/// `children` = indices of child boxes
/// `boxes` = the full layout box array (mutable for positioning)
pub fn layout_flex(
    container_x: i32,
    container_y: i32,
    container_w: i32,
    container_h: i32,
    flex: &FlexStyle,
    child_indices: &[usize],
    boxes: &mut [LayoutBox],
) {
    let count = child_indices.len();
    if count == 0 { return; }

    let is_row = flex.direction == FlexDirection::Row || flex.direction == FlexDirection::RowReverse;
    let main_size = if is_row { container_w } else { container_h };
    let cross_size = if is_row { container_h } else { container_w };

    // Calculate total content size + flex grow factors
    let mut total_fixed = 0i32;
    let mut total_grow = 0.0f32;
    for &ci in child_indices {
        let child_main = if is_row { boxes[ci].width } else { boxes[ci].height };
        total_fixed += child_main + flex.gap;
        total_grow += flex.flex_grow;
    }
    total_fixed -= flex.gap; // remove last gap

    // Distribute remaining space
    let remaining = (main_size - total_fixed).max(0);
    let grow_unit = if total_grow > 0.0 { remaining as f32 / total_grow } else { 0.0 };

    // Position children along main axis
    let mut main_pos = match flex.justify {
        JustifyContent::FlexStart => 0,
        JustifyContent::FlexEnd => main_size - total_fixed,
        JustifyContent::Center => (main_size - total_fixed) / 2,
        JustifyContent::SpaceBetween => 0,
        JustifyContent::SpaceAround => if count > 0 { remaining / (count as i32 * 2) } else { 0 },
        JustifyContent::SpaceEvenly => if count > 0 { remaining / (count as i32 + 1) } else { 0 },
    };

    let space_between = if flex.justify == JustifyContent::SpaceBetween && count > 1 {
        remaining / (count as i32 - 1)
    } else { 0 };

    for (_i, &ci) in child_indices.iter().enumerate() {
        let child_main = if is_row { boxes[ci].width } else { boxes[ci].height };
        let grow_extra = (flex.flex_grow * grow_unit) as i32;

        // Main axis position
        let effective_main = child_main + grow_extra;

        // Cross axis position
        let child_cross = if is_row { boxes[ci].height } else { boxes[ci].width };
        let cross_pos = match flex.align {
            AlignItems::FlexStart => 0,
            AlignItems::FlexEnd => cross_size - child_cross,
            AlignItems::Center => (cross_size - child_cross) / 2,
            AlignItems::Stretch => 0,
        };
        let effective_cross = if flex.align == AlignItems::Stretch { cross_size } else { child_cross };

        // Apply position
        if is_row {
            boxes[ci].x = container_x + main_pos;
            boxes[ci].y = container_y + cross_pos;
            boxes[ci].width = effective_main;
            boxes[ci].height = effective_cross;
        } else {
            boxes[ci].x = container_x + cross_pos;
            boxes[ci].y = container_y + main_pos;
            boxes[ci].width = effective_cross;
            boxes[ci].height = effective_main;
        }

        boxes[ci].content_x = boxes[ci].x;
        boxes[ci].content_y = boxes[ci].y;
        boxes[ci].content_w = boxes[ci].width;
        boxes[ci].content_h = boxes[ci].height;

        main_pos += effective_main + flex.gap;

        // Extra spacing for justify modes
        if flex.justify == JustifyContent::SpaceBetween { main_pos += space_between; }
        if flex.justify == JustifyContent::SpaceAround { main_pos += remaining / count as i32; }
        if flex.justify == JustifyContent::SpaceEvenly { main_pos += remaining / (count as i32 + 1); }
    }
}
