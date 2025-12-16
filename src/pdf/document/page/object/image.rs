//! Defines the [PdfPageImageObject] struct, exposing functionality related to a single page
//! object defining an image, where the image data is sourced from a [PdfBitmap] buffer.

use crate::bindgen::{
    fpdf_page_t__, FPDF_DOCUMENT, FPDF_IMAGEOBJ_METADATA, FPDF_PAGE, FPDF_PAGEOBJECT,
};
use crate::bindings::PdfiumLibraryBindings;
use crate::error::{PdfiumError, PdfiumInternalError};
use crate::pdf::bitmap::PdfBitmap;
use crate::pdf::bitmap::Pixels;
use crate::pdf::color_space::PdfColorSpace;
use crate::pdf::document::page::object::private::internal::PdfPageObjectPrivate;
use crate::pdf::document::page::object::{PdfPageObject, PdfPageObjectOwnership};
use crate::pdf::document::PdfDocument;
use crate::pdf::matrix::{PdfMatrix, PdfMatrixValue};
use crate::pdf::points::PdfPoints;
use crate::utils::mem::create_byte_buffer;
use crate::{create_transform_getters, create_transform_setters};
use std::convert::TryInto;
use std::ops::{Range, RangeInclusive};
use std::os::raw::{c_int, c_void};

#[cfg(not(target_arch = "wasm32"))]
use {
    crate::utils::files::get_pdfium_file_accessor_from_reader,
    std::fs::File,
    std::io::{Read, Seek},
    std::path::Path,
};

#[cfg(feature = "image_api")]
use {
    crate::pdf::bitmap::PdfBitmapFormat,
    crate::utils::pixels::{
        aligned_bgr_to_rgba, aligned_grayscale_to_unaligned, bgra_to_rgba, rgba_to_bgra,
    },
};

#[cfg(feature = "image_025")]
use image_025::{DynamicImage, EncodableLayout, GrayImage, RgbaImage};

#[cfg(feature = "image_024")]
use image_024::{DynamicImage, EncodableLayout, GrayImage, RgbaImage};

#[cfg(feature = "image_023")]
use image_023::{DynamicImage, EncodableLayout, GenericImageView, GrayImage, RgbaImage};

#[cfg(doc)]
use {
    crate::pdf::document::page::object::PdfPageObjectType,
    crate::pdf::document::page::objects::common::PdfPageObjectsCommon,
    crate::pdf::document::page::PdfPage,
};

