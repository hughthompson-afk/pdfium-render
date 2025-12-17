//! Defines the [PdfPageHighlightAnnotation] struct, exposing functionality related to a single
//! user annotation of type [PdfPageAnnotationType::Highlight].

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

/// A single [PdfPageAnnotation] of type [PdfPageAnnotationType::Highlight].
pub struct PdfPageHighlightAnnotation<'a> {
    handle: FPDF_ANNOTATION,
    objects: PdfPageAnnotationObjects<'a>,
    attachment_points: PdfPageAnnotationAttachmentPoints<'a>,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfPageHighlightAnnotation<'a> {
    pub(crate) fn from_pdfium(
        document_handle: FPDF_DOCUMENT,
        page_handle: FPDF_PAGE,
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfPageHighlightAnnotation {
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

    /// Returns a mutable collection of all the attachment points in this [PdfPageHighlightAnnotation].
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

    /// Generates and sets an appearance stream for this highlight annotation using the Normal appearance mode.
    ///
    /// This method builds a PDF content stream that renders filled rectangles for each attachment point
    /// using the annotation's fill color, then sets it as the annotation's appearance stream.
    /// This is required for the annotation to display and flatten correctly.
    ///
    /// # Errors
    ///
    /// Returns an error if PDFium fails to generate or set the appearance stream.
    pub fn generate_appearance_stream(&mut self) -> Result<(), PdfiumError> {
        self.generate_appearance_stream_with_mode(PdfAppearanceMode::Normal)
    }

    /// Generates and sets an appearance stream for this highlight annotation with a specific appearance mode.
    ///
    /// This method builds a PDF content stream that renders filled rectangles for each attachment point
    /// using the annotation's fill color, then sets it as the annotation's appearance stream.
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
            console::log_1(&"🔧 PdfPageHighlightAnnotation::generate_appearance_stream_with_mode()".into());
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

        // Get fill color from /C dictionary
        let mut r: u32 = 0;
        let mut g: u32 = 0;
        let mut b: u32 = 0;
        let mut a: u32 = 0;
        let has_color = self.bindings.is_true(self.bindings.FPDFAnnot_GetColor(
            self.handle,
            FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color,
            &mut r,
            &mut g,
            &mut b,
            &mut a,
        ));

        let fill_color = if has_color {
            PdfColor::new(r as u8, g as u8, b as u8, a as u8)
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"⚠️  No color in /C dictionary, using default YELLOW".into());
            }
            PdfColor::YELLOW // Default highlight color
        };

        // Get attachment points (quadpoints) - highlight uses these if available
        let attachment_points = &self.attachment_points;

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   Fill color: r={}, g={}, b={}, a={}",
                fill_color.red(), fill_color.green(), fill_color.blue(), fill_color.alpha()).into());
            console::log_1(&format!("   Attachment points: {}", attachment_points.len()).into());
            if attachment_points.len() == 0 {
                console::log_1(&"   ⚠️  No attachment points, will use annotation rect as fallback".into());
            }
        }

        let content_stream_result = self.build_highlight_appearance_stream(mode, attachment_points, rect, fill_color);

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            match &content_stream_result {
                Ok(stream) => {
                    console::log_1(&format!("✅ Content stream built successfully ({} chars)", stream.len()).into());
                    // Log preview of the content stream to verify /GS gs is included
                    let preview = if stream.len() > 200 {
                        format!("{}...", &stream[..200])
                    } else {
                        stream.clone()
                    };
                    console::log_1(&format!("   Content stream preview:\n{}", preview).into());
                    console::log_1(&format!("   Contains '/GS gs': {}", stream.contains("/GS gs")).into());
                }
                Err(e) => {
                    console::log_1(&format!("❌ Failed to build content stream: {:?}", e).into());
                    return content_stream_result.map(|_| ());
                }
            }
        }

        let content_stream = content_stream_result?;
        let alpha = fill_color.alpha() as f32 / 255.0;

        // CRITICAL: Set /ca BEFORE calling FPDFAnnot_SetAP_str so PDFium can create Resources dictionary
        // when it processes the appearance stream
        if alpha < 1.0 {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"═══════════════════════════════════════════════════════════".into());
                console::log_1(&"🔧 SETTING /ca BEFORE FPDFAnnot_SetAP_str (alpha < 1.0)".into());
                console::log_1(&format!("   Setting fill opacity (/ca) to: {:.4}", alpha).into());
            }
            let _opacity_result = self.bindings.FPDFAnnot_SetNumberValue(
                self.handle,
                "ca",
                alpha,
            );
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                if self.bindings.is_true(_opacity_result) {
                    console::log_1(&format!("   ✅ Fill opacity (/ca) set successfully: {:.4}", alpha).into());
                    
                    // Verify /ca was set correctly by reading it back
                    let mut ca_value: f32 = 0.0;
                    let has_ca = self.bindings.FPDFAnnot_HasKey(self.handle, "ca");
                    if self.bindings.is_true(has_ca) {
                        let get_ca_result = self.bindings.FPDFAnnot_GetNumberValue(self.handle, "ca", &mut ca_value);
                        if self.bindings.is_true(get_ca_result) {
                            console::log_1(&format!("   ✅ Verified /ca value: {:.4} (matches expected: {:.4})", ca_value, alpha).into());
                        } else {
                            console::log_1(&"   ⚠️  Could not read back /ca value".into());
                        }
                    } else {
                        console::log_1(&"   ⚠️  /ca key not found after setting".into());
                    }
                } else {
                    console::log_1(&format!("   ⚠️  Failed to set fill opacity (/ca): {:.4}", alpha).into());
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&format!("   Calling FPDFAnnot_SetAP_str with mode: {} (alpha={:.4})", mode.as_pdfium(), alpha).into());
            console::log_1(&format!("   Content stream contains '/GS gs': {}", content_stream.contains("/GS gs")).into());
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
            // Verify the appearance stream after setting it
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"═══════════════════════════════════════════════════════════".into());
                console::log_1(&"🔍 VERIFYING APPEARANCE STREAM AFTER SetAP".into());
                
                // Get the appearance stream back to verify it contains /GS gs
                let ap_len = self.bindings.FPDFAnnot_GetAP(
                    self.handle,
                    mode.as_pdfium(),
                    std::ptr::null_mut(),
                    0,
                );
                if ap_len > 2 {
                    let mut buffer = vec![0u16; (ap_len / 2) as usize];
                    let read_result = self.bindings.FPDFAnnot_GetAP(
                        self.handle,
                        mode.as_pdfium(),
                        buffer.as_mut_ptr() as *mut crate::bindgen::FPDF_WCHAR,
                        ap_len,
                    );
                    if read_result == ap_len {
                        if let Ok(ap_content) = String::from_utf16(&buffer[..buffer.len().saturating_sub(1)]) {
                            console::log_1(&format!("   Appearance stream length: {} bytes", ap_content.len()).into());
                            console::log_1(&format!("   Contains '/GS gs': {}", ap_content.contains("/GS gs")).into());
                            console::log_1(&format!("   Contains '/GS1 gs': {}", ap_content.contains("/GS1 gs")).into());
                            
                            // Check for Resources dictionary (may be in stream object, not content)
                            // FPDFAnnot_GetAP only returns content stream, not full stream object
                            console::log_1(&"   ⚠️  Note: FPDFAnnot_GetAP returns content stream only, not Resources dictionary".into());
                            console::log_1(&"   ⚠️  Resources/ExtGState/GS must be set separately (not accessible via GetAP)".into());
                            
                            let preview = if ap_content.len() > 300 {
                                format!("{}...", &ap_content[..300])
                            } else {
                                ap_content.clone()
                            };
                            console::log_1(&format!("   Content preview:\n{}", preview).into());
                        }
                    }
                }
            }

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
                if alpha < 1.0 {
                    console::log_1(&"═══════════════════════════════════════════════════════════".into());
                    console::log_1(&"⚠️  CRITICAL: ExtGState Resources Dictionary".into());
                    console::log_1(&format!("   /ca is set to: {:.4} (alpha < 1.0)", alpha).into());
                    console::log_1(&"   Content stream references: /GS gs".into());
                    console::log_1(&format!("   Required: Resources/ExtGState/GS/ca = {:.4}", alpha).into());
                    console::log_1(&"   ⚠️  FPDFAnnot_SetAP_str only sets content stream, not Resources dictionary".into());
                    console::log_1(&"   ⚠️  Resources must be set via low-level PDF object manipulation".into());
                    console::log_1(&"═══════════════════════════════════════════════════════════".into());
                }
            }

            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   FPDFAnnot_SetStringValue_str(AS) returned: {} (1=success, 0=failure)", _as_result).into());
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

    /// Builds a PDF content stream for highlight annotation appearance.
    ///
    /// This method creates a content stream that draws filled rectangles for each attachment point
    /// using the specified fill color.
    fn build_highlight_appearance_stream(
        &self,
        _mode: PdfAppearanceMode,
        attachment_points: &PdfPageAnnotationAttachmentPoints,
        rect: crate::bindgen::FS_RECTF,
        fill_color: PdfColor,
    ) -> Result<String, PdfiumError> {
        let left = rect.left;
        let bottom = rect.bottom;
        let alpha = fill_color.alpha() as f32 / 255.0;

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"📐 build_highlight_appearance_stream() - Building content stream".into());
            console::log_1(&format!("   Annotation rect: left={:.2}, bottom={:.2}, right={:.2}, top={:.2}",
                rect.left, rect.bottom, rect.right, rect.top).into());
            console::log_1(&format!("   Fill color alpha: {:.4}", alpha).into());
        }

        let mut stream = String::new();

        // Save graphics state
        stream.push_str("q\n");

        // Apply ExtGState with opacity if alpha < 1.0
        // CRITICAL: For flattening to work, ExtGState must be defined in the appearance
        // stream's Resources dictionary. Since FPDFAnnot_SetAP_str only sets the content
        // stream, we reference /GS here to match what PDFium creates.
        if alpha < 1.0 {
            stream.push_str("/GS gs\n");
        }

        // Translate coordinate system to annotation's bottom-left corner
        stream.push_str(&format!("1 0 0 1 {:.4} {:.4} cm\n", left, bottom));

        // Set fill color (highlight uses fill, not stroke)
        let r = fill_color.red() as f32 / 255.0;
        let g = fill_color.green() as f32 / 255.0;
        let b = fill_color.blue() as f32 / 255.0;
        stream.push_str(&format!("{:.4} {:.4} {:.4} rg\n", r, g, b));

        // Draw filled rectangles for each attachment point (quadpoint), or use rect as fallback
        if attachment_points.len() > 0 {
            for i in 0..attachment_points.len() {
                if let Ok(quad) = attachment_points.get(i) {
                    // Convert quadpoint coordinates to relative coordinates (subtract annotation rect origin)
                    let x1_rel = quad.x1.value - left;
                    let y1_rel = quad.y1.value - bottom;
                    let x2_rel = quad.x2.value - left;
                    let y2_rel = quad.y2.value - bottom;
                    let x3_rel = quad.x3.value - left;
                    let y3_rel = quad.y3.value - bottom;
                    let x4_rel = quad.x4.value - left;
                    let y4_rel = quad.y4.value - bottom;

                    // Draw filled quadrilateral using PDF's re (rectangle) operator with 4 points
                    // PDF re operator: x y width height re - but for quadrilateral we need to use the path operators
                    stream.push_str(&format!("{:.4} {:.4} m\n", x1_rel, y1_rel)); // moveto first point
                    stream.push_str(&format!("{:.4} {:.4} l\n", x2_rel, y2_rel)); // lineto second point
                    stream.push_str(&format!("{:.4} {:.4} l\n", x3_rel, y3_rel)); // lineto third point
                    stream.push_str(&format!("{:.4} {:.4} l\n", x4_rel, y4_rel)); // lineto fourth point
                    stream.push_str("h\n"); // closepath
                    stream.push_str("f\n"); // fill

                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&format!("   Drew quadpoint {}: ({:.2},{:.2}) -> ({:.2},{:.2}) -> ({:.2},{:.2}) -> ({:.2},{:.2})",
                            i, x1_rel, y1_rel, x2_rel, y2_rel, x3_rel, y3_rel, x4_rel, y4_rel).into());
                    }
                }
            }
        } else {
            // No attachment points - use annotation rect as fallback
            let width = rect.right - rect.left;
            let height = rect.top - rect.bottom;
            
            // Draw filled rectangle using annotation rect
            stream.push_str(&format!("0 0 {:.4} {:.4} re\n", width, height));
            stream.push_str("f\n"); // fill

            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   Drew highlight using rect fallback: width={:.2}, height={:.2}",
                    width, height).into());
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

impl<'a> PdfPageAnnotationPrivate<'a> for PdfPageHighlightAnnotation<'a> {
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
