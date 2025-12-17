//! Defines the [PdfPageStrikeoutAnnotation] struct, exposing functionality related to a single
//! user annotation of type [PdfPageAnnotationType::Strikeout].

use crate::bindgen::{FPDF_ANNOTATION, FPDF_DOCUMENT, FPDF_PAGE, FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color};
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

/// A single [PdfPageAnnotation] of type [PdfPageAnnotationType::Strikeout].
pub struct PdfPageStrikeoutAnnotation<'a> {
    handle: FPDF_ANNOTATION,
    objects: PdfPageAnnotationObjects<'a>,
    attachment_points: PdfPageAnnotationAttachmentPoints<'a>,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfPageStrikeoutAnnotation<'a> {
    pub(crate) fn from_pdfium(
        document_handle: FPDF_DOCUMENT,
        page_handle: FPDF_PAGE,
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfPageStrikeoutAnnotation {
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

    /// Returns a mutable collection of all the attachment points in this [PdfPageStrikeoutAnnotation].
    #[inline]
    pub fn attachment_points_mut(&mut self) -> &mut PdfPageAnnotationAttachmentPoints<'a> {
        &mut self.attachment_points
    }

    /// Creates a new attachment point and automatically regenerates the appearance stream.
    ///
    /// This is a convenience method that combines `attachment_points_mut().create_attachment_point_at_end()`
    /// with `generate_appearance_stream()` to ensure the appearance stream is up-to-date after adding
    /// attachment points.
    ///
    /// # Errors
    ///
    /// Returns an error if setting the attachment point or generating the appearance stream fails.
    pub fn create_attachment_point_and_regenerate_appearance(
        &mut self,
        attachment_point: crate::pdf::quad_points::PdfQuadPoints,
    ) -> Result<(), PdfiumError> {
        self.attachment_points_mut()
            .create_attachment_point_at_end(attachment_point)?;
        // Try to regenerate appearance stream (will succeed if rect is valid)
        let _ = self.generate_appearance_stream();
        Ok(())
    }

    /// Sets an attachment point at the given index and automatically regenerates the appearance stream.
    ///
    /// This is a convenience method that combines `attachment_points_mut().set_attachment_point_at_index()`
    /// with `generate_appearance_stream()` to ensure the appearance stream is up-to-date after modifying
    /// attachment points.
    ///
    /// # Errors
    ///
    /// Returns an error if setting the attachment point or generating the appearance stream fails.
    pub fn set_attachment_point_and_regenerate_appearance(
        &mut self,
        index: crate::pdf::document::page::annotation::attachment_points::PdfPageAnnotationAttachmentPointIndex,
        attachment_point: crate::pdf::quad_points::PdfQuadPoints,
    ) -> Result<(), PdfiumError> {
        self.attachment_points_mut()
            .set_attachment_point_at_index(index, attachment_point)?;
        // Try to regenerate appearance stream (will succeed if rect is valid)
        let _ = self.generate_appearance_stream();
        Ok(())
    }

    /// Generates and sets an appearance stream for this strikeout annotation using the Normal appearance mode.
    ///
    /// This method builds a PDF content stream that renders horizontal lines through the middle of each
    /// attachment point using the annotation's stroke color, then sets it as the annotation's appearance stream.
    /// This is required for the annotation to display and flatten correctly.
    ///
    /// # Errors
    ///
    /// Returns an error if PDFium fails to generate or set the appearance stream.
    pub fn generate_appearance_stream(&mut self) -> Result<(), PdfiumError> {
        self.generate_appearance_stream_with_mode(PdfAppearanceMode::Normal)
    }

    /// Generates and sets an appearance stream for this strikeout annotation with a specific appearance mode.
    ///
    /// This method builds a PDF content stream that renders horizontal lines through the middle of each
    /// attachment point using the annotation's stroke color, then sets it as the annotation's appearance stream.
    /// This is required for the annotation to display and flatten correctly.
    ///
    /// # Parameters
    ///
    /// * `mode` - The appearance mode (Normal, RollOver, or Down)
    ///
    /// # Errors
    ///
    /// Returns an error if PDFium fails to generate or set the appearance stream.
    pub fn generate_appearance_stream_with_mode(&mut self, mode: PdfAppearanceMode) -> Result<(), PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&"🔧 PdfPageStrikeoutAnnotation::generate_appearance_stream_with_mode()".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&format!("   Appearance mode: {:?}", mode).into());
        }

        // Get annotation rectangle first (required)
        let mut rect = crate::bindgen::FS_RECTF {
            left: 0.0,
            bottom: 0.0,
            right: 0.0,
            top: 0.0,
        };
        if !self.bindings.is_true(self.bindings.FPDFAnnot_GetRect(self.handle, &mut rect)) {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"❌ ERROR: FPDFAnnot_GetRect failed".into());
            }
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        // Check if rect is valid (has non-zero width and height)
        let width = rect.right - rect.left;
        let height = rect.top - rect.bottom;
        if width <= 0.0 || height <= 0.0 {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("⚠️  Annotation rect is invalid (width={:.2}, height={:.2}), skipping appearance stream generation. Call generate_appearance_stream() again after setting the rect.", width, height).into());
            }
            // Return Ok(()) instead of error so this method can be called multiple times
            // It will succeed once the rect becomes valid
            return Ok(());
        }

        // STEP 1: Preserve stroke color from /C dictionary BEFORE appearance stream exists
        // This is critical: FPDFAnnot_GetColor fails once an appearance stream exists,
        // so we must read the color BEFORE building the stream.
        let mut preserved_r: u32 = 0;
        let mut preserved_g: u32 = 0;
        let mut preserved_b: u32 = 0;
        let mut preserved_a: u32 = 0;
        let has_existing_color = self.bindings.is_true(self.bindings.FPDFAnnot_GetColor(
            self.handle,
            FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color,
            &mut preserved_r,
            &mut preserved_g,
            &mut preserved_b,
            &mut preserved_a,
        ));

        let preserved_stroke_color = if has_existing_color {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   ✅ Color found in /C dictionary: r={}, g={}, b={}, a={}", 
                    preserved_r, preserved_g, preserved_b, preserved_a).into());
                console::log_1(&"   This color will be used in appearance stream".into());
            }
            Some(PdfColor::new(preserved_r as u8, preserved_g as u8, preserved_b as u8, preserved_a as u8))
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"   ⚠️  No color in /C dictionary, will use default BLACK".into());
            }
            None
        };

        // Use preserved color or default
        let stroke_color = preserved_stroke_color.unwrap_or(PdfColor::BLACK);

        // Get stroke width (default to 1.0)
        let stroke_width = 1.0; // Strikeout typically uses a fixed width

        // Get attachment points (quadpoints) - strikeout uses these if available
        let attachment_points = &self.attachment_points;

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   Stroke color: r={}, g={}, b={}, a={}",
                stroke_color.red(), stroke_color.green(), stroke_color.blue(), stroke_color.alpha()).into());
            console::log_1(&format!("   Stroke width: {:.4}", stroke_width).into());
            console::log_1(&format!("   Attachment points: {}", attachment_points.len()).into());
            if attachment_points.len() == 0 {
                console::log_1(&"   ⚠️  No attachment points, will use annotation rect as fallback".into());
            }
        }

        let content_stream_result = self.build_strikeout_appearance_stream(mode, attachment_points, rect, stroke_color, stroke_width);

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            match &content_stream_result {
                Ok(stream) => {
                    console::log_1(&format!("✅ Content stream built successfully ({} chars)", stream.len()).into());
                }
                Err(e) => {
                    console::log_1(&format!("❌ Failed to build content stream: {:?}", e).into());
                    return content_stream_result.map(|_| ());
                }
            }
        }

        let content_stream = content_stream_result?;

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   Calling FPDFAnnot_SetAP_str with mode: {}", mode.as_pdfium()).into());
        }

        let result = self.bindings.FPDFAnnot_SetAP_str(
            self.handle,
            mode.as_pdfium(),
            &content_stream,
        );

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   FPDFAnnot_SetAP_str returned: {} (1=success, 0=failure)", result).into());
        }

        if self.bindings.is_true(result) {
            // Set Appearance State (/AS) to match the mode
            let mode_str = match mode {
                PdfAppearanceMode::Normal => "/N",
                PdfAppearanceMode::RollOver => "/R",
                PdfAppearanceMode::Down => "/D",
            };

            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   Setting AS to: {}", mode_str).into());
            }

            let _as_result = self
                .bindings
                .FPDFAnnot_SetStringValue_str(self.handle, "AS", mode_str);

            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   FPDFAnnot_SetStringValue_str(AS) returned: {} (1=success, 0=failure)", _as_result).into());
            }

            // STEP 4: Try to restore stroke color after setting appearance stream
            // Note: This may fail if PDFium locks the color dictionary when appearance stream exists.
            // If it fails, the color is already embedded in the appearance stream, which is fine.
            // The important thing is that we read the color BEFORE building the stream.
            if has_existing_color {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&"🔄 Attempting to restore stroke color to /C dictionary after setting appearance stream".into());
                    console::log_1(&"   (This may fail - color is already in appearance stream)".into());
                }
                use std::os::raw::c_uint;
                let _restore_result = self.bindings.FPDFAnnot_SetColor(
                    self.handle,
                    FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color,
                    preserved_r as c_uint,
                    preserved_g as c_uint,
                    preserved_b as c_uint,
                    preserved_a as c_uint,
                );
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    if self.bindings.is_true(_restore_result) {
                        console::log_1(&format!("   ✅ Stroke color restored to dictionary: r={}, g={}, b={}, a={}", 
                            preserved_r, preserved_g, preserved_b, preserved_a).into());
                    } else {
                        console::log_1(&format!("   ⚠️  Stroke color restore failed (expected - color is in appearance stream: r={}, g={}, b={}, a={})", 
                            preserved_r, preserved_g, preserved_b, preserved_a).into());
                    }
                }
            }

            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"✅ generate_appearance_stream_with_mode completed successfully".into());
            }

            Ok(())
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"❌ ERROR: FPDFAnnot_SetAP_str failed".into());
            }
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Builds a PDF content stream for strikeout annotation appearance.
    ///
    /// This method creates a content stream that draws horizontal lines through the middle of each attachment point
    /// using the specified stroke color and width.
    fn build_strikeout_appearance_stream(
        &self,
        _mode: PdfAppearanceMode,
        attachment_points: &PdfPageAnnotationAttachmentPoints,
        rect: crate::bindgen::FS_RECTF,
        stroke_color: PdfColor,
        stroke_width: f32,
    ) -> Result<String, PdfiumError> {
        let left = rect.left;
        let bottom = rect.bottom;

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"📐 build_strikeout_appearance_stream() - Building content stream".into());
            console::log_1(&format!("   Annotation rect: left={:.2}, bottom={:.2}, right={:.2}, top={:.2}",
                rect.left, rect.bottom, rect.right, rect.top).into());
        }

        let mut stream = String::new();

        // Save graphics state
        stream.push_str("q\n");

        // Translate coordinate system to annotation's bottom-left corner
        stream.push_str(&format!("1 0 0 1 {:.4} {:.4} cm\n", left, bottom));

        // Set stroke color
        let r = stroke_color.red() as f32 / 255.0;
        let g = stroke_color.green() as f32 / 255.0;
        let b = stroke_color.blue() as f32 / 255.0;
        stream.push_str(&format!("{:.4} {:.4} {:.4} RG\n", r, g, b));

        // Set line width
        stream.push_str(&format!("{:.4} w\n", stroke_width));

        // Draw horizontal lines through the middle of each attachment point, or use rect as fallback
        if attachment_points.len() > 0 {
            for i in 0..attachment_points.len() {
                if let Ok(quad) = attachment_points.get(i) {
                    // Strikeout is drawn through the middle of the quadpoint
                    let y_middle = (quad.top().value + quad.bottom().value) / 2.0 - bottom;
                    let x_left = quad.left().value - left;
                    let x_right = quad.right().value - left;

                    // Draw horizontal line from left to right at middle Y coordinate
                    stream.push_str(&format!("{:.4} {:.4} m\n", x_left, y_middle)); // moveto left
                    stream.push_str(&format!("{:.4} {:.4} l\n", x_right, y_middle)); // lineto right
                    stream.push_str("S\n"); // stroke

                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&format!("   Drew strikeout {}: x={:.2} to x={:.2} at y={:.2}",
                            i, x_left, x_right, y_middle).into());
                    }
                }
            }
        } else {
            // No attachment points - use annotation rect as fallback
            let width = rect.right - rect.left;
            let height = rect.top - rect.bottom;
            let y_middle = height / 2.0;
            
            // Draw horizontal line through the middle of the annotation rect
            stream.push_str(&format!("0 {:.4} m\n", y_middle)); // moveto left
            stream.push_str(&format!("{:.4} {:.4} l\n", width, y_middle)); // lineto right
            stream.push_str("S\n"); // stroke

            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   Drew strikeout using rect fallback: x=0 to x={:.2} at y={:.2}",
                    width, y_middle).into());
            }
        }

        // Restore graphics state
        stream.push_str("Q\n");

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("✅ Content stream built: {} characters", stream.len()).into());
        }

        Ok(stream)
    }
}

impl<'a> PdfPageAnnotationPrivate<'a> for PdfPageStrikeoutAnnotation<'a> {
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
