use crate::{MediaTime, Packet, SubtitleInfo};

pub mod ass;
pub mod nal;

#[derive(Default, Debug)]
pub struct TextStyle {
    name: String,
    font: Option<String>,
    primary_color: Option<u32>,
    secondary_color: Option<u32>,
    outline_color: Option<u32>,
    back_color: Option<u32>,
    bold: bool,
    italic: bool,
    underline: bool,
    strikeout: bool,
    scale_x: f32,
    scale_y: f32,
    spacing: i32,
    angle: i32,
    border_style: Option<i32>,
    outline: Option<i32>,
    shadow: Option<i32>,
    alignment: Option<i32>,
    margin_left: Option<i32>,
    margin_right: Option<i32>,
    margin_vertical: Option<i32>,
}

#[derive(Debug)]
pub struct TextCue {
    pub time: MediaTime,
    pub style: String,
    pub text: Vec<TextPart>,
}

#[derive(Eq, PartialEq, Debug)]
pub enum TextAlign {
    TopLeft,
    Top,
    TopRight,
    MidLeft,
    Mid,
    MidRight,
    BotLeft,
    Bot,
    BotRight,
}

#[derive(Eq, PartialEq, Debug)]
pub enum ColorType {
    Primary,
    Karaoke,
    Outline,
    Shadow,
}

#[derive(Debug, PartialEq)]
pub struct TextPosition(f32, f32);

#[derive(Eq, PartialEq, Debug)]
pub struct TextFill(ColorType, u32);

#[derive(Eq, PartialEq, Debug)]
pub struct TextAlpha(ColorType, u8);

#[derive(Debug)]
pub enum TextPart {
    Text(String),
    Italic(bool),
    Underline(bool),
    Strikeout(bool),
    Border(f32),
    FontSize(u32),
    Position(TextPosition),
    Fill(TextFill),
    Alpha(TextAlpha),
}

pub trait SubtitleDecoder {
    fn start(&mut self, info: &SubtitleInfo) -> anyhow::Result<()>;
    fn feed(&mut self, packet: Packet) -> anyhow::Result<()>;
    fn receive(&mut self) -> Option<TextCue>;
}
