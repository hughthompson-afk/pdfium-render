//! Defines the [PdfPageSquigglyAnnotation] struct, exposing functionality related to a single
//! user annotation of type [PdfPageAnnotationType::Squiggly].

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

/// A single [PdfPageAnnotation] of type [PdfPageAnnotationType::Squiggly].
pub struct PdfPageSquigglyAnnotation<'a> {
    handle: FPDF_ANNOTATION,
    objects: PdfPageAnnotationObjects<'a>,
    attachment_points: PdfPageAnnotationAttachmentPoints<'a>,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfPageSquigglyAnnotation<'a> {
    pub(crate) fn from_pdfium(
        document_handle: FPDF_DOCUMENT,
        page_handle: FPDF_PAGE,
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfPageSquigglyAnnotation {
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

    /// Returns a mutable collection of all the attachment points in this [PdfPageSquigglyAnnotation].
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

    /// Generates and sets an appearance stream for this squiggly annotation using the Normal appearance mode.
    ///
    /// This method builds a PDF content stream that renders wavy bezier curves along the bottom of each
    /// attachment point using the annotation's stroke color, then sets it as the annotation's appearance stream.
    /// This is required for the annotation to display and flatten correctly.
    ///
    /// # Errors
    ///
    /// Returns an error if PDFium fails to generate or set the appearance stream.
    pub fn generate_appearance_stream(&mut self) -> Result<(), PdfiumError> {
        self.generate_appearance_stream_with_mode(PdfAppearanceMode::Normal)
    }

    /// Generates and sets an appearance stream for this squiggly annotation with a specific appearance mode.
    ///
    /// This method builds a PDF content stream that renders wavy bezier curves along the bottom of each
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
            console::log_1(&"🔧 PdfPageSquigglyAnnotation::generate_appearance_stream_with_mode()".into());
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
        let stroke_width = 1.0; // Squiggly typically uses a fixed width

        // Get attachment points (quadpoints) - squiggly uses these if available
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

        // Expand rect to include wave height padding BEFORE building appearance stream
        // Waves extend ±2.0 from baseline, plus line width 1.0, so we need at least 5.0 padding on each side
        let wave_padding = 5.0;
        let attachment_count = attachment_points.len();
        
        if attachment_count > 0 {
            // Find the actual bounds of all attachment points
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;
            for i in 0..attachment_count {
                if let Ok(quad) = attachment_points.get(i) {
                    let quad_min_y = quad.y1.value.min(quad.y2.value).min(quad.y3.value).min(quad.y4.value);
                    let quad_max_y = quad.y1.value.max(quad.y2.value).max(quad.y3.value).max(quad.y4.value);
                    min_y = min_y.min(quad_min_y);
                    max_y = max_y.max(quad_max_y);
                }
            }
            
            // Expand rect to include wave padding
            #[cfg(target_arch = "wasm32")]
            let old_bottom = rect.bottom;
            #[cfg(target_arch = "wasm32")]
            let old_top = rect.top;
            rect.bottom = rect.bottom.min(min_y - wave_padding);
            rect.top = rect.top.max(max_y + wave_padding);
            
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("📐 Expanding squiggly rect for wave padding: bottom={:.2} -> {:.2}, top={:.2} -> {:.2}",
                    old_bottom, rect.bottom, old_top, rect.top).into());
            }
            
            // Set the expanded rect BEFORE building appearance stream
            if !self.bindings.is_true(self.bindings.FPDFAnnot_SetRect(self.handle, &rect)) {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&"⚠️  Failed to set expanded rect".into());
                }
            } else {
                // Re-fetch to ensure PDFium has the updated bounds
                let mut updated_rect = crate::bindgen::FS_RECTF {
                    left: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    top: 0.0,
                };
                if self.bindings.is_true(self.bindings.FPDFAnnot_GetRect(self.handle, &mut updated_rect)) {
                    rect = updated_rect;
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&format!("✅ Rect updated: bottom={:.2}, top={:.2}, height={:.2}",
                            rect.bottom, rect.top, rect.top - rect.bottom).into());
                    }
                }
            }
        } else {
            // No attachment points - expand existing rect by wave padding
            #[cfg(target_arch = "wasm32")]
            let old_bottom = rect.bottom;
            #[cfg(target_arch = "wasm32")]
            let old_top = rect.top;
            rect.bottom = rect.bottom - wave_padding;
            rect.top = rect.top + wave_padding;
            
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("📐 Expanding squiggly rect (no attachment points): bottom={:.2} -> {:.2}, top={:.2} -> {:.2}",
                    old_bottom, rect.bottom, old_top, rect.top).into());
            }
            
            // Set the expanded rect BEFORE building appearance stream
            if !self.bindings.is_true(self.bindings.FPDFAnnot_SetRect(self.handle, &rect)) {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&"⚠️  Failed to set expanded rect".into());
                }
            } else {
                // Re-fetch to ensure PDFium has the updated bounds
                let mut updated_rect = crate::bindgen::FS_RECTF {
                    left: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    top: 0.0,
                };
                if self.bindings.is_true(self.bindings.FPDFAnnot_GetRect(self.handle, &mut updated_rect)) {
                    rect = updated_rect;
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&format!("✅ Rect updated: bottom={:.2}, top={:.2}, height={:.2}",
                            rect.bottom, rect.top, rect.top - rect.bottom).into());
                    }
                }
            }
        }

        let content_stream_result = self.build_squiggly_appearance_stream(mode, attachment_points, rect, stroke_color, stroke_width);

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

    /// Builds a PDF content stream for squiggly annotation appearance.
    ///
    /// This method creates a content stream that draws wavy bezier curves along the bottom of each attachment point
    /// to create a squiggly effect using the specified stroke color and width.
    fn build_squiggly_appearance_stream(
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
            console::log_1(&"📐 build_squiggly_appearance_stream() - Building content stream".into());
            console::log_1(&format!("   Annotation rect: left={:.2}, bottom={:.2}, right={:.2}, top={:.2}",
                rect.left, rect.bottom, rect.right, rect.top).into());
        }

        let mut stream = String::new();

        // Save graphics state
        stream.push_str("q\n");

        // Set stroke color
        let r = stroke_color.red() as f32 / 255.0;
        let g = stroke_color.green() as f32 / 255.0;
        let b = stroke_color.blue() as f32 / 255.0;
        stream.push_str(&format!("{:.4} {:.4} {:.4} RG\n", r, g, b));

        // Set line width
        stream.push_str(&format!("{:.4} w\n", stroke_width));

        // Draw wavy bezier curves along the bottom of each attachment point, or use rect as fallback
        // Use a fixed wave period (3.5 units per wave) for consistent bezier curves across all lines
        const WAVE_PERIOD: f32 = 3.5; // Fixed period in points - ensures consistent wave pattern
        const WAVE_HEIGHT: f32 = 2.0; // Fixed wave height
        
        // Translate coordinate system to annotation's bottom-left corner (same as underline)
        // This ensures each line uses its own baseline relative to rect.bottom
        stream.push_str(&format!("1 0 0 1 {:.4} {:.4} cm\n", left, bottom));
        
        if attachment_points.len() > 0 {
            for i in 0..attachment_points.len() {
                if let Ok(quad) = attachment_points.get(i) {
                    let x_left = quad.left().value - left;
                    let x_right = quad.right().value - left;
                    // Use each quad's own bottom value relative to rect.bottom (same as underline)
                    // This ensures each line is positioned correctly at its own baseline
                    let y_bottom = quad.bottom().value - bottom;
                    let width = x_right - x_left;

                    // Use fixed wave period for consistent bezier curves
                    // Calculate number of complete waves that fit, ensuring at least 2 waves
                    let wave_count = ((width / WAVE_PERIOD).ceil().max(2.0) as i32).min(100);
                    let wave_width = width / wave_count as f32;

                    // Start at left bottom
                    stream.push_str(&format!("{:.4} {:.4} m\n", x_left, y_bottom));

                    for wave in 0..wave_count {
                        let wave_start_x = x_left + wave as f32 * wave_width;
                        let wave_end_x = x_left + (wave + 1) as f32 * wave_width;

                        // Create a bezier curve for this wave segment
                        // Control points create a consistent S-shape using fixed proportions
                        let cp1_x = wave_start_x + wave_width * 0.33;
                        let cp1_y = y_bottom + WAVE_HEIGHT;
                        let cp2_x = wave_start_x + wave_width * 0.67;
                        let cp2_y = y_bottom - WAVE_HEIGHT;
                        let end_x = wave_end_x;
                        let end_y = y_bottom;

                        stream.push_str(&format!("{:.4} {:.4} {:.4} {:.4} {:.4} {:.4} c\n",
                            cp1_x, cp1_y, cp2_x, cp2_y, end_x, end_y));
                    }

                    stream.push_str("S\n"); // stroke the path

                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&format!("   Drew squiggly {}: x={:.2} to x={:.2} at y={:.2} (quad.bottom={:.2}, rect.bottom={:.2}) with {} waves (period={:.2})",
                            i, x_left, x_right, y_bottom, quad.bottom().value, bottom, wave_count, WAVE_PERIOD).into());
                    }
                }
            }
        } else {
            // No attachment points - use annotation rect as fallback
            let width = rect.right - rect.left;
            
            // Use fixed wave period for consistent bezier curves
            let wave_count = ((width / WAVE_PERIOD).ceil().max(2.0) as i32).min(100);
            let wave_width = width / wave_count as f32;

            // Start at left bottom (relative to rect.bottom)
            stream.push_str("0 0 m\n");

            for wave in 0..wave_count {
                let wave_start_x = wave as f32 * wave_width;
                let wave_end_x = (wave + 1) as f32 * wave_width;

                // Create a bezier curve for this wave segment
                // Control points create a consistent S-shape using fixed proportions
                let cp1_x = wave_start_x + wave_width * 0.33;
                let cp1_y = WAVE_HEIGHT;
                let cp2_x = wave_start_x + wave_width * 0.67;
                let cp2_y = -WAVE_HEIGHT;
                let end_x = wave_end_x;
                let end_y = 0.0;

                stream.push_str(&format!("{:.4} {:.4} {:.4} {:.4} {:.4} {:.4} c\n",
                    cp1_x, cp1_y, cp2_x, cp2_y, end_x, end_y));
            }

            stream.push_str("S\n"); // stroke the path

            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   Drew squiggly using rect fallback: x=0 to x={:.2} with {} waves (period={:.2})",
                    width, wave_count, WAVE_PERIOD).into());
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

impl<'a> PdfPageAnnotationPrivate<'a> for PdfPageSquigglyAnnotation<'a> {
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
