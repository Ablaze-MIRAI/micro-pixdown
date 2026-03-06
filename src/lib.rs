use std::collections::HashMap;
use std::sync::LazyLock;
use fronma::{engines::Toml, parser::parse_with_engine};
use serde::{Deserialize, Serialize};
use regex::Regex;
use itertools::Itertools;
use wasm_bindgen::prelude::*;

#[derive(Deserialize, Clone)]
struct Config {
    size: SizeConfig,
    binaries: HashMap<char, bool>,
    mix: Option<String>,
    options: Option<Options>,
}

#[derive(Deserialize, Clone)]
struct SizeConfig {
    w: usize,
    h: usize,
    frames: usize,
    rate: Option<[u64; 2]>
}

#[derive(Deserialize, Clone)]
struct Options {
    order: Option<Vec<usize>>
}

struct Layer {
    index: usize,
    content: LayerContent
}

struct Frame {
    index: usize,
    content: Vec<String>
}

enum LayerContent {
    Still(Vec<String>),
    Video(Vec<Frame>)
}

enum LayerPixmap {
    Still(FrameRaw),
    Video(FramesRaw),
}

#[derive(PartialEq)]
enum Token {
    Layer(usize),
    Frame(usize),
    Normal(String)
}

#[derive(Serialize)]
struct FrameJson {
    width: usize,
    height: usize,
    rate: [u64; 2],
    frames: Vec<Vec<Vec<bool>>>
}

type FrameRaw = Vec<Vec<bool>>;
type FramesRaw = Vec<FrameRaw>;

#[wasm_bindgen]
pub fn compile(s: &str) -> String {
    let data = parse_with_engine::<Config, Toml>(s).unwrap();
    let config = data.headers;
    let body = data.body;
    println!("[1/5] Parsing...");
    let token = body.lines().map(|c| tokenize(c)).filter(|c| c != &Token::Normal("".to_string())).collect::<Vec<_>>();
    let ast = parse(token);
    println!("[2/5] Putting color data...");
    let layers = generate_layers(&config.clone(), ast);
    println!("[3/5] Merging layers...");
    let frames = generate_frames(&config, layers);
    println!("[4/5] Applying options...");
    let applyed = if let Some(option) = config.clone().options {
        applyoption(frames, option)
    } else {
        frames
    };
    println!("[5/5] Serializing...");
    let result = generate_json(&config, applyed);
    result
}

fn generate_json(conf: &Config, frames: FramesRaw) -> String {
    let json = FrameJson {
        width: conf.size.w,
        height: conf.size.h,
        rate: conf.size.rate.unwrap_or([1000, 60]),
        frames: frames
    };
    serde_json::to_string(&json).unwrap()
}

fn applyoption(frames: FramesRaw, option: Options) -> FramesRaw {
    let mut result: FramesRaw = frames;
    if let Some(args) = option.order {
        result = options::order(result, args);
    }
    result
}

// default: xor
fn generate_frames(conf: &Config, layers: Vec<LayerPixmap>) -> FramesRaw {
    let mut frames: FramesRaw = vec![];
    for f in 0..conf.size.frames {
        let mut frame_layers = vec![vec![Vec::<bool>::new(); conf.size.w];conf.size.h];
        for l in layers.iter() {
            if let LayerPixmap::Still(v) = l {
                v.iter().enumerate().for_each(|(y, c)| {
                    c.iter().enumerate().for_each(|(x, &d)| {
                        frame_layers[y % conf.size.h][x % conf.size.w].push(d);
                    });
                });
            }
            if let LayerPixmap::Video(vs) = l {
                let v = vs[f % vs.len()].clone();
                v.iter().enumerate().for_each(|(y, c)| {
                    c.iter().enumerate().for_each(|(x, &d)| {
                        frame_layers[y % conf.size.h][x % conf.size.w].push(d);
                    });
                });
            }
        }
        frames.push(frame_layers.into_iter().map(|c| c.into_iter().map(|d| {
            if let Some(mix) = &conf.mix {
                evalbit::eval(mix, &d)
            } else {
                d.into_iter().fold(false, |acc, x| acc ^ x)
            }
        }).collect::<Vec<_>>()).collect::<Vec<_>>());
    }
    frames
}

fn generate_layers(conf: &Config, ast: Vec<Layer>) -> Vec<LayerPixmap> {
    let mut layers: Vec<LayerPixmap> = vec![];
    for l in ast.iter().sorted_by_key(|c| c.index) {
        if let LayerContent::Still(s) = &l.content {
            let pixmap = s.iter().map(|c| c.chars().map(|d| conf.binaries.get(&d).unwrap_or(&false).to_owned()).collect::<Vec<_>>()).collect::<Vec<_>>();
            layers.push(LayerPixmap::Still(pixmap));
        }
        if let LayerContent::Video(fs) = &l.content  {
            let mut pixmaps: FramesRaw = vec![];
            for f in fs.iter().sorted_by_key(|c| c.index) {
                let pixmap = f.content.iter().map(|c| c.chars().map(|d| conf.binaries.get(&d).unwrap_or(&false).to_owned()).collect::<Vec<_>>()).collect::<Vec<_>>();
                pixmaps.push(pixmap);
            }
            layers.push(LayerPixmap::Video(pixmaps));
        }
    }
    layers
}

fn parse(token_r: Vec<Token>) -> Vec<Layer> {
    let token = &token_r;
    let mut i = 0usize;
    let mut layers = Vec::<Layer>::new();
    while i < token.len() {
        while let Token::Layer(layern) = token[i] {
            i += 1;
            if let Token::Frame(_) = token[i] {
                let mut frames = Vec::<Frame>::new();
                while let Token::Frame(framen) = token[i]  {
                    i += 1;
                    let mut contents = Vec::<String>::new();
                    while let Token::Normal(s) = &token[i] {
                        contents.push(s.to_owned());
                        i += 1;
                        if i >= token.len() {
                            break;
                        }
                    }
                    frames.push(Frame {
                        index: framen,
                        content: contents
                    });
                    if i >= token.len() {
                        break;
                    }
                }
                layers.push(Layer {
                    index: layern,
                    content: LayerContent::Video(frames)
                });
            } else {
                let mut contents = Vec::<String>::new();
                while let Token::Normal(s) = &token[i] {
                    contents.push(s.to_owned());
                    i += 1;
                    if i >= token.len() {
                        break;
                    }
                }
                layers.push(Layer {
                    index: layern,
                    content: LayerContent::Still(contents)
                });
            }
            if i >= token.len() {
                break;
            }
        }
    }
    layers
}

static LAYER_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^# \d+$").unwrap());
static FRAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^## \d+$").unwrap());
fn tokenize(s: &str) -> Token {
    if FRAME_REGEX.is_match(s) {
        Token::Frame(s.split_at(3).1.parse::<usize>().unwrap())
    } else if LAYER_REGEX.is_match(s) {
        Token::Layer(s.split_at(2).1.parse::<usize>().unwrap())
    } else {
        Token::Normal(s.to_owned())
    }
}

mod options {
    use crate::FramesRaw;
    pub fn order(frames: FramesRaw, order: Vec<usize>) -> FramesRaw {
        let mut result: FramesRaw = vec![];
        for &i in &order {
            result.push((&frames)[i].clone());
        }
        result
    }
}
