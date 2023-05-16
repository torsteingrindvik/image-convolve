use crate::prelude::*;
use std::debug_assert;

use image::{GenericImageView, SubImage};

/// Creates a view into an image of a 3x3 pixel area.
///
/// # Panics
///
/// If there isn't space to create the pixel area.
pub fn view3x3(image: &Image, row: u32, column: u32) -> SubImage<&Image> {
    debug_assert!(row != 0);
    debug_assert!(row < image.height() - 1);
    debug_assert!(column != 0);
    debug_assert!(column < image.width() - 1);

    image.view(column - 1, row - 1, 3, 3)
}
