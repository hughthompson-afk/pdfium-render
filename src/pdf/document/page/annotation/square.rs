//! Defines the [PdfPageSquareAnnotation] struct, exposing functionality related to a single
//! user annotation of type [PdfPageAnnotationType::Square].

use crate::bindgen::{
    FPDF_ANNOTATION, FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color,
    FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_InteriorColor, FPDF_DOCUMENT, FPDF_PAGE,
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
use crate::pdf::rect::PdfRect;

#[cfg(doc)]
use crate::pdf::document::page::annotation::{PdfPageAnnotation, PdfPageAnnotationType};

/// A single [PdfPageAnnotation] of type [PdfPageAnnotationType::Square].
pub struct PdfPageSquareAnnotation<'a> {
    handle: FPDF_ANNOTATION,
    objects: PdfPageAnnotationObjects<'a>,
    attachment_points: PdfPageAnnotationAttachmentPoints<'a>,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfPageSquareAnnotation<'a> {
    pub(crate) fn from_pdfium(
        document_handle: FPDF_DOCUMENT,
        page_handle: FPDF_PAGE,
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfPageSquareAnnotation {
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

    /// Returns the stroke width of this square annotation.
    ///
    /// Returns the width from the `/BS/W` dictionary entry, or `1.0` if not set (per PDF specification default).
    #[cfg(feature = "pdfium_future")]
    pub fn stroke_width(&self) -> Result<f32, PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"🔍 PdfPageSquareAnnotation::stroke_width() - Starting".into());
        }

        let mut width: f32 = 1.0;
        let result = self.bindings.FPDFAnnot_GetBSWidth(self.handle, &mut width);
        
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   FPDFAnnot_GetBSWidth returned: {} (1=success, 0=failure)", result).into());
            console::log_1(&format!("   Retrieved width: {:.4}", width).into());
        }

        if self.bindings.is_true(result) {
            Ok(width)
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"⚠️  /BS/W doesn't exist, returning default 1.0".into());
            }
            Ok(1.0)
        }
    }

    /// Returns the stroke width of this square annotation.
    ///
    /// Returns the default value of `1.0` when the `pdfium_future` feature is not enabled.
    #[cfg(not(feature = "pdfium_future"))]
    pub fn stroke_width(&self) -> Result<f32, PdfiumError> {
        Ok(1.0)
    }

    /// Sets the stroke width of this square annotation.
    ///
    /// The width is stored in the `/BS/W` dictionary entry per PDF specification.
    /// If the rectangle is already set, the appearance stream will be rebuilt
    /// with the new stroke width.
    ///
    /// # Arguments
    ///
    /// * `width` - The stroke width in points. Must be >= 0.0.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The width is negative
    /// - The `pdfium_future` feature is not enabled
    /// - PDFium fails to set the stroke width
    /// - Rebuilding the appearance stream fails (if rectangle is already set)
    #[cfg(feature = "pdfium_future")]
    pub fn set_stroke_width(&mut self, width: f32) -> Result<(), PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("🔧 PdfPageSquareAnnotation::set_stroke_width() - width: {:.4}", width).into());
        }

        // Validate width
        if width < 0.0 {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"❌ ERROR: Width is negative".into());
            }
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        let set_result = self.bindings.FPDFAnnot_SetBSWidth(self.handle, width);
        
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   FPDFAnnot_SetBSWidth returned: {} (1=success, 0=failure)", set_result).into());
        }

        if !self.bindings.is_true(set_result) {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"❌ ERROR: FPDFAnnot_SetBSWidth failed".into());
            }
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        // If rectangle is already set, rebuild appearance stream with new width
        let mut rect = crate::bindgen::FS_RECTF {
            left: 0.0,
            bottom: 0.0,
            right: 0.0,
            top: 0.0,
        };
        let get_rect_result = self.bindings.FPDFAnnot_GetRect(self.handle, &mut rect);
        
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   FPDFAnnot_GetRect returned: {} (1=success, 0=failure)", get_rect_result).into());
        }

        if self.bindings.is_true(get_rect_result) {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"   Rectangle is set, rebuilding appearance stream".into());
            }
            let pdf_rect = PdfRect::from_pdfium(rect);
            self.set_rect(pdf_rect)?;
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"   Rectangle not set yet, width will be used when rect is set".into());
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"✅ set_stroke_width() completed".into());
        }

        Ok(())
    }

    /// Sets the stroke width of this square annotation.
    ///
    /// Returns an error when the `pdfium_future` feature is not enabled.
    #[cfg(not(feature = "pdfium_future"))]
    pub fn set_stroke_width(&mut self, _width: f32) -> Result<(), PdfiumError> {
        Err(PdfiumError::PdfiumLibraryInternalError(
            PdfiumInternalError::Unknown,
        ))
    }

    /// Sets the rectangle of this square annotation using an appearance stream.
    ///
    /// This method builds a PDF content stream that draws the square and sets it as the
    /// annotation's appearance stream. The square will be drawn using the annotation's
    /// current stroke color, fill color (if set), and stroke width settings.
    pub fn set_rect(&mut self, rect: PdfRect) -> Result<(), PdfiumError> {
        self.set_rect_with_mode(rect, PdfAppearanceMode::Normal)
    }

    /// Sets the rectangle of this square annotation with a specific appearance mode.
    pub fn set_rect_with_mode(
        &mut self,
        rect: PdfRect,
        mode: PdfAppearanceMode,
    ) -> Result<(), PdfiumError> {
        // Get stroke width to expand bounding box to accommodate border
        let stroke_width = self.stroke_width().unwrap_or(1.0);
        let half_stroke = stroke_width / 2.0;
        
        // Expand the rect by half the stroke width on all sides to prevent clipping
        let expanded_rect = PdfRect::new_from_values(
            rect.bottom().value - half_stroke,
            rect.left().value - half_stroke,
            rect.top().value + half_stroke,
            rect.right().value + half_stroke,
        );
        
        // Convert PdfRect to FS_RECTF
        let fs_rect = expanded_rect.as_pdfium();

        // Set the annotation rectangle
        if !self.bindings.is_true(
            self.bindings.FPDFAnnot_SetRect(self.handle, &fs_rect)
        ) {
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        // Build appearance stream
        let content_stream = self.build_square_appearance_stream()?;

        // Set the appearance stream
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

    /// Builds the PDF content stream string for drawing the square.
    fn build_square_appearance_stream(&self) -> Result<String, PdfiumError> {
        // Get the annotation rectangle to translate coordinates
        let mut rect = crate::bindgen::FS_RECTF {
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
        
        // Get stroke width to account for expanded bounding box
        let stroke_width = self.stroke_width().unwrap_or(1.0);
        let half_stroke = stroke_width / 2.0;
        
        // The rect has been expanded by half_stroke on all sides, so we need to
        // adjust the drawing area to account for this
        let width = (rect.right - rect.left) - (2.0 * half_stroke);
        let height = (rect.top - rect.bottom) - (2.0 * half_stroke);
        
        // Offset the drawing position to account for the expansion
        let draw_left = left + half_stroke;
        let draw_bottom = bottom + half_stroke;

        // Get stroke color (default to black if not set)
        let mut r: u32 = 0;
        let mut g: u32 = 0;
        let mut b: u32 = 0;
        let mut a: u32 = 0;
        let stroke_color = if self.bindings.is_true(self.bindings.FPDFAnnot_GetColor(
            self.handle,
            FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color,
            &mut r,
            &mut g,
            &mut b,
            &mut a,
        )) {
            PdfColor::new(r as u8, g as u8, b as u8, a as u8)
        } else {
            PdfColor::BLACK
        };

        // Get fill color
        let mut fr: u32 = 0;
        let mut fg: u32 = 0;
        let mut fb: u32 = 0;
        let mut fa: u32 = 0;
        let fill_color = if self.bindings.is_true(self.bindings.FPDFAnnot_GetColor(
            self.handle,
            FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_InteriorColor,
            &mut fr,
            &mut fg,
            &mut fb,
            &mut fa,
        )) {
            // Check if the fill color is actually transparent (alpha = 0)
            // OR if it's a default black/gray color that PDFium sets by default
            if fa == 0 {
                None
            } else if fr == 0 && fg == 0 && fb == 0 && fa == 255 {
                // PDFium default black fill - treat as transparent
                None
            } else if fr == fg && fg == fb && (fa == 128 || fa == 191 || fa == 255) {
                // PDFium default gray fill - treat as transparent
                None
            } else {
                Some(PdfColor::new(fr as u8, fg as u8, fb as u8, fa as u8))
            }
        } else {
            None
        };

        // Get line width from /BS/W dictionary or default to 1.0
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"📏 build_square_appearance_stream() - Getting stroke width".into());
        }
        let line_width = match self.stroke_width() {
            Ok(w) => {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&format!("   Retrieved stroke width: {:.4}", w).into());
                }
                w
            }
            Err(_e) => {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&format!("   ⚠️  Error: {:?}, using default 1.0", _e).into());
                }
                1.0
            }
        };

        let mut stream = String::new();

        // Save graphics state
        stream.push_str("q\n");

        // Translate coordinate system to the drawing position (accounting for border expansion)
        stream.push_str(&format!("1 0 0 1 {:.4} {:.4} cm\n", draw_left, draw_bottom));

        // Set line cap style (round caps)
        stream.push_str("1 J\n");
        // Set line join style (round joins)
        stream.push_str("1 j\n");

        // Set stroke color (RGB)
        let r = stroke_color.red() as f32 / 255.0;
        let g = stroke_color.green() as f32 / 255.0;
        let b = stroke_color.blue() as f32 / 255.0;
        stream.push_str(&format!("{:.4} {:.4} {:.4} RG\n", r, g, b));

        // Set fill color if available and not transparent
        if let Some(fill) = fill_color {
            // Only set fill color if alpha > 0 (not transparent)
            if fill.alpha() > 0 {
                let fr = fill.red() as f32 / 255.0;
                let fg = fill.green() as f32 / 255.0;
                let fb = fill.blue() as f32 / 255.0;
                stream.push_str(&format!("{:.4} {:.4} {:.4} rg\n", fr, fg, fb));
            }
        }

        // Set line width
        stream.push_str(&format!("{:.4} w\n", line_width));

        // Draw rectangle path (relative to translated origin)
        // PDF re operator: x y width height re
        stream.push_str(&format!("0 0 {:.4} {:.4} re\n", width, height));

        // Fill and/or stroke based on fill color presence and transparency
        // Default behavior: transparent fill (stroke only) when no fill color is set or alpha is 0
        if fill_color.is_some() && fill_color.as_ref().map(|c| c.alpha() > 0).unwrap_or(false) {
            stream.push_str("B\n"); // Fill and stroke
        } else {
            stream.push_str("S\n"); // Stroke only (transparent fill)
        }

        // Restore graphics state
        stream.push_str("Q\n");

        Ok(stream)
    }
}

impl<'a> PdfPageAnnotationPrivate<'a> for PdfPageSquareAnnotation<'a> {
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
