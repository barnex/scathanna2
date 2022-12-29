/// ! Common imports.
pub use anyhow::anyhow;
pub use anyhow::Context;
pub use anyhow::Error;
pub use anyhow::Result;
pub use bytemuck::Pod;
pub use bytemuck::Zeroable;
pub use flate2::read::GzDecoder;
pub use flate2::write::GzEncoder;
pub use image::DynamicImage;
pub use image::GenericImageView;
pub use image::Rgb;
pub use image::RgbImage;
pub use log::error;
pub use log::info;
pub use log::trace;
pub use rand::{prelude::StdRng, Rng};
pub use rand_xoshiro::rand_core::SeedableRng;
pub use rand_xoshiro::Xoshiro256PlusPlus;
pub use rayon::prelude::*;
pub use serde::de::DeserializeOwned;
pub use serde::{Deserialize, Serialize};
pub use smallvec::*;
pub use wgpu::util::DeviceExt;
pub use winit::event::ElementState;
pub use winit::event::KeyboardInput;
pub use winit::event::VirtualKeyCode;
pub use winit::event::WindowEvent;
pub use winit::window::Window;

pub use matrix::*;
pub use vector::*;

pub use crate::color::*;
pub use crate::geom::*;
pub use crate::graphics::*;
pub use crate::img::Img;
pub use crate::lightmap_baking::*;
pub use crate::raytrace::*;
pub use crate::settings::*;
pub use crate::shell::*;
pub use crate::sound::*;
pub use crate::util::*;

pub use std::cell::Cell;
pub use std::cell::RefCell;
pub use std::cmp::Ordering;
pub use std::fmt;
pub use std::fs::File;
pub use std::io::BufReader;
pub use std::io::BufWriter;
pub use std::io::Read;
pub use std::io::Write;
pub use std::mem;
pub use std::mem::take;
pub use std::net::TcpListener;
pub use std::net::TcpStream;
pub use std::num::NonZeroU32;
pub use std::num::NonZeroU8;
pub use std::ops::Add;
pub use std::ops::Div;
pub use std::ops::Mul;
pub use std::ops::Range;
pub use std::ops::Sub;
pub use std::path::Path;
pub use std::path::PathBuf;
pub use std::rc::Rc;
pub use std::str::FromStr;
pub use std::sync::mpsc;
pub use std::sync::mpsc::channel;
pub use std::sync::mpsc::Receiver;
pub use std::sync::mpsc::Sender;
pub use std::sync::Arc;
pub use std::sync::Mutex;
pub use std::thread;
pub use std::thread::spawn;
pub use std::time::Duration;
pub use std::time::Instant;

pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type Set<T> = fnv::FnvHashSet<T>;
pub type CowStr = std::borrow::Cow<'static, str>;

/// Shorthand for Default::default()
pub fn default<T: Default>() -> T {
	T::default()
}