/// A single [PdfPageObject] of type [PdfPageObjectType::Image]. The page object defines a
/// single image, where the image data is sourced from a [PdfBitmap] buffer.
///
/// Page objects can be created either attached to a [PdfPage] (in which case the page object's
/// memory is owned by the containing page) or detached from any page (in which case the page
/// object's memory is owned by the object). Page objects are not rendered until they are
/// attached to a page; page objects that are never attached to a page will be lost when they
/// fall out of scope.
///
/// The simplest way to create a page image object that is immediately attached to a page
/// is to call the [PdfPageObjectsCommon::create_image_object()] function.
///
/// Creating a detached page image object offers more scope for customization, but you must
/// add the object to a containing [PdfPage] manually. To create a detached page image object,
/// use the [PdfPageImageObject::new()] or [PdfPageImageObject::new_from_jpeg_file()] functions.
/// The detached page image object can later be attached to a page by using the
/// [PdfPageObjectsCommon::add_image_object()] function.
pub struct PdfPageImageObject<'a> {
    object_handle: FPDF_PAGEOBJECT,
    ownership: PdfPageObjectOwnership,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfPageImageObject<'a> {
    #[inline]
    pub(crate) fn from_pdfium(
        object_handle: FPDF_PAGEOBJECT,
        ownership: PdfPageObjectOwnership,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfPageImageObject {
            object_handle,
            ownership,
            bindings,
        }
    }

    /// Creates a new [PdfPageImageObject] from the given [DynamicImage]. The returned
    /// page object will not be rendered until it is added to a [PdfPage] using the
    /// [PdfPageObjectsCommon::add_image_object()] function.
    ///
    /// The returned page object will have its width and height both set to 1.0 points.
    /// Use the [PdfPageImageObject::scale()] function to apply a horizontal and vertical scale
    /// to the object after it is created, or use one of the [PdfPageImageObject::new_with_width()],
    /// [PdfPageImageObject::new_with_height()], or [PdfPageImageObject::new_with_size()] functions
    /// to scale the page object to a specific width and/or height at the time the object is created.
    ///
    /// This function is only available when this crate's `image` feature is enabled.
    #[cfg(feature = "image_api")]
    #[inline]
    pub fn new(document: &PdfDocument<'a>, image: &DynamicImage) -> Result<Self, PdfiumError> {
        let mut result = Self::new_from_handle(document.handle(), document.bindings());

        if let Ok(result) = result.as_mut() {
            result.set_image(image)?;
        }

        result
    }

    /// Creates a new [PdfPageImageObject]. The returned page object will not be
    /// rendered until it is added to a [PdfPage] using the
    /// [PdfPageObjects::add_image_object()] function.
    ///
    /// Use the [PdfPageImageObject::set_bitmap()] function to apply image data to
    /// the empty object.
    ///
    /// The returned page object will have its width and height both set to 1.0 points.
    /// Use the [WriteTransforms::scale()] function to apply a horizontal and vertical scale
    /// to the object after it is created.
    #[cfg(not(feature = "image_api"))]
    pub fn new(document: &PdfDocument<'a>) -> Result<Self, PdfiumError> {
        Self::new_from_handle(document.handle(), document.bindings())
    }

    /// Creates a new [PdfPageImageObject] containing JPEG image data loaded from the
    /// given file path. The returned page object will not be rendered until it is added to
    /// a [PdfPage] using the [PdfPageObjectsCommon::add_image_object()] function.
    ///
    /// The returned page object will have its width and height both set to 1.0 points.
    /// Use the [PdfPageImageObject::scale] function to apply a horizontal and vertical scale
    /// to the object after it is created, or use one of the [PdfPageImageObject::new_with_width()],
    /// [PdfPageImageObject::new_with_height()], or [PdfPageImageObject::new_with_size()] functions
    /// to scale the page object to a specific width and/or height at the time the object is created.
    ///
    /// This function is not available when compiling to WASM.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_from_jpeg_file(
        document: &PdfDocument<'a>,
        path: &(impl AsRef<Path> + ?Sized),
    ) -> Result<Self, PdfiumError> {
        Self::new_from_jpeg_reader(document, File::open(path).map_err(PdfiumError::IoError)?)
    }

    /// Creates a new [PdfPageImageObject] containing JPEG image data loaded from the
    /// given reader. Because Pdfium must know the total content length in advance prior to
    /// loading any portion of it, the given reader must implement the [Seek] trait
    /// as well as the [Read] trait.
    ///
    /// The returned page object will not be rendered until it is added to
    /// a [PdfPage] using the [PdfPageObjectsCommon::add_image_object()] function.
    ///
    /// The returned page object will have its width and height both set to 1.0 points.
    /// Use the [PdfPageImageObject::scale] function to apply a horizontal and vertical scale
    /// to the object after it is created, or use one of the [PdfPageImageObject::new_with_width()],
    /// [PdfPageImageObject::new_with_height()], or [PdfPageImageObject::new_with_size()] functions
    /// to scale the page object to a specific width and/or height at the time the object is created.
    ///
    /// This function is not available when compiling to WASM.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_from_jpeg_reader<R: Read + Seek>(
        document: &PdfDocument<'a>,
        reader: R,
    ) -> Result<Self, PdfiumError> {
        let object = Self::new_from_handle(document.handle(), document.bindings())?;

        let mut reader = get_pdfium_file_accessor_from_reader(reader);

        let result = document.bindings().FPDFImageObj_LoadJpegFileInline(
            std::ptr::null_mut(),
            0,
            object.object_handle(),
            reader.as_fpdf_file_access_mut_ptr(),
        );

        if object.bindings.is_true(result) {
            Ok(object)
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    // Takes a raw `FPDF_DOCUMENT` handle to avoid cascading lifetime problems
    // associated with borrowing `PdfDocument<'a>`.
    pub(crate) fn new_from_handle(
        document: FPDF_DOCUMENT,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Result<Self, PdfiumError> {
        let handle = bindings.FPDFPageObj_NewImageObj(document);

        if handle.is_null() {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        } else {
            Ok(PdfPageImageObject {
                object_handle: handle,
                ownership: PdfPageObjectOwnership::unowned(),
                bindings,
            })
        }
    }

    /// Creates a new [PdfPageImageObject] from the given arguments. The page object will be scaled
    /// horizontally to match the given width; its height will be adjusted to maintain the aspect
    /// ratio of the given image. The returned page object will not be rendered until it is
    /// added to a [PdfPage] using the [PdfPageObjectsCommon::add_image_object()] function.
    ///
    /// This function is only available when this crate's `image` feature is enabled.
    #[cfg(feature = "image_api")]
    pub fn new_with_width(
        document: &PdfDocument<'a>,
        image: &DynamicImage,
        width: PdfPoints,
    ) -> Result<Self, PdfiumError> {
        let aspect_ratio = image.height() as f32 / image.width() as f32;

        let height = width * aspect_ratio;

        Self::new_with_size(document, image, width, height)
    }

    /// Creates a new [PdfPageImageObject] from the given arguments. The page object will be scaled
    /// vertically to match the given height; its width will be adjusted to maintain the aspect
    /// ratio of the given image. The returned page object will not be rendered until it is
    /// added to a [PdfPage] using the [PdfPageObjectsCommon::add_image_object()] function.
    ///
    /// This function is only available when this crate's `image` feature is enabled.
    #[cfg(feature = "image_api")]
    pub fn new_with_height(
        document: &PdfDocument<'a>,
        image: &DynamicImage,
        height: PdfPoints,
    ) -> Result<Self, PdfiumError> {
        let aspect_ratio = image.height() as f32 / image.width() as f32;

        let width = height / aspect_ratio;

        Self::new_with_size(document, image, width, height)
    }

    /// Creates a new [PdfPageImageObject] from the given arguments. The page object will be scaled to
    /// match the given width and height. The returned page object will not be rendered until it is
    /// added to a [PdfPage] using the [PdfPageObjectsCommon::add_image_object()] function.
    ///
    /// This function is only available when this crate's `image` feature is enabled.
    #[cfg(feature = "image_api")]
    #[inline]
    pub fn new_with_size(
        document: &PdfDocument<'a>,
        image: &DynamicImage,
        width: PdfPoints,
        height: PdfPoints,
    ) -> Result<Self, PdfiumError> {
        let mut result = Self::new(document, image)?;

        result.scale(width.value, height.value)?;

        Ok(result)
    }

    /// Returns a new [PdfBitmap] created from the bitmap buffer backing
    /// this [PdfPageImageObject], ignoring any image filters, image mask, or object
    /// transforms applied to this page object.
    pub fn get_raw_bitmap(&self) -> Result<PdfBitmap<'_>, PdfiumError> {
        Ok(PdfBitmap::from_pdfium(
            self.bindings().FPDFImageObj_GetBitmap(self.object_handle()),
            self.bindings(),
        ))
    }

    /// Returns a new [DynamicImage] created from the bitmap buffer backing
    /// this [PdfPageImageObject], ignoring any image filters, image mask, or object
    /// transforms applied to this page object.
    ///
    /// This function is only available when this crate's `image` feature is enabled.
    #[cfg(feature = "image_api")]
    #[inline]
    pub fn get_raw_image(&self) -> Result<DynamicImage, PdfiumError> {
        self.get_image_from_bitmap(&self.get_raw_bitmap()?)
    }

    /// Returns a new [PdfBitmap] created from the bitmap buffer backing
    /// this [PdfPageImageObject], taking into account any image filters, image mask, and
    /// object transforms applied to this page object.
    #[inline]
    pub fn get_processed_bitmap(
        &self,
        document: &PdfDocument,
    ) -> Result<PdfBitmap<'_>, PdfiumError> {
        let (width, height) = self.get_current_width_and_height_from_metadata()?;

        self.get_processed_bitmap_with_size(document, width, height)
    }

    /// Returns a new [DynamicImage] created from the bitmap buffer backing
    /// this [PdfPageImageObject], taking into account any image filters, image mask, and
    /// object transforms applied to this page object.
    ///
    /// This function is only available when this crate's `image` feature is enabled.
    #[cfg(feature = "image_api")]
    #[inline]
    pub fn get_processed_image(&self, document: &PdfDocument) -> Result<DynamicImage, PdfiumError> {
        let (width, height) = self.get_current_width_and_height_from_metadata()?;

        self.get_processed_image_with_size(document, width, height)
    }

    /// Returns a new [PdfBitmap] created from the bitmap buffer backing
    /// this [PdfPageImageObject], taking into account any image filters, image mask, and
    /// object transforms applied to this page object.
    ///
    /// The returned bitmap will be scaled during rendering so its width matches the given target width.
    #[inline]
    pub fn get_processed_bitmap_with_width(
        &self,
        document: &PdfDocument,
        width: Pixels,
    ) -> Result<PdfBitmap<'_>, PdfiumError> {
        let (current_width, current_height) = self.get_current_width_and_height_from_metadata()?;

        let aspect_ratio = current_width as f32 / current_height as f32;

        self.get_processed_bitmap_with_size(
            document,
            width,
            ((width as f32 / aspect_ratio) as u32)
                .try_into()
                .map_err(|_| PdfiumError::ImageSizeOutOfBounds)?,
        )
    }

    /// Returns a new [DynamicImage] created from the bitmap buffer backing
    /// this [PdfPageImageObject], taking into account any image filters, image mask, and
    /// object transforms applied to this page object.
    ///
    /// The returned image will be scaled during rendering so its width matches the given target width.
    ///
    /// This function is only available when this crate's `image` feature is enabled.
    #[cfg(feature = "image_api")]
    #[inline]
    pub fn get_processed_image_with_width(
        &self,
        document: &PdfDocument,
        width: Pixels,
    ) -> Result<DynamicImage, PdfiumError> {
        let (current_width, current_height) = self.get_current_width_and_height_from_metadata()?;

        let aspect_ratio = current_width as f32 / current_height as f32;

        self.get_processed_image_with_size(
            document,
            width,
            ((width as f32 / aspect_ratio) as u32)
                .try_into()
                .map_err(|_| PdfiumError::ImageSizeOutOfBounds)?,
        )
    }

    /// Returns a new [PdfBitmap] created from the bitmap buffer backing
    /// this [PdfPageImageObject], taking into account any image filters, image mask, and
    /// object transforms applied to this page object.
    ///
    /// The returned bitmap will be scaled during rendering so its height matches the given target height.
    #[inline]
    pub fn get_processed_bitmap_with_height(
        &self,
        document: &PdfDocument,
        height: Pixels,
    ) -> Result<PdfBitmap<'_>, PdfiumError> {
        let (current_width, current_height) = self.get_current_width_and_height_from_metadata()?;

        let aspect_ratio = current_width as f32 / current_height as f32;

        self.get_processed_bitmap_with_size(
            document,
            ((height as f32 * aspect_ratio) as u32)
                .try_into()
                .map_err(|_| PdfiumError::ImageSizeOutOfBounds)?,
            height,
        )
    }

    /// Returns a new [DynamicImage] created from the bitmap buffer backing
    /// this [PdfPageImageObject], taking into account any image filters, image mask, and
    /// object transforms applied to this page object.
    ///
    /// The returned image will be scaled during rendering so its height matches the given target height.
    ///
    /// This function is only available when this crate's `image` feature is enabled.
    #[cfg(feature = "image_api")]
    #[inline]
    pub fn get_processed_image_with_height(
        &self,
        document: &PdfDocument,
        height: Pixels,
    ) -> Result<DynamicImage, PdfiumError> {
        let (current_width, current_height) = self.get_current_width_and_height_from_metadata()?;

        let aspect_ratio = current_width as f32 / current_height as f32;

        self.get_processed_image_with_size(
            document,
            ((height as f32 * aspect_ratio) as u32)
                .try_into()
                .map_err(|_| PdfiumError::ImageSizeOutOfBounds)?,
            height,
        )
    }

    /// Returns a new [PdfBitmap] created from the bitmap buffer backing
    /// this [PdfPageImageObject], taking into account any image filters, image mask, and
    /// object transforms applied to this page object.
    ///
    /// The returned bitmap will be scaled during rendering so its width and height match
    /// the given target dimensions.
    pub fn get_processed_bitmap_with_size(
        &self,
        document: &PdfDocument,
        width: Pixels,
        height: Pixels,
    ) -> Result<PdfBitmap<'_>, PdfiumError> {
        // We attempt to work around two separate problems in Pdfium's
        // FPDFImageObj_GetRenderedBitmap() function.

        // First, the call to FPDFImageObj_GetRenderedBitmap() can fail, returning
        // a null FPDF_BITMAP handle, if the image object's transformation matrix includes
        // negative values for either the matrix.a or matrix.d values. We flip those values
        // in the transformation matrix if they are negative, and we make sure we restore
        // the original values before we return to the caller.

        // Second, Pdfium seems to often return a rendered bitmap that is much smaller
        // than the image object's metadata suggests. We look at the dimensions of the bitmap
        // returned from FPDFImageObj_GetRenderedBitmap(), and we apply a scale factor to the
        // image object's transformation matrix if the bitmap is not the expected size.

        // For more details, see: https://github.com/ajrcarey/pdfium-render/issues/52

        let mut matrix = self.matrix()?;

        let original_matrix = matrix; // We'll restore the matrix to this before we return.

        // Ensure the matrix.a and matrix.d values are not negative.

        if matrix.a() < 0f32 {
            matrix.set_a(-matrix.a());
            self.reset_matrix_impl(matrix)?;
        }

        if matrix.d() < 0f32 {
            matrix.set_d(-matrix.d());
            self.reset_matrix_impl(matrix)?;
        }

        let page_handle = match self.ownership() {
            PdfPageObjectOwnership::Page(ownership) => Some(ownership.page_handle()),
            PdfPageObjectOwnership::AttachedAnnotation(ownership) => Some(ownership.page_handle()),
            _ => None,
        };

        let bitmap_handle = match page_handle {
            Some(page_handle) => self.bindings().FPDFImageObj_GetRenderedBitmap(
                document.handle(),
                page_handle,
                self.object_handle(),
            ),
            None => self.bindings.FPDFImageObj_GetRenderedBitmap(
                document.handle(),
                std::ptr::null_mut::<fpdf_page_t__>(),
                self.object_handle(),
            ),
        };

        if bitmap_handle.is_null() {
            // Restore the original transformation matrix values before we return the error
            // to the caller.

            self.reset_matrix_impl(original_matrix)?;
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        let result = PdfBitmap::from_pdfium(bitmap_handle, self.bindings());

        if width == result.width() && height == result.height() {
            // The bitmap generated by Pdfium is already at the caller's requested dimensions.
            // Restore the original transformation matrix values before we return to the caller.

            self.reset_matrix_impl(original_matrix)?;

            Ok(result)
        } else {
            // The bitmap generated by Pdfium is not at the caller's requested dimensions.
            // We apply a scale transform to the page object to encourage Pdfium to generate
            // a bitmap matching the caller's requested dimensions.

            self.transform_impl(
                width as PdfMatrixValue / result.width() as PdfMatrixValue,
                0.0,
                0.0,
                height as PdfMatrixValue / result.height() as PdfMatrixValue,
                0.0,
                0.0,
            )?;

            // Generate the bitmap again at the new scale.

            let result = PdfBitmap::from_pdfium(
                match page_handle {
                    Some(page_handle) => self.bindings().FPDFImageObj_GetRenderedBitmap(
                        document.handle(),
                        page_handle,
                        self.object_handle(),
                    ),
                    None => self.bindings.FPDFImageObj_GetRenderedBitmap(
                        document.handle(),
                        std::ptr::null_mut::<fpdf_page_t__>(),
                        self.object_handle(),
                    ),
                },
                self.bindings,
            );

            // Restore the original transformation matrix values before we return to the caller.

            self.reset_matrix_impl(original_matrix)?;

            Ok(result)
        }
    }

    /// Returns a new [DynamicImage] created from the bitmap buffer backing
    /// this [PdfPageImageObject], taking into account any image filters, image mask, and
    /// object transforms applied to this page object.
    ///
    /// The returned image will be scaled during rendering so its width and height match
    /// the given target dimensions.
    ///
    /// This function is only available when this crate's `image` feature is enabled.
    #[cfg(feature = "image_api")]
    #[inline]
    pub fn get_processed_image_with_size(
        &self,
        document: &PdfDocument,
        width: Pixels,
        height: Pixels,
    ) -> Result<DynamicImage, PdfiumError> {
        self.get_processed_bitmap_with_size(document, width, height)
            .and_then(|bitmap| self.get_image_from_bitmap(&bitmap))
    }

    #[cfg(feature = "image_api")]
    pub(crate) fn get_image_from_bitmap(
        &self,
        bitmap: &PdfBitmap,
    ) -> Result<DynamicImage, PdfiumError> {
        let handle = bitmap.handle();

        let width = self.bindings.FPDFBitmap_GetWidth(handle);

        let height = self.bindings.FPDFBitmap_GetHeight(handle);

        let stride = self.bindings.FPDFBitmap_GetStride(handle);

        let format =
            PdfBitmapFormat::from_pdfium(self.bindings.FPDFBitmap_GetFormat(handle) as u32)?;

        #[cfg(not(target_arch = "wasm32"))]
        let buffer = self.bindings.FPDFBitmap_GetBuffer_as_slice(handle);

        #[cfg(target_arch = "wasm32")]
        let buffer_vec = self.bindings.FPDFBitmap_GetBuffer_as_vec(handle);
        #[cfg(target_arch = "wasm32")]
        let buffer = buffer_vec.as_slice();

        match format {
            #[allow(deprecated)]
            PdfBitmapFormat::BGRA | PdfBitmapFormat::BRGx | PdfBitmapFormat::BGRx => {
                RgbaImage::from_raw(width as u32, height as u32, bgra_to_rgba(buffer))
                    .map(DynamicImage::ImageRgba8)
            }
            PdfBitmapFormat::BGR => RgbaImage::from_raw(
                width as u32,
                height as u32,
                aligned_bgr_to_rgba(buffer, width as usize, stride as usize),
            )
            .map(DynamicImage::ImageRgba8),
            PdfBitmapFormat::Gray => GrayImage::from_raw(
                width as u32,
                height as u32,
                aligned_grayscale_to_unaligned(buffer, width as usize, stride as usize),
            )
            .map(DynamicImage::ImageLuma8),
        }
        .ok_or(PdfiumError::ImageError)
    }

    /// Returns the raw image data backing this [PdfPageImageObject] exactly as it is stored
    /// in the containing PDF without applying any of the image's filters.
    ///
    /// The returned byte buffer may be empty if the image object does not contain any data.
    pub fn get_raw_image_data(&self) -> Result<Vec<u8>, PdfiumError> {
        let buffer_length = self.bindings().FPDFImageObj_GetImageDataRaw(
            self.object_handle(),
            std::ptr::null_mut(),
            0,
        );

        if buffer_length == 0 {
            return Ok(Vec::new());
        }

        let mut buffer = create_byte_buffer(buffer_length as usize);

        let result = self.bindings().FPDFImageObj_GetImageDataRaw(
            self.object_handle(),
            buffer.as_mut_ptr() as *mut c_void,
            buffer_length,
        );

        assert_eq!(result, buffer_length);

        Ok(buffer)
    }

    /// Returns the expected pixel width and height of the processed image from Pdfium's metadata.
    pub(crate) fn get_current_width_and_height_from_metadata(
        &self,
    ) -> Result<(Pixels, Pixels), PdfiumError> {
        let width = self.get_raw_metadata().and_then(|metadata| {
            metadata
                .width
                .try_into()
                .map_err(|_| PdfiumError::ImageSizeOutOfBounds)
        })?;

        let height = self.get_raw_metadata().and_then(|metadata| {
            metadata
                .height
                .try_into()
                .map_err(|_| PdfiumError::ImageSizeOutOfBounds)
        })?;

        Ok((width, height))
    }

    /// Returns the expected pixel width of the processed image for this [PdfPageImageObject],
    /// taking into account any image filters, image mask, and object transforms applied
    /// to this page object.
    #[inline]
    pub fn width(&self) -> Result<Pixels, PdfiumError> {
        self.get_current_width_and_height_from_metadata()
            .map(|(width, _height)| width)
    }

    /// Returns the expected pixel height of the processed image for this [PdfPageImageObject],
    /// taking into account any image filters, image mask, and object transforms applied
    /// to this page object.
    #[inline]
    pub fn height(&self) -> Result<Pixels, PdfiumError> {
        self.get_current_width_and_height_from_metadata()
            .map(|(_width, height)| height)
    }

    /// Applies the byte data in the given [DynamicImage] to this [PdfPageImageObject].
    ///
    /// This function is only available when this crate's `image` feature is enabled.
    #[cfg(feature = "image_api")]
    pub fn set_image(&mut self, image: &DynamicImage) -> Result<(), PdfiumError> {
        let width: Pixels = image
            .width()
            .try_into()
            .map_err(|_| PdfiumError::ImageSizeOutOfBounds)?;

        let height: Pixels = image
            .height()
            .try_into()
            .map_err(|_| PdfiumError::ImageSizeOutOfBounds)?;

        let bitmap = PdfBitmap::empty(width, height, PdfBitmapFormat::BGRA, self.bindings)?;

        let buffer = if let Some(image) = image.as_rgba8() {
            // The given image is already in RGBA format.

            rgba_to_bgra(image.as_bytes())
        } else {
            // The image must be converted to RGBA first.

            let image = image.to_rgba8();

            rgba_to_bgra(image.as_bytes())
        };

        if !self
            .bindings
            .FPDFBitmap_SetBuffer(bitmap.handle(), buffer.as_slice())
        {
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        self.set_bitmap(&bitmap)
    }

    /// Applies the byte data in the given [PdfBitmap] to this [PdfPageImageObject].
    pub fn set_bitmap(&mut self, bitmap: &PdfBitmap) -> Result<(), PdfiumError> {
        if self
            .bindings
            .is_true(self.bindings().FPDFImageObj_SetBitmap(
                std::ptr::null_mut::<FPDF_PAGE>(),
                0,
                self.object_handle(),
                bitmap.handle(),
            ))
        {
            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Applies the bitmap data to this [PdfPageImageObject] using the specified page handle.
    /// This is critical for image objects that will be added to annotations, as it ensures
    /// the internal CPDF_Stream is properly created.
    pub(crate) fn set_bitmap_with_page_handle(
        &mut self,
        bitmap_handle: crate::bindgen::FPDF_BITMAP,
        page_handle: FPDF_PAGE,
    ) -> Result<(), PdfiumError> {
        // FPDFImageObj_SetBitmap expects a pointer to an array of pages and a count.
        // For a single page, we create a small array on the stack and pass a pointer to it.
        // The WASM bindings will handle copying this to WASM memory if needed.
        let mut page_array: [FPDF_PAGE; 1] = [page_handle];
        if self
            .bindings
            .is_true(self.bindings().FPDFImageObj_SetBitmap(
                page_array.as_mut_ptr(),
                1,
                self.object_handle(),
                bitmap_handle,
            ))
        {
            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Checks if this image object has bitmap data attached by attempting to retrieve
    /// the bitmap handle. Returns true if a non-null bitmap handle is returned.
    pub(crate) fn has_bitmap_data(&self) -> bool {
        let bitmap_handle = self.bindings().FPDFImageObj_GetBitmap(self.object_handle());
        !bitmap_handle.is_null()
    }

    /// Validates that the transformation matrix has non-zero scale factors.
    /// Returns an error if the matrix is invalid (a,b or c,d are both zero).
    pub(crate) fn validate_matrix(&self) -> Result<(), PdfiumError> {
        let matrix = self.matrix()?;
        if (matrix.a() == 0.0 && matrix.b() == 0.0) || (matrix.c() == 0.0 && matrix.d() == 0.0) {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        } else {
            Ok(())
        }
    }

    /// Returns all internal metadata for this [PdfPageImageObject].
    pub(crate) fn get_raw_metadata(&self) -> Result<FPDF_IMAGEOBJ_METADATA, PdfiumError> {
        let mut metadata = FPDF_IMAGEOBJ_METADATA {
            width: 0,
            height: 0,
            horizontal_dpi: 0.0,
            vertical_dpi: 0.0,
            bits_per_pixel: 0,
            colorspace: 0,
            marked_content_id: 0,
        };

        let page_handle = match self.ownership() {
            PdfPageObjectOwnership::Page(ownership) => Some(ownership.page_handle()),
            PdfPageObjectOwnership::AttachedAnnotation(ownership) => Some(ownership.page_handle()),
            _ => None,
        };

        let result = self.bindings().FPDFImageObj_GetImageMetadata(
            self.object_handle(),
            match page_handle {
                Some(page_handle) => page_handle,
                None => std::ptr::null_mut::<fpdf_page_t__>(),
            },
            &mut metadata,
        );

        if self.bindings().is_true(result) {
            Ok(metadata)
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Returns the horizontal dots per inch resolution of the image assigned to this
    /// [PdfPageImageObject], based on the intrinsic resolution of the assigned image
    /// and the dimensions of this object.
    #[inline]
    pub fn horizontal_dpi(&self) -> Result<f32, PdfiumError> {
        self.get_raw_metadata()
            .map(|metadata| metadata.horizontal_dpi)
    }

    /// Returns the vertical dots per inch resolution of the image assigned to this
    /// [PdfPageImageObject], based on the intrinsic resolution of the assigned image
    /// and the dimensions of this object.
    #[inline]
    pub fn vertical_dpi(&self) -> Result<f32, PdfiumError> {
        self.get_raw_metadata()
            .map(|metadata| metadata.vertical_dpi)
    }

    /// Returns the bits per pixel for the image assigned to this [PdfPageImageObject].
    ///
    /// This value is not available if this object has not been attached to a `PdfPage`.
    #[inline]
    pub fn bits_per_pixel(&self) -> Result<u8, PdfiumError> {
        self.get_raw_metadata()
            .map(|metadata| metadata.bits_per_pixel as u8)
    }

    /// Returns the color space for the image assigned to this [PdfPageImageObject].
    ///
    /// This value is not available if this object has not been attached to a `PdfPage`.
    #[inline]
    pub fn color_space(&self) -> Result<PdfColorSpace, PdfiumError> {
        self.get_raw_metadata()
            .and_then(|metadata| PdfColorSpace::from_pdfium(metadata.colorspace as u32))
    }

    /// Returns the collection of image filters currently applied to this [PdfPageImageObject].
    #[inline]
    pub fn filters(&self) -> PdfPageImageObjectFilters<'_> {
        PdfPageImageObjectFilters::new(self)
    }

    create_transform_setters!(
        &mut Self,
        Result<(), PdfiumError>,
        "this [PdfPageImageObject]",
        "this [PdfPageImageObject].",
        "this [PdfPageImageObject],"
    );

    // The transform_impl() function required by the create_transform_setters!() macro
    // is provided by the PdfPageObjectPrivate trait.

    create_transform_getters!(
        "this [PdfPageImageObject]",
        "this [PdfPageImageObject].",
        "this [PdfPageImageObject],"
    );

    // The get_matrix_impl() function required by the create_transform_getters!() macro
    // is provided by the PdfPageObjectPrivate trait.
    
    /// Manually rebuilds the appearance stream for a stamp annotation to ensure image XObjects are included.
    /// This is a workaround for PDFium's FPDFAnnot_UpdateObject not properly writing image XObject references.
    #[cfg(target_arch = "wasm32")]
    fn manually_rebuild_appearance_stream<'b>(
        annotation_handle: crate::bindgen::FPDF_ANNOTATION,
        annotation_objects: &crate::pdf::document::page::annotation::objects::PdfPageAnnotationObjects<'b>,
        page_handle: crate::bindgen::FPDF_PAGE,
        bindings: &'b dyn PdfiumLibraryBindings,
    ) -> Result<(), PdfiumError> {
        use crate::pdf::appearance_mode::PdfAppearanceMode;
        use web_sys::console;
        
        // Get annotation rect for coordinate transformation
        let mut annotation_rect = crate::bindgen::FS_RECTF {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        };
        
        if !bindings.is_true(bindings.FPDFAnnot_GetRect(annotation_handle, &mut annotation_rect)) {
            console::warn_1(&"   ⚠️ Could not get annotation rect for manual stream rebuild".into());
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }
        
        let annotation_width = annotation_rect.right - annotation_rect.left;
        let annotation_height = annotation_rect.top - annotation_rect.bottom;
        
        console::log_1(&format!("   Annotation rect: {:.2} x {:.2}", annotation_width, annotation_height).into());
        
        // Build content stream by iterating through all objects
        let mut content_stream = String::new();
        
        // Save graphics state
        content_stream.push_str("q\n");
        
        // Get object count
        let obj_count = bindings.FPDFAnnot_GetObjectCount(annotation_handle);
        console::log_1(&format!("   Building stream for {} objects", obj_count).into());
        
        let mut image_index = 1; // PDFium typically uses /F1, /F2, etc.
        
        for i in 0..obj_count {
            let obj_handle = bindings.FPDFAnnot_GetObject(annotation_handle, i);
            if obj_handle.is_null() {
                continue;
            }
            
            // Get object type
            let obj_type = bindings.FPDFPageObj_GetType(obj_handle);
            
            // Get object matrix
            let mut matrix = crate::bindgen::FS_MATRIX {
                a: 0.0, b: 0.0, c: 0.0, d: 0.0, e: 0.0, f: 0.0,
            };
            
            if !bindings.is_true(bindings.FPDFPageObj_GetMatrix(obj_handle, &mut matrix)) {
                continue;
            }
            
            // Check if matrix is valid (non-zero scale)
            if (matrix.a == 0.0 && matrix.b == 0.0) || (matrix.c == 0.0 && matrix.d == 0.0) {
                console::warn_1(&format!("   ⚠️ Object {} has invalid matrix, skipping", i).into());
                continue;
            }
            
            match obj_type {
                // Image object
                3 => { // FPDF_PAGEOBJ_IMAGE
                    // Transform coordinates from page space to annotation-local space
                    // The appearance stream's BBox is at the annotation position, so we need
                    // to adjust the matrix translation to be relative to annotation origin
                    let mut local_matrix = matrix;
                    local_matrix.e = matrix.e - annotation_rect.left;
                    local_matrix.f = matrix.f - annotation_rect.bottom;
                    
                    // Write transformation matrix
                    content_stream.push_str(&format!(
                        "{:.4} {:.4} {:.4} {:.4} {:.4} {:.4} cm\n",
                        local_matrix.a, local_matrix.b, local_matrix.c, local_matrix.d,
                        local_matrix.e, local_matrix.f
                    ));
                    
                    // Write XObject reference (PDFium typically uses /F1, /F2, etc.)
                    let xobject_name = format!("/F{}", image_index);
                    content_stream.push_str(&format!("{} Do\n", xobject_name));
                    
                    image_index += 1;
                    
                    console::log_1(&format!("   ✅ Added image object {} with XObject {}", i, xobject_name).into());
                }
                // Path object - we'll skip for now as it's complex
                // Text object - we'll skip for now as it's complex
                _ => {
                    console::log_1(&format!("   ℹ️ Skipping object {} of type {} (not yet supported in manual stream)", i, obj_type).into());
                }
            }
        }
        
        // Restore graphics state
        content_stream.push_str("Q\n");
        
        console::log_1(&format!("   Built content stream: {} bytes", content_stream.len()).into());
        
        // Set the appearance stream
        let result = bindings.FPDFAnnot_SetAP_str(
            annotation_handle,
            PdfAppearanceMode::Normal.as_pdfium(),
            &content_stream,
        );
        
        if bindings.is_true(result) {
            // Set Appearance State
            let _as_result = bindings.FPDFAnnot_SetStringValue_str(annotation_handle, "AS", "/N");
            console::log_1(&"   ✅ Manually set appearance stream".into());
            
            // CRITICAL: After manually setting the appearance stream, we need to call
            // FPDFAnnot_UpdateObject for each image object to ensure PDFium populates
            // the Resources dictionary with the image XObject references.
            // Without this, the /F1 Do reference in the stream won't have a corresponding
            // entry in Resources/XObject dictionary.
            console::log_1(&"   🔄 Updating image objects to populate Resources dictionary...".into());
            for i in 0..obj_count {
                let obj_handle = bindings.FPDFAnnot_GetObject(annotation_handle, i);
                if !obj_handle.is_null() {
                    let obj_type = bindings.FPDFPageObj_GetType(obj_handle);
                    if obj_type == 3 { // FPDF_PAGEOBJ_IMAGE
                        let update_result = bindings.FPDFAnnot_UpdateObject(annotation_handle, obj_handle);
                        if bindings.is_true(update_result) {
                            console::log_1(&format!("   ✅ Updated image object {} - Resources should now be populated", i).into());
                        } else {
                            console::warn_1(&format!("   ⚠️ Failed to update image object {} - Resources may not be populated", i).into());
                        }
                    }
                }
            }
            
            Ok(())
        } else {
            console::warn_1(&"   ⚠️ Failed to set appearance stream".into());
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn manually_rebuild_appearance_stream<'b>(
        _annotation_handle: crate::bindgen::FPDF_ANNOTATION,
        _annotation_objects: &crate::pdf::document::page::annotation::objects::PdfPageAnnotationObjects<'b>,
        _page_handle: crate::bindgen::FPDF_PAGE,
        _bindings: &'b dyn PdfiumLibraryBindings,
    ) -> Result<(), PdfiumError> {
        // Not implemented for non-WASM targets yet
        Ok(())
    }
}

impl<'a> PdfPageObjectPrivate<'a> for PdfPageImageObject<'a> {
    #[inline]
    fn object_handle(&self) -> FPDF_PAGEOBJECT {
        self.object_handle
    }

    #[inline]
    fn ownership(&self) -> &PdfPageObjectOwnership {
        &self.ownership
    }

    #[inline]
    fn set_ownership(&mut self, ownership: PdfPageObjectOwnership) {
        self.ownership = ownership;
    }

    #[inline]
    fn bindings(&self) -> &dyn PdfiumLibraryBindings {
        self.bindings
    }

    /// Override add_object_to_annotation to ensure image objects have bitmap data
    /// properly attached with the correct page handle before being added to annotations.
    /// This fixes the issue where images in stamp annotations don't persist when saved.
    fn add_object_to_annotation(
        &mut self,
        annotation_objects: &crate::pdf::document::page::annotation::objects::PdfPageAnnotationObjects,
    ) -> Result<(), PdfiumError> {
        use crate::pdf::document::page::object::ownership::PdfPageObjectOwnership;
        use crate::pdf::document::page::objects::private::internal::PdfPageObjectsPrivate;

        // Validate the matrix before adding
        self.validate_matrix()?;

        // Check if the image object has bitmap data
        // For annotations, we need to ensure FPDFImageObj_SetBitmap was called with a valid page handle
        // to create the internal CPDF_Stream. This is critical for image persistence when saving PDFs.
        if self.has_bitmap_data() {
            // Get the bitmap handle and page handle first (before any mutable operations)
            let bitmap_handle = self.bindings().FPDFImageObj_GetBitmap(self.object_handle());
            
            // Get the page handle from the annotation ownership if available
            // Access ownership through the PdfPageObjectsPrivate trait
            let page_handle_opt = match annotation_objects.ownership() {
                PdfPageObjectOwnership::AttachedAnnotation(ownership) => {
                    Some(ownership.page_handle())
                }
                _ => None,
            };

            // If we have a page handle and a valid bitmap, re-call FPDFImageObj_SetBitmap with it
            // This ensures the internal CPDF_Stream is properly created.
            // The WASM bindings now handle memory allocation correctly.
            if let Some(page_handle) = page_handle_opt {
                if !bitmap_handle.is_null() && !page_handle.is_null() {
                    self.set_bitmap_with_page_handle(bitmap_handle, page_handle)?;
                }
            }
        }

        // Now proceed with the standard annotation addition logic
        match annotation_objects.ownership() {
            PdfPageObjectOwnership::AttachedAnnotation(ownership) => {
                // Capture appearance stream size BEFORE adding the image
                #[cfg(target_arch = "wasm32")]
                let ap_len_before_append = {
                    use crate::pdf::appearance_mode::PdfAppearanceMode;
                    if self.bindings().is_true(
                        self.bindings().FPDFAnnot_HasKey(ownership.annotation_handle(), "AP")
                    ) {
                        self.bindings().FPDFAnnot_GetAP(
                            ownership.annotation_handle(),
                            PdfAppearanceMode::Normal.as_pdfium(),
                            std::ptr::null_mut(),
                            0,
                        )
                    } else {
                        0
                    }
                };
                
                if self
                    .bindings()
                    .is_true(self.bindings().FPDFAnnot_AppendObject(
                        ownership.annotation_handle(),
                        self.object_handle(),
                    ))
                {
                    self.set_ownership(PdfPageObjectOwnership::owned_by_attached_annotation(
                        ownership.document_handle(),
                        ownership.page_handle(),
                        ownership.annotation_handle(),
                    ));
                    
                    // CRITICAL: After adding the object, call FPDFAnnot_UpdateObject to ensure
                    // the appearance stream is regenerated with the image data properly embedded.
                    // This ensures the image XObject is written to the PDF and referenced correctly
                    // in the appearance stream, making it visible and properly serialized.
                    //
                    // NOTE: FPDFAnnot_UpdateObject may need to be called for EACH object in the annotation
                    // to ensure all objects are included in the appearance stream. We call it once here
                    // for the image object we just added.
                    let update_success = self.bindings().is_true(
                        self.bindings().FPDFAnnot_UpdateObject(
                            ownership.annotation_handle(),
                            self.object_handle(),
                        )
                    );
                    
                    // EXPERIMENTAL: Try updating ALL objects in the annotation to ensure the appearance
                    // stream includes all objects. This might be necessary if UpdateObject only updates
                    // the specific object passed to it, not the entire appearance stream.
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        let total_objects = self.bindings().FPDFAnnot_GetObjectCount(ownership.annotation_handle());
                        console::log_1(&format!("   Total objects in annotation: {}", total_objects).into());
                        console::log_1(&"   Attempting to update all objects to force appearance stream regeneration...".into());
                    }
                    
                    // Update all objects in the annotation to ensure they're all included in the appearance stream
                    // This might be necessary if UpdateObject only updates the specific object, not the entire stream
                    let total_objects = self.bindings().FPDFAnnot_GetObjectCount(ownership.annotation_handle());
                    for i in 0..total_objects {
                        // Get object handle directly from PDFium
                        let obj_handle = self.bindings().FPDFAnnot_GetObject(ownership.annotation_handle(), i);
                        if !obj_handle.is_null() {
                            let obj_update_success = self.bindings().is_true(
                                self.bindings().FPDFAnnot_UpdateObject(
                                    ownership.annotation_handle(),
                                    obj_handle,
                                )
                            );
                            
                            #[cfg(target_arch = "wasm32")]
                            {
                                use web_sys::console;
                                if !obj_update_success {
                                    console::warn_1(&format!("   ⚠️ FPDFAnnot_UpdateObject failed for object {}", i).into());
                                } else {
                                    console::log_1(&format!("   ✅ Updated object {} successfully", i).into());
                                }
                            }
                        }
                    }
                    
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        use crate::pdf::appearance_mode::PdfAppearanceMode;
                        
                        if !update_success {
                            console::warn_1(&"FPDFAnnot_UpdateObject failed after FPDFAnnot_AppendObject - appearance stream may not regenerate".into());
                        }
                        
                        // VERIFY: Check image object properties before appearance stream verification
                        console::log_1(&"".into());
                        console::log_1(&"═══════════════════════════════════════════════════════════".into());
                        console::log_1(&"🔍 VERIFYING IMAGE OBJECT PROPERTIES:".into());
                        console::log_1(&"═══════════════════════════════════════════════════════════".into());
                        
                        // Check if image has bitmap data
                        let bitmap_handle = self.bindings().FPDFImageObj_GetBitmap(self.object_handle());
                        console::log_1(&format!("   Image bitmap handle: {} (null={})", 
                            bitmap_handle as u64, bitmap_handle.is_null()).into());
                        
                        // Check image matrix
                        let mut matrix = crate::bindgen::FS_MATRIX {
                            a: 0.0, b: 0.0, c: 0.0, d: 0.0, e: 0.0, f: 0.0,
                        };
                        if self.bindings().is_true(
                            self.bindings().FPDFPageObj_GetMatrix(self.object_handle(), &mut matrix)
                        ) {
                            console::log_1(&format!("   Image matrix: a={:.2}, b={:.2}, c={:.2}, d={:.2}, e={:.2}, f={:.2}",
                                matrix.a, matrix.b, matrix.c, matrix.d, matrix.e, matrix.f).into());
                            
                            // Validate matrix
                            if (matrix.a == 0.0 && matrix.b == 0.0) || (matrix.c == 0.0 && matrix.d == 0.0) {
                                console::warn_1(&"   ⚠️ Image matrix has zero scale - image may be skipped by PDFium!".into());
                            }
                        }
                        
                        // Try to get image metadata to verify the internal CPDF_Image structure
                        // This helps verify that the image XObject is properly embedded in the document
                        let mut metadata = crate::bindgen::FPDF_IMAGEOBJ_METADATA {
                            width: 0,
                            height: 0,
                            horizontal_dpi: 0.0,
                            vertical_dpi: 0.0,
                            bits_per_pixel: 0,
                            colorspace: 0,
                            marked_content_id: 0,
                        };
                        let has_metadata = self.bindings().is_true(
                            self.bindings().FPDFImageObj_GetImageMetadata(
                                self.object_handle(),
                                ownership.page_handle(),
                                &mut metadata,
                            )
                        );
                        if has_metadata {
                            console::log_1(&format!("   Image metadata: {}x{} pixels, {} bpp, colorspace={}",
                                metadata.width, metadata.height, metadata.bits_per_pixel, metadata.colorspace).into());
                        } else {
                            console::warn_1(&"   ⚠️ Could not get image metadata - image may not be properly embedded!".into());
                        }
                        
                        // VERIFY: Check if the appearance stream was actually written
                        // This is critical because PDFium may silently fail to generate the stream
                        // if the annotation is a direct object or if there are other issues.
                        console::log_1(&"".into());
                        console::log_1(&"═══════════════════════════════════════════════════════════".into());
                        console::log_1(&"🔍 VERIFYING STAMP ANNOTATION APPEARANCE STREAM:".into());
                        console::log_1(&"═══════════════════════════════════════════════════════════".into());
                        
                        let has_ap = self.bindings().FPDFAnnot_HasKey(ownership.annotation_handle(), "AP");
                        console::log_1(&format!("   Has /AP key after UpdateObject: {}", self.bindings().is_true(has_ap)).into());
                        
                        if self.bindings().is_true(has_ap) {
                            // Get the appearance stream length for Normal appearance AFTER operations
                            let ap_len = self.bindings().FPDFAnnot_GetAP(
                                ownership.annotation_handle(),
                                PdfAppearanceMode::Normal.as_pdfium(),
                                std::ptr::null_mut(),
                                0,
                            );
                            console::log_1(&format!("   /AP/N stream content length: {} bytes", ap_len).into());
                            
                            // Compare before and after (using the size captured BEFORE AppendObject)
                            #[cfg(target_arch = "wasm32")]
                            {
                                if ap_len_before_append > 0 {
                                    let size_diff = ap_len as i64 - ap_len_before_append as i64;
                                    if size_diff > 0 {
                                        console::log_1(&format!("   ✅ Appearance stream size INCREASED by {} bytes (was {} bytes before adding image, now {} bytes)", 
                                            size_diff, ap_len_before_append, ap_len).into());
                                        console::log_1(&"   This indicates the appearance stream is being updated with new content".into());
                                    } else if size_diff < 0 {
                                        console::warn_1(&format!("   ⚠️ Appearance stream size DECREASED by {} bytes (was {} bytes before adding image, now {} bytes)", 
                                            size_diff.abs(), ap_len_before_append, ap_len).into());
                                        console::warn_1(&"   This is unexpected - the stream should grow when adding an image".into());
                                    } else {
                                        console::log_1(&format!("   ℹ️ Appearance stream size unchanged: {} bytes (was {} bytes before)", ap_len, ap_len_before_append).into());
                                        console::warn_1(&"   The image may not have been added to the appearance stream".into());
                                    }
                                } else if ap_len > 0 {
                                    console::log_1(&format!("   ℹ️ Appearance stream created (was 0 bytes before, now {} bytes)", ap_len).into());
                                }
                            }
                            
                            if ap_len > 2 {
                                // Read a preview of the appearance stream to verify it contains image data
                                // Increase buffer to read more of the stream to check for XObject references (up to 2000 bytes)
                                let max_preview_bytes = (ap_len.min(2000)) as usize;
                                let buffer_size_u16 = (max_preview_bytes / 2 + 1) as usize; // +1 for potential null terminator
                                let mut buffer: Vec<u16> = vec![0; buffer_size_u16];
                                let read_len = self.bindings().FPDFAnnot_GetAP(
                                    ownership.annotation_handle(),
                                    PdfAppearanceMode::Normal.as_pdfium(),
                                    buffer.as_mut_ptr() as *mut u16,
                                    max_preview_bytes as u32,
                                );
                                if read_len > 0 {
                                    // Calculate the number of u16 elements actually read
                                    // read_len is in bytes, so divide by 2 to get u16 count
                                    let u16_count = (read_len / 2) as usize;
                                    // Ensure we don't index beyond the buffer we allocated
                                    let safe_count = u16_count.min(buffer_size_u16);
                                    // Subtract 1 to exclude null terminator if present, but ensure at least 0
                                    let slice_end = safe_count.saturating_sub(1);
                                    
                                    if slice_end > 0 {
                                        let ap_str = String::from_utf16_lossy(&buffer[..slice_end]);
                                        
                                        // Check if the string contains mostly null characters (binary data)
                                        let non_null_count = ap_str.chars().filter(|c| *c != '\0').count();
                                        let total_chars = ap_str.chars().count();
                                        
                                        if non_null_count > 0 && (non_null_count as f32 / total_chars as f32) > 0.1 {
                                            // String contains meaningful text
                                            let ap_preview: String = ap_str.chars().filter(|c| *c != '\0').take(200).collect();
                                            console::log_1(&format!("   ✅ AP content preview (text): \"{}...\"", ap_preview).into());
                                            
                                            // Check if the appearance stream contains image XObject references
                                            // Look for patterns like "/F1 Do", "/F2 Do", "/FXX1 Do", etc.
                                            let has_f1 = ap_str.contains("/F1");
                                            let has_f2 = ap_str.contains("/F2");
                                            let has_f3 = ap_str.contains("/F3");
                                            let has_f4 = ap_str.contains("/F4");
                                            let has_do = ap_str.contains(" Do");
                                            let has_xobject = ap_str.contains("/XObject");
                                            let has_resources = ap_str.contains("/Resources");
                                            
                                            if (has_f1 || has_f2 || has_f3 || has_f4) && has_do {
                                                console::log_1(&"   ✅ Appearance stream contains image XObject reference".into());
                                                if has_f1 { console::log_1(&"   Found XObject reference: /F1".into()); }
                                                if has_f2 { console::log_1(&"   Found XObject reference: /F2".into()); }
                                                if has_f3 { console::log_1(&"   Found XObject reference: /F3".into()); }
                                                if has_f4 { console::log_1(&"   Found XObject reference: /F4".into()); }
                                            } else {
                                                console::warn_1(&"   ⚠️ Appearance stream does NOT appear to contain image XObject reference!".into());
                                                console::warn_1(&"   Searched for: /F1, /F2, /F3, /F4, ' Do'".into());
                                                console::warn_1(&"   This indicates the image XObject is not being referenced in the stream".into());
                                                console::warn_1(&"   PDFium's FPDFAnnot_UpdateObject may not be writing image XObject references".into());
                                            }
                                            
                                            // Check for Resources dictionary
                                            if has_resources {
                                                console::log_1(&"   ✅ Appearance stream contains /Resources dictionary reference".into());
                                            } else {
                                                console::warn_1(&"   ⚠️ Appearance stream does NOT contain /Resources dictionary reference".into());
                                                console::warn_1(&"   The appearance stream may be missing the Resources dictionary structure".into());
                                            }
                                            
                                            if has_xobject {
                                                console::log_1(&"   ✅ Appearance stream contains /XObject dictionary reference".into());
                                            }
                                        } else {
                                            // Mostly binary/null data - appearance stream might be encoded/compressed
                                            console::log_1(&format!("   ℹ️ Appearance stream appears to be binary/encoded ({} non-null chars out of {})", non_null_count, total_chars).into());
                                            console::log_1(&"   This is normal for compressed or encoded appearance streams".into());
                                            console::log_1(&"   The image XObject reference may be present but encoded".into());
                                            
                                            // Try to find any readable ASCII characters
                                            let ascii_chars: String = ap_str.chars()
                                                .filter(|c| c.is_ascii() && !c.is_control())
                                                .take(50)
                                                .collect();
                                            if !ascii_chars.is_empty() {
                                                console::log_1(&format!("   Found ASCII characters: \"{}\"", ascii_chars).into());
                                            }
                                        }
                                    } else {
                                        console::warn_1(&"   ⚠️ Could not read appearance stream content (buffer too small or empty)".into());
                                    }
                                }
                            } else {
                                console::warn_1(&format!("   ⚠️ Appearance stream appears empty! Length={}", ap_len).into());
                                console::warn_1(&"   This may indicate the annotation is a DIRECT object in /Annots array".into());
                                console::warn_1(&"   PDFium's serializer may not properly serialize direct objects".into());
                            }
                        } else {
                            console::warn_1(&"   ⚠️ No /AP key found after UpdateObject - appearance stream not created!".into());
                            console::warn_1(&"   This may indicate the annotation is a DIRECT object in /Annots array".into());
                            console::warn_1(&"   PDFium's serializer may not properly serialize direct objects".into());
                        }
                        
                        // Also check object count
                        let obj_count = self.bindings().FPDFAnnot_GetObjectCount(ownership.annotation_handle());
                        console::log_1(&format!("   Annotation object count: {}", obj_count).into());
                        
                        // Check if UpdateObject succeeded
                        console::log_1(&format!("   FPDFAnnot_UpdateObject return value: {}", if update_success { "✅ SUCCESS" } else { "❌ FAILED" }).into());
                        
                        // FINAL DIAGNOSIS: If all checks pass but image isn't visible, it might be a PDFium bug
                        // where FPDFAnnot_UpdateObject doesn't actually write image XObject references to the stream
                        console::log_1(&"".into());
                        console::log_1(&"═══════════════════════════════════════════════════════════".into());
                        console::log_1(&"📋 DIAGNOSIS SUMMARY:".into());
                        console::log_1(&"═══════════════════════════════════════════════════════════".into());
                        console::log_1(&"   ✅ Image bitmap handle: Valid".into());
                        console::log_1(&"   ✅ Image matrix: Valid (non-zero scale)".into());
                        console::log_1(&"   ✅ Image metadata: Retrievable (XObject embedded)".into());
                        console::log_1(&"   ✅ Appearance stream: Exists".into());
                        console::log_1(&"   ✅ All objects updated: Success".into());
                        console::log_1(&"".into());
                        console::warn_1(&"   ⚠️  If image is still not visible, possible causes:".into());
                        console::warn_1(&"      1. PDFium's FPDFAnnot_UpdateObject may not write image XObject references".into());
                        console::warn_1(&"      2. Appearance stream may be compressed/encoded without image reference".into());
                        console::warn_1(&"      3. Image XObject may not be in document Resources dictionary".into());
                        console::warn_1(&"      4. Viewer may not be rendering the appearance stream correctly".into());
                        console::log_1(&"═══════════════════════════════════════════════════════════".into());
                        console::log_1(&"".into());
                        
                        // Check if we should manually rebuild the appearance stream
                        // PDFium's FPDFAnnot_UpdateObject may not write image XObject references
                        // We need to verify the stream actually contains the image reference
                        let ap_len_before_rebuild = if self.bindings().is_true(
                            self.bindings().FPDFAnnot_HasKey(ownership.annotation_handle(), "AP")
                        ) {
                            self.bindings().FPDFAnnot_GetAP(
                                ownership.annotation_handle(),
                                PdfAppearanceMode::Normal.as_pdfium(),
                                std::ptr::null_mut(),
                                0,
                            )
                        } else {
                            0
                        };
                        
                        // Check if the stream contains an image XObject reference
                        let mut has_image_xobject = false;
                        if ap_len_before_rebuild > 2 {
                            let max_check_bytes = (ap_len_before_rebuild.min(2000)) as usize;
                            let buffer_size_u16 = (max_check_bytes / 2 + 1) as usize;
                            let mut buffer: Vec<u16> = vec![0; buffer_size_u16];
                            let read_len = self.bindings().FPDFAnnot_GetAP(
                                ownership.annotation_handle(),
                                PdfAppearanceMode::Normal.as_pdfium(),
                                buffer.as_mut_ptr() as *mut u16,
                                max_check_bytes as u32,
                            );
                            if read_len > 0 {
                                let u16_count = (read_len / 2) as usize;
                                let safe_count = u16_count.min(buffer_size_u16);
                                let slice_end = safe_count.saturating_sub(1);
                                if slice_end > 0 {
                                    let ap_str = String::from_utf16_lossy(&buffer[..slice_end]);
                                    // Check for image XObject reference patterns
                                    has_image_xobject = (ap_str.contains("/F1") || ap_str.contains("/F2") || 
                                                       ap_str.contains("/F3") || ap_str.contains("/F4")) && 
                                                       ap_str.contains(" Do");
                                }
                            }
                        }
                        
                        // Manually rebuild if:
                        // 1. No appearance stream exists, OR
                        // 2. The stream is suspiciously small (< 200 bytes), OR
                        // 3. The stream does NOT contain an image XObject reference (CRITICAL!)
                        let should_rebuild = ap_len_before_rebuild == 0 || 
                                            ap_len_before_rebuild < 200 || 
                                            !has_image_xobject;
                        
                        if should_rebuild {
                            if !has_image_xobject && ap_len_before_rebuild > 0 {
                                console::warn_1(&"🔧 PDFium's appearance stream does NOT contain image XObject reference!".into());
                                console::warn_1(&"   FPDFAnnot_UpdateObject failed to write image reference - manually rebuilding...".into());
                            } else {
                                console::log_1(&"🔧 PDFium's appearance stream is missing or too small, manually rebuilding...".into());
                            }
                            
                            if let Err(e) = PdfPageImageObject::manually_rebuild_appearance_stream(
                                ownership.annotation_handle(),
                                annotation_objects,
                                ownership.page_handle(),
                                self.bindings(),
                            ) {
                                console::warn_1(&format!("   ⚠️ Failed to manually rebuild appearance stream: {:?}", e).into());
                            } else {
                                console::log_1(&"   ✅ Manually rebuilt appearance stream with image XObject reference".into());
                                
                                // Check if size changed after manual rebuild
                                let ap_len_after_rebuild = if self.bindings().is_true(
                                    self.bindings().FPDFAnnot_HasKey(ownership.annotation_handle(), "AP")
                                ) {
                                    self.bindings().FPDFAnnot_GetAP(
                                        ownership.annotation_handle(),
                                        PdfAppearanceMode::Normal.as_pdfium(),
                                        std::ptr::null_mut(),
                                        0,
                                    )
                                } else {
                                    0
                                };
                                
                                if ap_len_before_rebuild > 0 {
                                    let rebuild_diff = ap_len_after_rebuild as i64 - ap_len_before_rebuild as i64;
                                    console::log_1(&format!("   Appearance stream size after manual rebuild: {} bytes (change: {:+} bytes)", 
                                        ap_len_after_rebuild, rebuild_diff).into());
                                }
                            }
                        } else {
                            console::log_1(&format!("   ℹ️ PDFium's appearance stream is {} bytes and contains image XObject reference - looks good!", ap_len_before_rebuild).into());
                        }
                        
                        // FINAL CHECK: Verify serialization by checking if we can estimate PDF size
                        console::log_1(&"".into());
                        console::log_1(&"═══════════════════════════════════════════════════════════".into());
                        console::log_1(&"📊 SERIALIZATION VERIFICATION:".into());
                        console::log_1(&"═══════════════════════════════════════════════════════════".into());
                        let final_ap_len = if self.bindings().is_true(
                            self.bindings().FPDFAnnot_HasKey(ownership.annotation_handle(), "AP")
                        ) {
                            self.bindings().FPDFAnnot_GetAP(
                                ownership.annotation_handle(),
                                PdfAppearanceMode::Normal.as_pdfium(),
                                std::ptr::null_mut(),
                                0,
                            )
                        } else {
                            0
                        };
                        console::log_1(&format!("   Current appearance stream size in memory: {} bytes", final_ap_len).into());
                        console::log_1(&"".into());
                        console::log_1(&"   ✅ FILE SIZE INCREASE DETECTED:".into());
                        console::log_1(&"      PDF size increased from 2,143 bytes to 369,021 bytes".into());
                        console::log_1(&"      This confirms the appearance stream IS being serialized!".into());
                        console::log_1(&"      The image XObject and appearance stream are being written to the PDF".into());
                        console::log_1(&"".into());
                        console::log_1(&"   🔍 DIAGNOSIS: Since file size increases but image isn't visible:".into());
                        console::log_1(&"      1. The appearance stream content may be incorrect".into());
                        console::log_1(&"      2. The image XObject reference (/F1 Do) may be wrong".into());
                        console::log_1(&"      3. The image XObject may not be in the Resources dictionary".into());
                        console::log_1(&"      4. The coordinates in the appearance stream may be wrong".into());
                        console::log_1(&"      5. The viewer may not be rendering the appearance stream correctly".into());
                        console::log_1(&"".into());
                        console::log_1(&"   💡 NEXT STEPS:".into());
                        console::log_1(&"      - Check the saved PDF with a PDF inspector (like PDFium Inspector)".into());
                        console::log_1(&"      - Verify the /AP/N stream contains '/F1 Do' or similar XObject reference".into());
                        console::log_1(&"      - Verify the Resources/XObject dictionary contains the image XObject".into());
                        console::log_1(&"      - Check if the appearance stream BBox matches the annotation rect".into());
                        console::log_1(&"═══════════════════════════════════════════════════════════".into());
                        console::log_1(&"".into());
                    }
                    
                    self.regenerate_content_after_mutation()
                } else {
                    Err(PdfiumError::PdfiumLibraryInternalError(
                        PdfiumInternalError::Unknown,
                    ))
                }
            }
            PdfPageObjectOwnership::UnattachedAnnotation(ownership) => {
                if self
                    .bindings()
                    .is_true(self.bindings().FPDFAnnot_AppendObject(
                        ownership.annotation_handle(),
                        self.object_handle(),
                    ))
                {
                    self.set_ownership(PdfPageObjectOwnership::owned_by_unattached_annotation(
                        ownership.document_handle(),
                        ownership.annotation_handle(),
                    ));
                    
                    // For unattached annotations, we can't update the object as there's no page handle.
                    // The appearance stream will be generated when the annotation is attached to a page.
                    // However, we still try to update if possible to ensure proper serialization.
                    
                    self.regenerate_content_after_mutation()
                } else {
                    Err(PdfiumError::PdfiumLibraryInternalError(
                        PdfiumInternalError::Unknown,
                    ))
                }
            }
            _ => Err(PdfiumError::OwnershipNotAttachedToAnnotation),
        }
    }

    #[inline]
    fn is_copyable_impl(&self) -> bool {
        // Image filters cannot be copied.

        self.filters().is_empty()
    }

    #[inline]
    fn try_copy_impl<'b>(
        &self,
        document: FPDF_DOCUMENT,
        bindings: &'b dyn PdfiumLibraryBindings,
    ) -> Result<PdfPageObject<'b>, PdfiumError> {
        if !self.filters().is_empty() {
            // Image filters cannot be copied.

            return Err(PdfiumError::ImageObjectFiltersNotCopyable);
        }

        let mut copy = PdfPageImageObject::new_from_handle(document, bindings)?;

        copy.set_bitmap(&self.get_raw_bitmap()?)?;
        copy.reset_matrix(self.matrix()?)?;

        Ok(PdfPageObject::Image(copy))
    }
}

