//! Defines the [PdfPageWatermarkAnnotation] struct, exposing functionality related to a single
//! user annotation of type [PdfPageAnnotationType::Watermark].

use crate::bindgen::{
    FPDF_ANNOTATION, FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color, FPDF_DOCUMENT, FPDF_PAGE,
    FS_RECTF,
};
use crate::bindings::PdfiumLibraryBindings;
use crate::error::{PdfiumError, PdfiumInternalError};
use crate::pdf::appearance_mode::PdfAppearanceMode;
use crate::pdf::color::PdfColor;
use crate::pdf::document::page::annotation::attachment_points::PdfPageAnnotationAttachmentPoints;
use crate::pdf::document::page::annotation::objects::PdfPageAnnotationObjects;
use crate::pdf::document::page::annotation::private::internal::PdfPageAnnotationPrivate;
use crate::pdf::document::page::object::ownership::PdfPageObjectOwnership;
use crate::pdf::document::page::objects::private::internal::PdfPageObjectsPrivate;

#[cfg(doc)]
use crate::pdf::document::page::annotation::{PdfPageAnnotation, PdfPageAnnotationType};

/// A single [PdfPageAnnotation] of type [PdfPageAnnotationType::Watermark].
pub struct PdfPageWatermarkAnnotation<'a> {
    handle: FPDF_ANNOTATION,
    objects: PdfPageAnnotationObjects<'a>,
    attachment_points: PdfPageAnnotationAttachmentPoints<'a>,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfPageWatermarkAnnotation<'a> {
    pub(crate) fn from_pdfium(
        document_handle: FPDF_DOCUMENT,
        page_handle: FPDF_PAGE,
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfPageWatermarkAnnotation {
            handle: annotation_handle,
            objects: PdfPageAnnotationObjects::from_pdfium(
                document_handle,
                page_handle,
                annotation_handle,
                bindings,
            ),
            attachment_points: PdfPageAnnotationAttachmentPoints::from_pdfium(
                annotation_handle,
                bindings,
            ),
            bindings,
        }
    }

    /// Sets the appearance stream for this watermark annotation using the default Normal mode.
    ///
    /// This method builds a PDF content stream that renders the annotation's text contents
    /// with the specified rotation and scale, then sets it as the annotation's appearance stream.
    ///
    /// # Parameters
    /// - `rotation_degrees`: Rotation angle in degrees (0 = horizontal, 45 = diagonal, etc.)
    /// - `scale`: Scale factor (1.0 = no scaling, 2.0 = double size, etc.)
    pub fn set_appearance(
        &mut self,
        rotation_degrees: f32,
        scale: f32,
    ) -> Result<(), PdfiumError> {
        self.set_appearance_with_mode(
            PdfAppearanceMode::Normal,
            rotation_degrees,
            scale,
        )
    }

    /// Sets the appearance stream for this watermark annotation with a specific appearance mode.
    ///
    /// This method builds a PDF content stream that renders the annotation's text contents
    /// with the specified rotation and scale, then sets it as the annotation's appearance stream.
    ///
    /// # Parameters
    /// - `mode`: The appearance mode (Normal, RollOver, or Down)
    /// - `rotation_degrees`: Rotation angle in degrees (0 = horizontal, 45 = diagonal, etc.)
    /// - `scale`: Scale factor (1.0 = no scaling, 2.0 = double size, etc.)
    pub fn set_appearance_with_mode(
        &mut self,
        mode: PdfAppearanceMode,
        rotation_degrees: f32,
        scale: f32,
    ) -> Result<(), PdfiumError> {
        // CRITICAL: Ensure annotation rect is set before building appearance stream.
        // PDFium requires a valid rect with minimum size before FPDFAnnot_SetAP will work.
        let mut rect = FS_RECTF {
            left: 0.0,
            bottom: 0.0,
            right: 0.0,
            top: 0.0,
        };
        let get_rect_result = self.bindings.FPDFAnnot_GetRect(self.handle, &mut rect);

        if !self.bindings.is_true(get_rect_result) {
            // If rect is not set, set a default rect
            let default_rect = FS_RECTF {
                left: 0.0,
                bottom: 0.0,
                right: 200.0,
                top: 100.0,
            };
            let set_rect_result = self.bindings.FPDFAnnot_SetRect(self.handle, &default_rect);
            if !self.bindings.is_true(set_rect_result) {
                return Err(PdfiumError::PdfiumLibraryInternalError(
                    PdfiumInternalError::Unknown,
                ));
            }
        } else {
            // Ensure rect has valid dimensions
            let width = rect.right - rect.left;
            let height = rect.top - rect.bottom;
            if width < 1.0 || height < 1.0 {
                // Expand rect to minimum size
                rect.right = rect.left + 200.0;
                rect.top = rect.bottom + 100.0;
                let set_rect_result = self.bindings.FPDFAnnot_SetRect(self.handle, &rect);
                if !self.bindings.is_true(set_rect_result) {
                    return Err(PdfiumError::PdfiumLibraryInternalError(
                        PdfiumInternalError::Unknown,
                    ));
                }
            }
        }

        // Get fill color to check for transparency
        let mut r: u32 = 128;
        let mut g: u32 = 128;
        let mut b: u32 = 128;
        let mut a: u32 = 128;
        let fill_color = if self.bindings.is_true(self.bindings.FPDFAnnot_GetColor(
            self.handle,
            FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color,
            &mut r,
            &mut g,
            &mut b,
            &mut a,
        )) {
            PdfColor::new(r as u8, g as u8, b as u8, a as u8)
        } else {
            PdfColor::new(128, 128, 128, 128) // Semi-transparent gray
        };

        let alpha = fill_color.alpha() as f32 / 255.0;

        // CRITICAL: Set /ca and /CA BEFORE calling FPDFAnnot_SetAP_str so PDFium can create
        // the Resources dictionary when it processes the appearance stream.
        if alpha < 1.0 {
            let _ = self.bindings.FPDFAnnot_SetNumberValue(self.handle, "ca", alpha);
            let _ = self.bindings.FPDFAnnot_SetNumberValue(self.handle, "CA", alpha);
        }

        let content_stream = self.build_watermark_appearance_stream(rotation_degrees, scale)?;

        let result = self.bindings.FPDFAnnot_SetAP_str(
            self.handle,
            mode.as_pdfium(),
            &content_stream,
        );

        // Set the Appearance State (/AS) to match the mode
        let mode_str = match mode {
            PdfAppearanceMode::Normal => "/N",
            PdfAppearanceMode::RollOver => "/R",
            PdfAppearanceMode::Down => "/D",
        };

        let _as_result = self
            .bindings
            .FPDFAnnot_SetStringValue_str(self.handle, "AS", mode_str);

        if self.bindings.is_true(result) {
            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Builds the PDF content stream string for drawing the watermark text.
    fn build_watermark_appearance_stream(
        &self,
        rotation_degrees: f32,
        scale: f32,
    ) -> Result<String, PdfiumError> {
        // Get the annotation rectangle
        let mut rect = FS_RECTF {
            left: 0.0,
            bottom: 0.0,
            right: 0.0,
            top: 0.0,
        };
        if !self.bindings.is_true(self.bindings.FPDFAnnot_GetRect(self.handle, &mut rect)) {
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        let left = rect.left;
        let bottom = rect.bottom;
        let width = rect.right - rect.left;
        let height = rect.top - rect.bottom;

        // Get text contents from annotation
        use crate::bindgen::FPDF_WCHAR;
        use crate::utils::mem::create_byte_buffer;
        use crate::utils::utf16le::get_string_from_pdfium_utf16le_bytes;

        let buffer_length = self.bindings.FPDFAnnot_GetStringValue(
            self.handle,
            "Contents",
            std::ptr::null_mut(),
            0,
        );

        let contents = if buffer_length <= 2 {
            // Empty or no contents, use default
            "WATERMARK".to_string()
        } else {
            let mut buffer = create_byte_buffer(buffer_length as usize);
            let result = self.bindings.FPDFAnnot_GetStringValue(
                self.handle,
                "Contents",
                buffer.as_mut_ptr() as *mut FPDF_WCHAR,
                buffer_length,
            );

            if result == buffer_length {
                get_string_from_pdfium_utf16le_bytes(buffer).unwrap_or_else(|| "WATERMARK".to_string())
            } else {
                "WATERMARK".to_string()
            }
        };

        // Get fill color (default to semi-transparent gray if not set)
        let mut r: u32 = 128;
        let mut g: u32 = 128;
        let mut b: u32 = 128;
        let mut a: u32 = 128;
        let fill_color = if self.bindings.is_true(self.bindings.FPDFAnnot_GetColor(
            self.handle,
            FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color,
            &mut r,
            &mut g,
            &mut b,
            &mut a,
        )) {
            PdfColor::new(r as u8, g as u8, b as u8, a as u8)
        } else {
            PdfColor::new(128, 128, 128, 128) // Semi-transparent gray
        };

        // Calculate font size based on annotation size (with scale applied)
        let base_font_size = (width.min(height) * 0.15).max(12.0).min(72.0);
        let font_size = base_font_size * scale;

        // Convert rotation to radians
        let rotation_rad = rotation_degrees.to_radians();
        let cos_theta = rotation_rad.cos();
        let sin_theta = rotation_rad.sin();

        // Calculate text position (center of annotation rect)
        // We'll position text at the center, then apply rotation around that point
        let center_x = width / 2.0;
        let center_y = height / 2.0;

        // Build transformation matrix: [a b c d e f] where:
        // a = sx * cos(θ), b = sx * sin(θ)
        // c = -sy * sin(θ), d = sy * cos(θ)
        // e = tx, f = ty
        // We need to translate to center, rotate, then translate back
        // Actually, we'll translate to center first, then rotate around origin, then scale
        let a = scale * cos_theta;
        let b = scale * sin_theta;
        let c = -scale * sin_theta;
        let d = scale * cos_theta;

        // Calculate translation to center the text after rotation
        // We need to account for the text width, which we'll approximate
        let text_width_estimate = contents.len() as f32 * font_size * 0.6;
        let text_height_estimate = font_size;
        
        // After rotation, the bounding box of the text will be larger
        let rotated_width = text_width_estimate * cos_theta.abs() + text_height_estimate * sin_theta.abs();
        let rotated_height = text_width_estimate * sin_theta.abs() + text_height_estimate * cos_theta.abs();
        
        // Center the rotated text
        let tx = center_x - (rotated_width / 2.0);
        let ty = center_y - (rotated_height / 2.0);

        // Escape text for PDF string (escape parentheses and backslashes)
        let escaped_text = contents
            .replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)");

        let mut stream = String::new();

        // Save graphics state
        stream.push_str("q\n");

        // Apply ExtGState with opacity if alpha < 1.0
        // PDFium will map /GS to the /ca value we set on the annotation earlier.
        let alpha = fill_color.alpha() as f32 / 255.0;
        if alpha < 1.0 {
            stream.push_str("/GS gs\n");
        }

        // Translate to annotation's bottom-left corner
        stream.push_str(&format!("1 0 0 1 {:.4} {:.4} cm\n", left, bottom));

        // Apply rotation and scale transformation matrix
        // Format: a b c d e f cm
        stream.push_str(&format!(
            "{:.4} {:.4} {:.4} {:.4} {:.4} {:.4} cm\n",
            a, b, c, d, tx, ty
        ));

        // Set fill color (RGB with alpha)
        let r_val = fill_color.red() as f32 / 255.0;
        let g_val = fill_color.green() as f32 / 255.0;
        let b_val = fill_color.blue() as f32 / 255.0;
        let _alpha = fill_color.alpha() as f32 / 255.0;

        // Use RG for RGB color
        stream.push_str(&format!("{:.4} {:.4} {:.4} rg\n", r_val, g_val, b_val));

        // Begin text object
        stream.push_str("BT\n");

        // Set font and font size
        // Use /F1 as font resource name (standard PDF convention)
        // Note: The font resource needs to be defined in the Resources dictionary.
        // PDFium should handle this when we call FPDFAnnot_SetAP_str, mapping /F1 to a standard font.
        stream.push_str(&format!("/F1 {:.4} Tf\n", font_size));

        // Move to text position (0,0 since we already translated)
        stream.push_str("0 0 Td\n");

        // Show text
        stream.push_str(&format!("({}) Tj\n", escaped_text));

        // End text object
        stream.push_str("ET\n");

        // Restore graphics state
        stream.push_str("Q\n");

        Ok(stream)
    }
}

impl<'a> PdfPageAnnotationPrivate<'a> for PdfPageWatermarkAnnotation<'a> {
    #[inline]
    fn handle(&self) -> FPDF_ANNOTATION {
        self.handle
    }

    #[inline]
    fn ownership(&self) -> &PdfPageObjectOwnership {
        self.objects_impl().ownership()
    }

    #[inline]
    fn bindings(&self) -> &dyn PdfiumLibraryBindings {
        self.bindings
    }

    #[inline]
    fn objects_impl(&self) -> &PdfPageAnnotationObjects<'_> {
        &self.objects
    }

    #[inline]
    fn attachment_points_impl(&self) -> &PdfPageAnnotationAttachmentPoints<'_> {
        &self.attachment_points
    }
}

