//! The `ImagePipeline` module contains a struct and implementation for converting images to ASCII
//! art. It offers a pipeline for processing images by resizing and converting them into ASCII
//! representations using a character lookup table.
use crate::common::errors::*;
use fast_image_resize as fr;
use image::{DynamicImage, GenericImageView as _, GrayImage, RgbImage};
use std::num::NonZeroU32;

use super::char_maps::CharMap;

/// The `ImagePipeline` struct encapsulates the process of converting an image to ASCII art. It
/// stores the target resolution (width and height) and the character lookup table used for the
/// conversion.
pub struct ImagePipeline<T: CharMap> {
    /// The target resolution (width and height) for the pipeline.
    pub target_resolution: (u32, u32),
    /// The character lookup table used for the conversion.
    pub char_map: T,
    /// Whether to add newlines to the output at the end of each line
    pub new_lines: bool,
}

impl<T: CharMap> ImagePipeline<T> {
    /// Constructs a new `ImagePipeline` with the given target resolution (width and height) and
    /// character lookup table (a vector of characters).
    ///
    /// # Arguments
    ///
    /// * `target_resolution` - A tuple of two u32 integers representing the target width and
    ///   height.
    /// * `char_map` - A vector of characters to be used as the lookup table for ASCII
    ///   conversion.
    pub fn new(target_resolution: (u32, u32), char_map: T, new_lines: bool) -> Self {
        Self {
            target_resolution,
            char_map,
            new_lines,
        }
    }

    /// Sets the target resolution (width and height) for the pipeline and returns a mutable
    /// reference to self.
    ///
    /// # Arguments
    ///
    /// * `width` - The target width as a u32 integer.
    /// * `height` - The target height as a u32 integer.
    pub fn set_target_resolution(&mut self, width: u32, height: u32) -> &mut Self {
        self.target_resolution = (width, height);
        self
    }

    /// Resizes a given `DynamicImage` to the target resolution specified in the `self` object.
    ///
    /// This function takes a reference to a `DynamicImage` and resizes it using the nearest
    /// neighbor algorithm. The resized image is returned as a `DynamicImage`.
    ///
    /// # Arguments
    ///
    /// * `img` - A reference to the `DynamicImage` to be resized.
    ///
    /// # Returns
    ///
    /// A `Result` containing a resized `DynamicImage` if the operation is successful, or a
    /// `MyError` if an error occurs.
    ///
    /// # Errors
    ///
    /// This function may return a `MyError` if any of the following conditions are encountered:
    ///
    /// * The input image has a width or height of zero.
    /// * The target resolution has a width or height of zero.
    /// * An error occurs while creating an `fr::Image` from the input image.
    /// * An error occurs while resizing the image using the `fr::Resizer`.
    /// * An error occurs while creating an `ImageBuffer` from the resized image data.
    pub fn resize(&self, img: &DynamicImage) -> Result<(GrayImage, RgbImage), MyError> {

        let subpixel_res = self.char_map.get_subpixels();

        let subpixel_img = self.resize_single(
            img,
            NonZeroU32::new(self.target_resolution.0 * subpixel_res.0)
                .ok_or(MyError::Pipeline(ERROR_DATA.to_string()))?,
            NonZeroU32::new(self.target_resolution.1 * subpixel_res.1)
                .ok_or(MyError::Pipeline(ERROR_DATA.to_string()))?,
            fr::ResizeAlg::Nearest
        )?;

        let color_img = self.resize_single(
            &subpixel_img,
            NonZeroU32::new(self.target_resolution.0)
                .ok_or(MyError::Pipeline(ERROR_DATA.to_string()))?,
            NonZeroU32::new(self.target_resolution.1)
                .ok_or(MyError::Pipeline(ERROR_DATA.to_string()))?,
            fr::ResizeAlg::Convolution(fr::FilterType::Box)
        )?;

        Ok((subpixel_img.into_luma8(), color_img.into_rgb8()))
    }