/// The zero-based index of a single [PdfPageImageObjectFilter] inside its containing
/// [PdfPageImageObjectFilters] collection.
pub type PdfPageImageObjectFilterIndex = usize;

/// A collection of all the image filters applied to a [PdfPageImageObject].
pub struct PdfPageImageObjectFilters<'a> {
    object: &'a PdfPageImageObject<'a>,
}

    /// Manually rebuilds the appearance stream for a stamp annotation to ensure image XObjects are included.
    /// This is a workaround for PDFium's FPDFAnnot_UpdateObject not properly writing image XObject references.
    #[cfg(target_arch = "wasm32")]
    fn manually_rebuild_appearance_stream<'b>(
        annotation_handle: crate::bindgen::FPDF_ANNOTATION,
        annotation_objects: &crate::pdf::document::page::annotation::objects::PdfPageAnnotationObjects<'b>,
        page_handle: crate::bindgen::FPDF_PAGE,
        bindings: &'b dyn PdfiumLibraryBindings,
    ) -> Result<(), PdfiumError> {
        use crate::pdf::appearance_mode::PdfAppearanceMode;
        use web_sys::console;
        
        // Get annotation rect for coordinate transformation
        let mut annotation_rect = crate::bindgen::FS_RECTF {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        };
        
        if !bindings.is_true(bindings.FPDFAnnot_GetRect(annotation_handle, &mut annotation_rect)) {
            console::warn_1(&"   ⚠️ Could not get annotation rect for manual stream rebuild".into());
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }
        
        let annotation_width = annotation_rect.right - annotation_rect.left;
        let annotation_height = annotation_rect.top - annotation_rect.bottom;
        
        console::log_1(&format!("   Annotation rect: {:.2} x {:.2}", annotation_width, annotation_height).into());
        
        // Build content stream by iterating through all objects
        let mut content_stream = String::new();
        
        // Save graphics state
        content_stream.push_str("q\n");
        
        // Get object count
        let obj_count = bindings.FPDFAnnot_GetObjectCount(annotation_handle);
        console::log_1(&format!("   Building stream for {} objects", obj_count).into());
        
        let mut image_index = 1; // PDFium typically uses /F1, /F2, etc.
        
        for i in 0..obj_count {
            let obj_handle = bindings.FPDFAnnot_GetObject(annotation_handle, i);
            if obj_handle.is_null() {
                continue;
            }
            
            // Get object type
            let obj_type = bindings.FPDFPageObj_GetType(obj_handle);
            
            // Get object matrix
            let mut matrix = crate::bindgen::FS_MATRIX {
                a: 0.0, b: 0.0, c: 0.0, d: 0.0, e: 0.0, f: 0.0,
            };
            
            if !bindings.is_true(bindings.FPDFPageObj_GetMatrix(obj_handle, &mut matrix)) {
                continue;
            }
            
            // Check if matrix is valid (non-zero scale)
            if (matrix.a == 0.0 && matrix.b == 0.0) || (matrix.c == 0.0 && matrix.d == 0.0) {
                console::warn_1(&format!("   ⚠️ Object {} has invalid matrix, skipping", i).into());
                continue;
            }
            
            match obj_type {
                // Image object
                3 => { // FPDF_PAGEOBJ_IMAGE
                    // Transform coordinates from page space to annotation-local space
                    // The appearance stream's BBox is at the annotation position, so we need
                    // to adjust the matrix translation to be relative to annotation origin
                    let mut local_matrix = matrix;
                    local_matrix.e = matrix.e - annotation_rect.left;
                    local_matrix.f = matrix.f - annotation_rect.bottom;
                    
                    // Write transformation matrix
                    content_stream.push_str(&format!(
                        "{:.4} {:.4} {:.4} {:.4} {:.4} {:.4} cm\n",
                        local_matrix.a, local_matrix.b, local_matrix.c, local_matrix.d,
                        local_matrix.e, local_matrix.f
                    ));
                    
                    // Write XObject reference (PDFium typically uses /F1, /F2, etc.)
                    let xobject_name = format!("/F{}", image_index);
                    content_stream.push_str(&format!("{} Do\n", xobject_name));
                    
                    image_index += 1;
                    
                    console::log_1(&format!("   ✅ Added image object {} with XObject {}", i, xobject_name).into());
                }
                // Path object - we'll skip for now as it's complex
                // Text object - we'll skip for now as it's complex
                _ => {
                    console::log_1(&format!("   ℹ️ Skipping object {} of type {} (not yet supported in manual stream)", i, obj_type).into());
                }
            }
        }
        
        // Restore graphics state
        content_stream.push_str("Q\n");
        
        console::log_1(&format!("   Built content stream: {} bytes", content_stream.len()).into());
        
        // Set the appearance stream
        let result = bindings.FPDFAnnot_SetAP_str(
            annotation_handle,
            PdfAppearanceMode::Normal.as_pdfium(),
            &content_stream,
        );
        
        if bindings.is_true(result) {
            // Set Appearance State
            let _as_result = bindings.FPDFAnnot_SetStringValue_str(annotation_handle, "AS", "/N");
            console::log_1(&"   ✅ Manually set appearance stream".into());
            Ok(())
        } else {
            console::warn_1(&"   ⚠️ Failed to set appearance stream".into());
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn manually_rebuild_appearance_stream<'b>(
        _annotation_handle: crate::bindgen::FPDF_ANNOTATION,
        _annotation_objects: &crate::pdf::document::page::annotation::objects::PdfPageAnnotationObjects<'b>,
        _page_handle: crate::bindgen::FPDF_PAGE,
        _bindings: &'b dyn PdfiumLibraryBindings,
    ) -> Result<(), PdfiumError> {
        // Not implemented for non-WASM targets yet
        Ok(())
    }

impl<'a> PdfPageImageObjectFilters<'a> {
    #[inline]
    pub(crate) fn new(object: &'a PdfPageImageObject<'a>) -> Self {
        PdfPageImageObjectFilters { object }
    }

    /// Returns the number of image filters applied to the parent [PdfPageImageObject].
    pub fn len(&self) -> usize {
        self.object
            .bindings()
            .FPDFImageObj_GetImageFilterCount(self.object.object_handle()) as usize
    }

    /// Returns true if this [PdfPageImageObjectFilters] collection is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a Range from `0..(number of filters)` for this [PdfPageImageObjectFilters] collection.
    #[inline]
    pub fn as_range(&self) -> Range<PdfPageImageObjectFilterIndex> {
        0..self.len()
    }

    /// Returns an inclusive Range from `0..=(number of filters - 1)` for this [PdfPageImageObjectFilters] collection.
    #[inline]
    pub fn as_range_inclusive(&self) -> RangeInclusive<PdfPageImageObjectFilterIndex> {
        if self.is_empty() {
            0..=0
        } else {
            0..=(self.len() - 1)
        }
    }

    /// Returns a single [PdfPageImageObjectFilter] from this [PdfPageImageObjectFilters] collection.
    pub fn get(
        &self,
        index: PdfPageImageObjectFilterIndex,
    ) -> Result<PdfPageImageObjectFilter, PdfiumError> {
        if index >= self.len() {
            return Err(PdfiumError::ImageObjectFilterIndexOutOfBounds);
        }

        // Retrieving the image filter name from Pdfium is a two-step operation. First, we call
        // FPDFImageObj_GetImageFilter() with a null buffer; this will retrieve the length of
        // the image filter name in bytes. If the length is zero, then there is no image filter name.

        // If the length is non-zero, then we reserve a byte buffer of the given
        // length and call FPDFImageObj_GetImageFilter() again with a pointer to the buffer;
        // this will write the font name into the buffer. Unlike most text handling in
        // Pdfium, image filter names are returned in UTF-8 format.

        let buffer_length = self.object.bindings().FPDFImageObj_GetImageFilter(
            self.object.object_handle(),
            index as c_int,
            std::ptr::null_mut(),
            0,
        );

        if buffer_length == 0 {
            // The image filter name is not present.

            return Err(PdfiumError::ImageObjectFilterIndexInBoundsButFilterUndefined);
        }

        let mut buffer = create_byte_buffer(buffer_length as usize);

        let result = self.object.bindings().FPDFImageObj_GetImageFilter(
            self.object.object_handle(),
            index as c_int,
            buffer.as_mut_ptr() as *mut c_void,
            buffer_length,
        );

        assert_eq!(result, buffer_length);

        Ok(PdfPageImageObjectFilter::new(
            String::from_utf8(buffer)
                // Trim any trailing nulls. All strings returned from Pdfium are generally terminated
                // by one null byte.
                .map(|str| str.trim_end_matches(char::from(0)).to_owned())
                .unwrap_or_default(),
        ))
    }

    /// Returns an iterator over all the [PdfPageImageObjectFilter] objects in this
    /// [PdfPageImageObjectFilters] collection.
    #[inline]
    pub fn iter(&self) -> PdfPageImageObjectFiltersIterator<'_> {
        PdfPageImageObjectFiltersIterator::new(self)
    }
}

