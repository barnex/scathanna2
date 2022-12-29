use super::*;
use bytemuck::{Pod, Zeroable};

unsafe impl<T> Zeroable for Vector2<T> where T: Zeroable {}
unsafe impl<T> Pod for Vector2<T> where T: Pod {}

unsafe impl<T> Zeroable for Vector3<T> where T: Zeroable {}
unsafe impl<T> Pod for Vector3<T> where T: Pod {}

unsafe impl<T> Zeroable for Vector4<T> where T: Zeroable {}
unsafe impl<T> Pod for Vector4<T> where T: Pod {}