    fn resize_single(&self, img: &DynamicImage, width: NonZeroU32, height: NonZeroU32, algo: fr::ResizeAlg) -> Result<DynamicImage, MyError> {
        let src_width =
            NonZeroU32::new(img.width()).ok_or(MyError::Pipeline(ERROR_DATA.to_string()))?;
        let src_height =
            NonZeroU32::new(img.height()).ok_or(MyError::Pipeline(ERROR_DATA.to_string()))?;
        let src_image = fr::Image::from_vec_u8(
            src_width,
            src_height,
            img.to_owned().into_rgb8().to_vec(),
            fr::PixelType::U8x3,
        )
        .map_err(|err| MyError::Pipeline(format!("{ERROR_RESIZE}:{err:?}")))?;
        let mut dst_image = fr::Image::new(
            width,
            height,
            fr::PixelType::U8x3,
        );
        let mut dst_view = dst_image.view_mut();

        let mut resizer = fr::Resizer::new(algo);
        resizer
            .resize(&src_image.view(), &mut dst_view)
            .map_err(|err| MyError::Pipeline(format!("{ERROR_RESIZE}:{err:?}")))?;

        let dst_image = dst_image.into_vec();
        let img_buff = image::ImageBuffer::<image::Rgb<u8>, _>::from_vec(
            width.into(),
            height.into(),
            dst_image,
        )
        .ok_or(MyError::Pipeline(ERROR_DATA.to_string()))?;
        Ok(DynamicImage::ImageRgb8(img_buff))
    }

    /// Converts the given grayscale image to ASCII art using the character lookup table stored in
    /// this `ImagePipeline`.
    ///
    /// This method iterates through the pixels of the input image and maps each pixel's grayscale
    /// value to a character from the lookup table. The resulting ASCII art is returned as a
    /// `String`.
    ///
    /// # Arguments
    ///
    /// * `input` - A reference to a `GrayImage` to be converted to ASCII art.
    ///
    /// # Returns
    ///
    /// A `String` containing the ASCII art representation of the input image.
    pub fn to_ascii(&self, input: &GrayImage) -> Vec<String> {
        let (width, height) = self.target_resolution;

        let mut output = Vec::with_capacity(height as usize);

        let (subpixel_width, subpixel_height) = self.char_map.get_subpixels();
        assert_eq!(width * subpixel_width, input.width());
        assert_eq!(height * subpixel_height, input.height());

        for y in 0..height {
            let line = (0..width).map(|x| {
                self.char_map.get_char(&input.view(x * subpixel_width, y * subpixel_height, subpixel_width, subpixel_height))
            })
            // Add newlines to the end of each row except the last. NOTE: these
            // are not really needed because the terminal will wrap lines. But
            // if you want to copy the output to a file it would be a single
            // long string without them.
            .chain(
                ['\n', '\r'].into_iter().take(if self.new_lines && y < height - 1 { 2 } else { 0 })
            )
            .collect();

            output.push(line);
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::char_maps::CHARS1;
    use image::{DynamicImage, ImageError};
    use reqwest;
    use std::io::Cursor;

    const TEST_IMAGE_URL: &str = "https://sipi.usc.edu/database/preview/misc/4.1.01.png";

    fn download_image(url: &str) -> Result<DynamicImage, ImageError> {
        let response = reqwest::blocking::get(url)
            .expect("Failed to download image")
            .bytes()
            .expect("Failed to get image bytes");

        let image_data = Cursor::new(response);
        image::load(image_data, image::ImageFormat::Png)
    }

    #[test]
    fn test_new() {
        let image = ImagePipeline::new((120, 80), vec!['a', 'b', 'c'], false);
        assert_eq!(image.target_resolution, (120, 80));
        assert_eq!(image.char_map, vec!['a', 'b', 'c']);
    }

    #[test]
    fn test_process() {
        let image = ImagePipeline::new((120, 80), vec!['a', 'b', 'c'], false);
        let input = download_image(TEST_IMAGE_URL).expect("Failed to download image");

        let output = image.resize(&input).expect("Failed to resize image").1;
        assert_eq!(output.width(), 120);
        assert_eq!(output.height(), 80);
    }

    #[test]
    fn test_to_ascii_ext() {
        let image = ImagePipeline::new((120, 80), CHARS1.chars().collect::<Vec<char>>(), false);
        let input = download_image(TEST_IMAGE_URL).expect("Failed to download image");
        let output = image.to_ascii(
            &image
                .resize(&input)
                .expect("Failed to resize image")
                .0,
        );
        assert_eq!(output.iter().map(|str| str.chars().count()).sum::<usize>(), 120 * 80);
    }

    #[test]
    fn test_to_ascii() {
        let image = ImagePipeline::new((120, 80), vec!['a', 'b', 'c'], false);
        let input = download_image(TEST_IMAGE_URL).expect("Failed to download image");
        let output = image.to_ascii(
            &image
                .resize(&input)
                .expect("Failed to resize image")
                .0,
        );
        assert_eq!(output.iter().map(|str| str.chars().count()).sum::<usize>(), 120 * 80);
    }
}