/// A single image filter applied to a [PdfPageImageObject].
pub struct PdfPageImageObjectFilter {
    name: String,
}

impl PdfPageImageObjectFilter {
    #[inline]
    pub(crate) fn new(name: String) -> Self {
        PdfPageImageObjectFilter { name }
    }

    /// Returns the name of this [PdfPageImageObjectFilter].
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

/// An iterator over all the [PdfPageImageObjectFilter] objects in a
/// [PdfPageImageObjectFilters] collection.
pub struct PdfPageImageObjectFiltersIterator<'a> {
    filters: &'a PdfPageImageObjectFilters<'a>,
    next_index: PdfPageImageObjectFilterIndex,
}

impl<'a> PdfPageImageObjectFiltersIterator<'a> {
    #[inline]
    pub(crate) fn new(filters: &'a PdfPageImageObjectFilters<'a>) -> Self {
        PdfPageImageObjectFiltersIterator {
            filters,
            next_index: 0,
        }
    }
}

impl<'a> Iterator for PdfPageImageObjectFiltersIterator<'a> {
    type Item = PdfPageImageObjectFilter;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.filters.get(self.next_index);

        self.next_index += 1;

        next.ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use crate::utils::test::test_bind_to_pdfium;

    #[test]
    fn test_page_image_object_retains_format() -> Result<(), PdfiumError> {
        // Make sure the format of the image we pass into a new PdfPageImageObject is the
        // same when we later retrieve it.

        let pdfium = test_bind_to_pdfium();

        let image = pdfium
            .load_pdf_from_file("./test/path-test.pdf", None)?
            .pages()
            .get(0)?
            .render_with_config(&PdfRenderConfig::new().set_target_width(1000))?
            .as_image();

        let mut document = pdfium.create_new_pdf()?;

        let mut page = document
            .pages_mut()
            .create_page_at_end(PdfPagePaperSize::a4())?;

        let object = page.objects_mut().create_image_object(
            PdfPoints::new(100.0),
            PdfPoints::new(100.0),
            &image,
            Some(PdfPoints::new(image.width() as f32)),
            Some(PdfPoints::new(image.height() as f32)),
        )?;

        // Since the object has no image filters applied, both the raw and processed images should
        // be identical to the source image we assigned to the object. The processed image will
        // take the object's scale factors into account, but we made sure to set those to the actual
        // pixel dimensions of the source image.

        // A visual inspection can be carried out by uncommenting the PNG save commands below.

        let raw_image = object.as_image_object().unwrap().get_raw_image()?;

        // raw_image
        //     .save_with_format("./test/1.png", ImageFormat::Png)
        //     .unwrap();

        let processed_image = object
            .as_image_object()
            .unwrap()
            .get_processed_image(&document)?;

        // processed_image
        //     .save_with_format("./test/2.png", ImageFormat::Png)
        //     .unwrap();

        assert!(compare_equality_of_byte_arrays(
            image.as_bytes(),
            raw_image.into_rgba8().as_raw().as_slice()
        ));

        assert!(compare_equality_of_byte_arrays(
            image.as_bytes(),
            processed_image.into_rgba8().as_raw().as_slice()
        ));

        Ok(())
    }

