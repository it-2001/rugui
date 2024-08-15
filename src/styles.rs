use std::{default, sync::Arc};

use bytemuck::Zeroable;

use crate::{render::Color, texture::Texture};

pub struct StyleSheet {
    /// Rotation of the element
    /// 
    /// Rotation will be inherited by children
    pub rotation: Rotation,
    pub position: Positions,
    /// Width of the element
    /// 
    /// Can be overridden by min_width and max_width
    pub width: Size,
    /// Maximum width of the element that the element can not go above under any circumstances
    pub max_width: Size,
    /// Minimum width of the element that the element can not go below under any circumstances
    pub min_width: Size,
    /// Height of the element
    /// 
    /// Can be overridden by min_height and max_height
    pub height: Size,
    /// Maximum height of the element that the element can not go above under any circumstances
    pub max_height: Size,
    /// Minimum height of the element that the element can not go below under any circumstances
    pub min_height: Size,
    /// Background of the element
    /// 
    /// Each type of background can applied once
    /// 
    /// Order of rendering:
    /// 1. Texture
    /// 3. LinGradient
    /// 4. RadGradient
    /// 2. Color
    /// 
    /// Background is rgba(0, 0, 0, 0) by default
    pub background: Background,
    /// Border of the element
    /// 
    /// Not implemented yet
    pub border: Border,
    /// Margin of the element
    /// 
    /// Margin is the space between the element and its parent
    pub margin: Size,
    /// Padding of the element
    /// 
    /// Padding is the space between the element and its children
    /// 
    /// Not implemented yet
    pub padding: Size,
    /// Visibility of the element
    /// 
    /// If false, the element and its children will not be rendered
    pub visible: bool,
}

impl Default for StyleSheet {
    fn default() -> Self {
        Self {
            rotation: Rotation::None,
            position: Positions::Center(Position::Center),
            width: Size::Fill,
            max_width: Size::None,
            min_width: Size::None,
            height: Size::Fill,
            max_height: Size::None,
            min_height: Size::None,
            background: Background {
                color: Color::zeroed(),
                texture: None,
                lin_gradient: None,
                rad_gradient: None,
            },
            border: Border {
                background: Background {
                    color: Color::zeroed(),
                    texture: None,
                    lin_gradient: None,
                    rad_gradient: None,
                },
                width: Size::None,
                min_width: Size::None,
                max_width: Size::None,
                radius: Size::None,
                min_radius: Size::None,
                max_radius: Size::None,
                visible: false,
            },
            margin: Size::Pixel(0.0),
            padding: Size::Pixel(0.0),
            visible: true,
        }
    }
}

impl StyleSheet {
    pub fn get_width(&self, parent_width: f32) -> f32 {
        let w = match self.width {
            Size::Fill => parent_width,
            Size::Pixel(width) => width,
            Size::Percent(percent) => parent_width * (percent / 100.),
            Size::None => parent_width,
        };
        let min = match self.min_width {
            Size::Pixel(width) => width,
            Size::Percent(percent) => parent_width * (percent / 100.),
            _ => 0.0,
        };
        let max = match self.max_width {
            Size::Pixel(width) => width,
            Size::Percent(percent) => parent_width * (percent / 100.),
            _ => std::f32::INFINITY,
        };
        let margin = match self.margin {
            Size::Pixel(width) => width,
            Size::Percent(percent) => parent_width * (percent / 100.),
            _ => 0.0,
        };
        (w - margin).max(min).min(max)
    }

    pub fn get_height(&self, parent_height: f32) -> f32 {
        let h = match self.height {
            Size::Fill => parent_height,
            Size::Pixel(height) => height,
            Size::Percent(percent) => parent_height * (percent / 100.),
            Size::None => parent_height,
        };
        let min = match self.min_height {
            Size::Pixel(height) => height,
            Size::Percent(percent) => parent_height * (percent / 100.),
            _ => 0.0,
        };
        let max = match self.max_height {
            Size::Pixel(height) => height,
            Size::Percent(percent) => parent_height * (percent / 100.),
            _ => std::f32::INFINITY,
        };
        let margin = match self.margin {
            Size::Pixel(height) => height,
            Size::Percent(percent) => parent_height * (percent / 100.),
            _ => 0.0,
        };
        (h - margin).max(min).min(max)
    }

