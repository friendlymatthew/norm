#![allow(clippy::suboptimal_flops)]

const K1: f32 = 0.01;
const K2: f32 = 0.03;

const C1: f32 = (K1 * 255.0) * (K1 * 255.0);
const C2: f32 = (K2 * 255.0) * (K2 * 255.0);
const C3: f32 = C2 / 2.0;

#[derive(Debug)]
pub struct LumaBuffer {
    pub(crate) mean_intensity: f32,
    pub(crate) std_dev: f32,
    pub(crate) lumas: Vec<f32>,
}

impl LumaBuffer {
    pub fn new(lumas: Vec<f32>, mean_intensity: f32) -> Self {
        let std_dev = Self::std_dev(mean_intensity, &lumas);
        Self {
            mean_intensity,
            lumas,
            std_dev,
        }
    }

    #[inline]
    pub fn ssim(&self, reference_image: &Self) -> f32 {
        let other_mean_intensity = reference_image.mean_intensity;
        let covariance = self.covariance(&reference_image.lumas, other_mean_intensity);
        let other_std_dev = reference_image.std_dev;

        let luminance = self.compare_luminance(other_mean_intensity);
        let contrast = self.compare_contrast(other_std_dev);
        let structure = self.compare_structure(covariance, other_std_dev);

        luminance * contrast * structure
    }

    #[inline]
    fn compare_luminance(&self, other_mean_intensity: f32) -> f32 {
        let m = self.mean_intensity;
        let n = other_mean_intensity;

        (2.0 * m * n + C1) / (m * m + n * n + C1)
    }

    #[inline]
    fn compare_contrast(&self, other_std_dev: f32) -> f32 {
        let v = self.std_dev;
        let u = other_std_dev;

        (2.0 * v * u + C2) / (v * v + u * u + C2)
    }

    #[inline]
    fn compare_structure(&self, covariance: f32, other_std_dev: f32) -> f32 {
        (covariance + C3) / (self.std_dev * other_std_dev + C3)
    }

    #[inline]
    fn covariance(&self, other_luma: &[f32], other_mean_intensity: f32) -> f32 {
        let m = self.mean_intensity;
        let n = other_mean_intensity;
        self.lumas
            .iter()
            .zip(other_luma)
            .map(|(&x, &y)| (x - m) * (y - n))
            .sum::<f32>()
            / (self.lumas.len() - 1) as f32
    }

    #[inline]
    fn std_dev(mean_intensity: f32, luma: &[f32]) -> f32 {
        let variance = luma
            .iter()
            .map(|&l| {
                let d = l - mean_intensity;
                d * d
            })
            .sum::<f32>()
            / (luma.len() - 1) as f32;
        variance.sqrt()
    }
}

/*
#[cfg(test)]
mod tests {
    /*
    The following tests uses the Matlab SSIM implementation to assert correctness.
    https://www.mathworks.com/help/images/ref/ssim.html
     */
    use crate::{Decoder, Png};
    use anyhow::Result;
    use std::fs;

    fn read_png(image_path: &str) -> Result<Png> {
        let image_data = fs::read(image_path)?;
        let image = Decoder::new(&image_data).decode()?;

        Ok(image)
    }
    #[test]
    fn test_mona_lisa_gauss_1() -> Result<()> {
        let reference = read_png("mona-lisa.png")?;
        let test = read_png("mona-lisa-gauss-1.png")?;

        let ssim = test.compute_sim(&reference)?;

        assert_eq!(ssim, 0.6990);

        Ok(())
    }

    #[test]
    fn test_mona_lisa_gauss_2() -> Result<()> {
        let reference = read_png("mona-lisa.png")?;
        let test = read_png("mona-lisa-gauss-2.png")?;

        let ssim = test.compute_sim(&reference)?;

        assert_eq!(ssim, 0.5232);

        Ok(())
    }

    #[test]
    fn test_mona_lisa_gauss_3() -> Result<()> {
        let reference = read_png("mona-lisa.png")?;
        let test = read_png("mona-lisa-gauss-3.png")?;

        let ssim = test.compute_sim(&reference)?;

        assert_eq!(ssim, 0.4921);

        Ok(())
    }

    #[test]
    fn test_mona_lisa_gauss_5() -> Result<()> {
        let reference = read_png("mona-lisa.png")?;
        let test = read_png("mona-lisa-gauss-5.png")?;

        let ssim = test.compute_sim(&reference)?;

        assert_eq!(ssim, 0.4506);

        Ok(())
    }

    #[test]
    fn test_mona_lisa_gauss_10() -> Result<()> {
        let reference = read_png("mona-lisa.png")?;
        let test = read_png("mona-lisa-gauss-10.png")?;

        let ssim = test.compute_sim(&reference)?;

        assert_eq!(ssim, 0.4128);

        Ok(())
    }
}
 */