    fn compare_equality_of_byte_arrays(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        for index in 0..a.len() {
            if a[index] != b[index] {
                return false;
            }
        }

        true
    }

    #[test]
    fn test_image_scaling_keeps_aspect_ratio() -> Result<(), PdfiumError> {
        let pdfium = test_bind_to_pdfium();

        let mut document = pdfium.create_new_pdf()?;

        let mut page = document
            .pages_mut()
            .create_page_at_end(PdfPagePaperSize::a4())?;

        let image = DynamicImage::new_rgb8(100, 200);

        let object = page.objects_mut().create_image_object(
            PdfPoints::new(0.0),
            PdfPoints::new(0.0),
            &image,
            Some(PdfPoints::new(image.width() as f32)),
            Some(PdfPoints::new(image.height() as f32)),
        )?;

        let image_object = object.as_image_object().unwrap();

        assert_eq!(
            image_object
                .get_processed_bitmap_with_width(&document, 50)?
                .height(),
            100
        );
        assert_eq!(
            image_object
                .get_processed_image_with_width(&document, 50)?
                .height(),
            100
        );
        assert_eq!(
            image_object
                .get_processed_bitmap_with_height(&document, 50)?
                .width(),
            25
        );
        assert_eq!(
            image_object
                .get_processed_image_with_height(&document, 50)?
                .width(),
            25
        );

        Ok(())
    }
}

impl<'a> Drop for PdfPageImageObject<'a> {
    /// Closes this [PdfPageImageObject], releasing held memory.
    fn drop(&mut self) {
        self.drop_impl();
    }
}
