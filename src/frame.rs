// Copyright (c) 2018-2020, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

use std::mem::size_of;
use std::mem::transmute;

use crate::math::*;
use crate::pixel::*;
use crate::plane::*;
use crate::serialize::{Deserialize, Serialize};

// One video frame.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Frame<T: Pixel> {
    /// Planes constituting the frame.
    pub planes: [Plane<T>; 3],
}

impl<T: Pixel> Frame<T> {
    /// Creates a new frame with the given parameters.
    ///
    /// Allocates data for the planes.
    pub fn new_with_padding(
        width: usize,
        height: usize,
        chroma_sampling: ChromaSampling,
        luma_padding: usize,
    ) -> Self {
        let luma_width = width.align_power_of_two(3);
        let luma_height = height.align_power_of_two(3);

        let (chroma_decimation_x, chroma_decimation_y) =
            chroma_sampling.get_decimation().unwrap_or((0, 0));
        let (chroma_width, chroma_height) =
            chroma_sampling.get_chroma_dimensions(luma_width, luma_height);
        let chroma_padding_x = luma_padding >> chroma_decimation_x;
        let chroma_padding_y = luma_padding >> chroma_decimation_y;

        Frame {
            planes: [
                Plane::new(luma_width, luma_height, 0, 0, luma_padding, luma_padding),
                Plane::new(
                    chroma_width,
                    chroma_height,
                    chroma_decimation_x,
                    chroma_decimation_y,
                    chroma_padding_x,
                    chroma_padding_y,
                ),
                Plane::new(
                    chroma_width,
                    chroma_height,
                    chroma_decimation_x,
                    chroma_decimation_y,
                    chroma_padding_x,
                    chroma_padding_y,
                ),
            ],
        }
    }

    /// Creates a new frame with the given parameters from existing data, without copying.
    ///
    /// # Safety
    ///
    /// - This changes a non-mutable reference to a mutable one.
    ///   DO NOT reuse the original source of the input data for ANY PURPOSES afterwards.
    ///
    /// # Panics
    ///
    /// - If the size of the data does not match the expected dimensions given
    ///   by width, height, and chroma sampling.
    pub unsafe fn new_zerocopy(
        data: [&[u8]; 3],
        width: usize,
        height: usize,
        chroma_sampling: ChromaSampling,
    ) -> Self {
        let luma_width = width;
        let luma_height = height;

        // SAFETY: We assert that the sizes of the input data match our expectations
        // in order to maintain safety constraints.
        unsafe {
            assert!(data[0].len() == luma_width * luma_height * size_of::<T>());

            if chroma_sampling == ChromaSampling::Cs400 {
                Frame {
                    planes: [
                        Plane::from_slice_zerocopy(transmute(data[0]), luma_width),
                        Plane::new(0, 0, 0, 0, 0, 0),
                        Plane::new(0, 0, 0, 0, 0, 0),
                    ],
                }
            } else {
                let (chroma_width, chroma_height) =
                    chroma_sampling.get_chroma_dimensions(luma_width, luma_height);

                assert!(data[1].len() == chroma_width * chroma_height * size_of::<T>());
                assert!(data[2].len() == chroma_width * chroma_height * size_of::<T>());
                Frame {
                    planes: [
                        Plane::from_slice_zerocopy(transmute(data[0]), luma_width),
                        Plane::from_slice_zerocopy(transmute(data[1]), chroma_width),
                        Plane::from_slice_zerocopy(transmute(data[2]), chroma_width),
                    ],
                }
            }
        }
    }
}
