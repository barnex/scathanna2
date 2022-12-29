use super::internal::*;

type Color = vec3;



// Fill image with its average color.
// For generating fast, very low-quality ambient/sky light during debug bakes.
pub fn smudge<C>(img: &mut Img<C>)
where
	C: Copy + Default + std::iter::Sum + Div<f32, Output = C>,
{
	let avg = img.pixels().iter().copied().sum::<C>() / (img.pixels().len() as f32);
	img.fill(avg);
}

pub fn sum_2_imgs<'a, I1, I2, T>(a: I1, b: I2) -> impl Iterator<Item = Img<T>> + 'a
where
	I1: IntoIterator<Item = &'a Img<T>> + 'a,
	I2: IntoIterator<Item = &'a Img<T>> + 'a,
	T: Add<Output = T> + Copy + Default + 'a,
{
	a.into_iter().zip(b.into_iter()).map(|(a, b)| Img::from_fn(a.size(), |pix| a.at(pix) + b.at(pix)))
}

pub fn sum_3_imgs<'a, I1, I2, I3, T>(a: I1, b: I2, c: I3) -> impl Iterator<Item = Img<T>> + 'a
where
	I1: IntoIterator<Item = &'a Img<T>> + 'a,
	I2: IntoIterator<Item = &'a Img<T>> + 'a,
	I3: IntoIterator<Item = &'a Img<T>> + 'a,
	T: Add<Output = T> + Copy + Default + 'a,
{
	a.into_iter()
		.zip(b.into_iter())
		.zip(c.into_iter())
		.map(|((a, b), c)| Img::from_fn(a.size(), |pix| a.at(pix) + b.at(pix) + c.at(pix)))
}