    pub fn get_x(&self, parent_x: f32, parent_width: f32, width: f32) -> f32 {
        let gap = parent_width - width;
        let x = match &self.position {
            Positions::Center(pos) => match pos {
                Position::BottomLeft | Position::Left | Position::TopLeft => parent_x - width / 2.0,
                Position::Bottom | Position::Center | Position::Top => parent_x,
                Position::BottomRight | Position::Right | Position::TopRight => parent_x + width / 2.0,
                Position::Custom(x, _) => match x {
                    Size::Pixel(x) => parent_x + x,
                    Size::Percent(percent) => parent_x + width * (percent / 100.),
                    _ => parent_x
                },
            },
            Positions::Align(pos) => match pos {
                Position::Bottom | Position::Center | Position::Top => parent_x,
                Position::Custom(x, _) => match x {
                    Size::Pixel(x) => parent_x + x,
                    Size::Percent(percent) => parent_x + width * (percent / 100.),
                    _ => parent_x
                },
                Position::BottomLeft | Position::Left | Position::TopLeft => parent_x - gap / 2.0,
                Position::BottomRight | Position::Right | Position::TopRight => parent_x + gap / 2.0,
            },
        };
        let margin = match self.margin {
            Size::Pixel(width) => width,
            Size::Percent(percent) => width * (percent / 100.),
            _ => 0.0,
        };
        x + margin
    }

    pub fn get_y(&self, parent_y: f32, parent_height: f32, height: f32) -> f32 {
        let gap = parent_height - height;
        let y = match &self.position {
            Positions::Center(pos) => match pos {
                Position::TopLeft | Position::Top | Position::TopRight => parent_y + height / 2.0,
                Position::Left | Position::Center | Position::Right => parent_y,
                Position::BottomLeft | Position::Bottom | Position::BottomRight => parent_y - height / 2.0,
                Position::Custom(_, y) => match y {
                    Size::Pixel(y) => parent_y + y,
                    Size::Percent(percent) => parent_y + height * (percent / 100.),
                    _ => parent_y
                },
            },
            Positions::Align(pos) => match pos {
                Position::Left | Position::Center | Position::Right => parent_y,
                Position::Custom(_, y) => match y {
                    Size::Pixel(y) => parent_y + y,
                    Size::Percent(percent) => parent_y + height * (percent / 100.),
                    _ => parent_y
                },
                Position::TopLeft | Position::Top | Position::TopRight => parent_y + gap / 2.0,
                Position::BottomLeft | Position::Bottom | Position::BottomRight => parent_y - gap / 2.0,
            },
        };
        let margin = match self.margin {
            Size::Pixel(height) => height,
            Size::Percent(percent) => height * (percent / 100.),
            _ => 0.0,
        };
        y + margin
    }
}

/// Position of the element relative to its parent
pub enum Position {
    Top,
    TopLeft,
    TopRight,
    Right,
    Bottom,
    BottomRight,
    BottomLeft,
    Left,
    Center,
    Custom(Size, Size),
}

/// Element placement method
pub enum Positions {
    Center(Position),
    Align(Position),
}

/// Border of the element
/// 
/// Not implemented yet
pub struct Border {
    pub background: Background,
    pub width: Size,
    pub min_width: Size,
    pub max_width: Size,
    pub radius: Size,
    pub min_radius: Size,
    pub max_radius: Size,
    pub visible: bool,
}

#[derive(Clone, Copy, Debug, Default)]
/// Size of the element
/// 
/// Size is the width or height of the element
pub enum Size {
    None,
    #[default]
    Fill,
    Pixel(f32),
    Percent(f32),
}

#[derive(Clone, Copy, Debug, Default)]
/// Rotation of the element
/// 
/// Not implemented yet
pub enum Rotation {
    #[default]
    None,
    Deg(f32),
    Rad(f32),
    Percent(f32),
}

/// Background of the element
pub struct Background {
    /// Color of the element
    /// 
    /// Color is rgba(0, 0, 0, 0) by default
    /// 
    /// Color is rendered last. If any other kind of background is applied,
    /// the color will be rendered last and can be used as a tint
    pub color: Color,
    pub texture: Option<Arc<Texture>>,
    /// Linear gradient of the element
    /// 
    /// Not implemented yet
    pub lin_gradient: Option<LinGradient>,
    /// Radial gradient of the element
    /// 
    /// Not implemented yet
    pub rad_gradient: Option<RadGradient>,
}

/// Linear gradient of the element
/// 
/// Not implemented yet
pub struct LinGradient {
    pub p1: (Position, Color),
    pub p2: (Position, Color),
}


/// Radial gradient of the element
/// 
/// Not implemented yet
pub struct RadGradient {
    pub p1: (Position, Color),
    pub p2: (Position, Color),
}