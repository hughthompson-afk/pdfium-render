//! Defines the [PdfPageAnnotationAttachmentPoints] struct, a collection of all the
//! attachment points that visually associate a `PdfPageAnnotation` object with one or more
//! `PdfPageObject` objects on a `PdfPage`.

use crate::bindgen::{FPDF_ANNOTATION, FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color, FS_RECTF};
use crate::bindings::PdfiumLibraryBindings;
use crate::error::{PdfiumError, PdfiumInternalError};
use crate::pdf::color::PdfColor;
use crate::pdf::document::page::annotation::PdfPageAnnotationType;
use crate::pdf::quad_points::PdfQuadPoints;
use std::ops::{Range, RangeInclusive};

/// The zero-based index of a single attachment point inside its containing
/// [PdfPageAnnotationAttachmentPoints] collection.
pub type PdfPageAnnotationAttachmentPointIndex = usize;

/// A set of all the attachment points that visually connect a `PdfPageAnnotation` object
/// to one or more `PdfPageObject` objects on a `PdfPage`.
pub struct PdfPageAnnotationAttachmentPoints<'a> {
    annotation_handle: FPDF_ANNOTATION,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfPageAnnotationAttachmentPoints<'a> {
    #[inline]
    pub(crate) fn from_pdfium(
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfPageAnnotationAttachmentPoints {
            annotation_handle,
            bindings,
        }
    }

    /// Returns the number of attachment points in this [PdfPageAnnotationAttachmentPoints] collection.
    pub fn len(&self) -> PdfPageAnnotationAttachmentPointIndex {
        if self.bindings.is_true(
            self.bindings
                .FPDFAnnot_HasAttachmentPoints(self.annotation_handle),
        ) {
            self.bindings
                .FPDFAnnot_CountAttachmentPoints(self.annotation_handle)
                as PdfPageAnnotationAttachmentPointIndex
        } else {
            // Attachment points are not supported for this annotation type.

            0
        }
    }

    /// Returns `true` if this [PdfPageAnnotationAttachmentPoints] collection is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a Range from `0..(number of attachment points)` for this
    /// [PdfPageAnnotationAttachmentPoints] collection.
    #[inline]
    pub fn as_range(&self) -> Range<PdfPageAnnotationAttachmentPointIndex> {
        0..self.len()
    }

    /// Returns an inclusive Range from `0..=(number of attachment points - 1)` for this
    /// [PdfPageAnnotationAttachmentPoints] collection.
    #[inline]
    pub fn as_range_inclusive(&self) -> RangeInclusive<PdfPageAnnotationAttachmentPointIndex> {
        if self.is_empty() {
            0..=0
        } else {
            0..=(self.len() - 1)
        }
    }

    /// Returns a single attachment point, expressed as a set of [PdfQuadPoints], from this
    /// [PdfPageAnnotationAttachmentPoints] collection.
    pub fn get(
        &self,
        index: PdfPageAnnotationAttachmentPointIndex,
    ) -> Result<PdfQuadPoints, PdfiumError> {
        if index >= self.len() {
            return Err(PdfiumError::PageAnnotationAttachmentPointIndexOutOfBounds);
        }

        let mut result = PdfQuadPoints::ZERO.as_pdfium();

        if self
            .bindings
            .is_true(self.bindings.FPDFAnnot_GetAttachmentPoints(
                self.annotation_handle,
                index,
                &mut result,
            ))
        {
            Ok(PdfQuadPoints::from_pdfium(result))
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Returns the first attachment point, expressed as a set of [PdfQuadPoints],
    /// in this [PdfPageAnnotationAttachmentPoints] collection.
    #[inline]
    pub fn first(&self) -> Result<PdfQuadPoints, PdfiumError> {
        if !self.is_empty() {
            self.get(0)
        } else {
            Err(PdfiumError::NoAttachmentPointsInPageAnnotation)
        }
    }

    /// Returns the last attachment point, expressed as a set of [PdfQuadPoints],
    /// in this [PdfPageAnnotationAttachmentPoints] collection.
    #[inline]
    pub fn last(&self) -> Result<PdfQuadPoints, PdfiumError> {
        if !self.is_empty() {
            self.get(self.len() - 1)
        } else {
            Err(PdfiumError::NoAttachmentPointsInPageAnnotation)
        }
    }

    /// Creates a new attachment point from the given set of [PdfQuadPoints],
    /// and appends it to the end of this [PdfPageAnnotationAttachmentPoints] collection.
    ///
    /// For text markup annotations (highlight, underline, strikeout, squiggly), this method
    /// will automatically attempt to regenerate the appearance stream after setting the attachment point,
    /// since setting attachment points typically makes the annotation rect valid.
    pub fn create_attachment_point_at_end(
        &mut self,
        attachment_point: PdfQuadPoints,
    ) -> Result<(), PdfiumError> {
        if self
            .bindings
            .is_true(self.bindings.FPDFAnnot_AppendAttachmentPoints(
                self.annotation_handle,
                &attachment_point.as_pdfium(),
            ))
        {
            // Try to automatically regenerate appearance stream for text markup annotations
            let _ = self.try_regenerate_appearance_stream_for_text_markup();
            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Replaces the attachment at the given index in this [PdfPageAnnotationAttachmentPoints]
    /// collection with the given updated set of [PdfQuadPoints].
    ///
    /// For text markup annotations (highlight, underline, strikeout, squiggly), this method
    /// will automatically attempt to regenerate the appearance stream after setting the attachment point,
    /// since setting attachment points typically makes the annotation rect valid.
    pub fn set_attachment_point_at_index(
        &mut self,
        index: PdfPageAnnotationAttachmentPointIndex,
        attachment_point: PdfQuadPoints,
    ) -> Result<(), PdfiumError> {
        if self
            .bindings
            .is_true(self.bindings.FPDFAnnot_SetAttachmentPoints(
                self.annotation_handle,
                index,
                &attachment_point.as_pdfium(),
            ))
        {
            // Try to automatically regenerate appearance stream for text markup annotations
            let _ = self.try_regenerate_appearance_stream_for_text_markup();
            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Attempts to regenerate the appearance stream for text markup annotations.
    ///
    /// This is a helper function that checks if the annotation is a text markup type
    /// (highlight, underline, strikeout, squiggly) and if the rect is valid, then
    /// attempts to regenerate the appearance stream. This is called automatically after
    /// setting attachment points.
    ///
    /// Note: This generates a basic appearance stream. For more control, call the annotation
    /// type's `generate_appearance_stream()` method directly.
    ///
    /// Returns Ok(()) if successful or if the annotation is not a text markup type.
    /// Extracts stroke color from existing appearance stream by parsing the RG command.
    /// Returns None if no appearance stream exists or color cannot be extracted.
    fn extract_color_from_appearance_stream(
        annotation_handle: FPDF_ANNOTATION,
        bindings: &dyn PdfiumLibraryBindings,
    ) -> Option<PdfColor> {
        // Get the appearance stream content (Normal mode = 0)
        let buffer_length = bindings.FPDFAnnot_GetAP(
            annotation_handle,
            0, // Normal mode
            std::ptr::null_mut(),
            0,
        );

        if buffer_length == 0 {
            return None;
        }

        let mut buffer = vec![0u16; (buffer_length / 2 + 1) as usize];
        let result = bindings.FPDFAnnot_GetAP(
            annotation_handle,
            0, // Normal mode
            buffer.as_mut_ptr() as *mut crate::bindgen::FPDF_WCHAR,
            buffer_length,
        );

        if result == 0 {
            return None;
        }

        // Convert UTF-16LE to String
        let stream_content = String::from_utf16_lossy(&buffer[..((result / 2) as usize).saturating_sub(1)]);
        
        use std::str::FromStr;
        
        // Try to extract stroke color from RG command: "r g b RG" (for underline, strikeout, squiggly)
        if let Some(rg_pos) = stream_content.find(" RG") {
            // Find the three numbers before "RG"
            let before_rg = &stream_content[..rg_pos];
            let parts: Vec<&str> = before_rg.split_whitespace().collect();
            
            // We need the last 3 numbers before "RG"
            if parts.len() >= 3 {
                // Try to parse the last 3 parts as numbers
                let r_str = parts[parts.len().saturating_sub(3)];
                let g_str = parts[parts.len().saturating_sub(2)];
                let b_str = parts[parts.len().saturating_sub(1)];
                
                if let (Ok(r_val), Ok(g_val), Ok(b_val)) = (
                    f64::from_str(r_str),
                    f64::from_str(g_str),
                    f64::from_str(b_str),
                ) {
                    // PDF colors are 0.0-1.0, convert to 0-255
                    let r = (r_val * 255.0).clamp(0.0, 255.0) as u8;
                    let g = (g_val * 255.0).clamp(0.0, 255.0) as u8;
                    let b = (b_val * 255.0).clamp(0.0, 255.0) as u8;
                    let a = 255u8; // Alpha is not in RG command, default to opaque
                    
                    return Some(PdfColor::new(r, g, b, a));
                }
            }
        }
        
        // Try to extract fill color from rg command: "r g b rg" (for highlight)
        if let Some(rg_pos) = stream_content.find(" rg") {
            // Find the three numbers before "rg"
            let before_rg = &stream_content[..rg_pos];
            let parts: Vec<&str> = before_rg.split_whitespace().collect();
            
            // We need the last 3 numbers before "rg"
            if parts.len() >= 3 {
                // Try to parse the last 3 parts as numbers
                let r_str = parts[parts.len().saturating_sub(3)];
                let g_str = parts[parts.len().saturating_sub(2)];
                let b_str = parts[parts.len().saturating_sub(1)];
                
                if let (Ok(r_val), Ok(g_val), Ok(b_val)) = (
                    f64::from_str(r_str),
                    f64::from_str(g_str),
                    f64::from_str(b_str),
                ) {
                    // PDF colors are 0.0-1.0, convert to 0-255
                    let r = (r_val * 255.0).clamp(0.0, 255.0) as u8;
                    let g = (g_val * 255.0).clamp(0.0, 255.0) as u8;
                    let b = (b_val * 255.0).clamp(0.0, 255.0) as u8;
                    let a = 255u8; // Alpha is not in rg command, default to opaque
                    
                    return Some(PdfColor::new(r, g, b, a));
                }
            }
        }
        
        None
    }

    fn try_regenerate_appearance_stream_for_text_markup(&self) -> Result<(), PdfiumError> {
        // Check if this is a text markup annotation
        let pdfium_subtype = self.bindings.FPDFAnnot_GetSubtype(self.annotation_handle);
        let annotation_type = PdfPageAnnotationType::from_pdfium(pdfium_subtype)
            .unwrap_or(PdfPageAnnotationType::Unknown);

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("🔍 try_regenerate_appearance_stream_for_text_markup: pdfium_subtype={}, annotation_type={:?}", 
                pdfium_subtype, annotation_type).into());
        }

        let is_text_markup = matches!(
            annotation_type,
            PdfPageAnnotationType::Highlight
                | PdfPageAnnotationType::Underline
                | PdfPageAnnotationType::Strikeout
                | PdfPageAnnotationType::Squiggly
        );

        if !is_text_markup {
            // Not a text markup annotation, nothing to do
            return Ok(());
        }

        // Check if rect is valid
        // Note: PDFium may automatically update the rect when attachment points are set,
        // so we re-fetch it here to get the updated value
        let mut rect = FS_RECTF {
            left: 0.0,
            bottom: 0.0,
            right: 0.0,
            top: 0.0,
        };
        if !self.bindings.is_true(self.bindings.FPDFAnnot_GetRect(self.annotation_handle, &mut rect)) {
            // Can't get rect, skip
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"⚠️  Cannot get annotation rect, skipping appearance stream generation".into());
            }
            return Ok(());
        }

        let mut width = rect.right - rect.left;
        let mut height = rect.top - rect.bottom;
        
        // If rect is invalid, try to calculate it from attachment points
        if width <= 0.0 || height <= 0.0 {
            let attachment_count = self.len();
            if attachment_count > 0 {
                // Calculate rect from attachment points
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                
                for i in 0..attachment_count {
                    if let Ok(quad) = self.get(i) {
                        min_x = min_x.min(quad.x1.value).min(quad.x2.value).min(quad.x3.value).min(quad.x4.value);
                        max_x = max_x.max(quad.x1.value).max(quad.x2.value).max(quad.x3.value).max(quad.x4.value);
                        min_y = min_y.min(quad.y1.value).min(quad.y2.value).min(quad.y3.value).min(quad.y4.value);
                        max_y = max_y.max(quad.y1.value).max(quad.y2.value).max(quad.y3.value).max(quad.y4.value);
                    }
                }
                
                if min_x < max_x && min_y < max_y {
                    // Update rect with calculated bounds
                    // We do NOT add wave padding for squiggly annotations anymore,
                    // to match Underline implementation and prevent scaling issues.
                    let wave_padding = 0.0;
                    
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&format!("🔍 Calculating rect: annotation_type={:?}, wave_padding={:.2}, min_y={:.2}, max_y={:.2}", 
                            annotation_type, wave_padding, min_y, max_y).into());
                    }
                    
                    rect.left = min_x;
                    rect.bottom = min_y - wave_padding; // Expand downward for waves
                    rect.right = max_x;
                    rect.top = max_y + wave_padding; // Expand upward for waves
                    width = max_x - min_x;
                    height = (max_y + wave_padding) - (min_y - wave_padding);
                    
                    // Set the calculated rect in PDFium
                    if !self.bindings.is_true(self.bindings.FPDFAnnot_SetRect(self.annotation_handle, &rect)) {
                        #[cfg(target_arch = "wasm32")]
                        {
                            use web_sys::console;
                            console::log_1(&"⚠️  Failed to set calculated rect".into());
                        }
                    }
                    
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&format!("📐 Calculated rect from attachment points: left={:.2}, bottom={:.2}, right={:.2}, top={:.2}, height={:.2} (wave_padding={:.2})", 
                            rect.left, rect.bottom, rect.right, rect.top, height, wave_padding).into());
                    }
                } else {
                    // Still invalid, skip
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&format!("⚠️  Cannot calculate valid rect from attachment points, skipping appearance stream generation",).into());
                    }
                    return Ok(());
                }
            } else {
                // No attachment points and rect is invalid, skip
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&format!("⚠️  Rect is invalid (w={:.2}, h={:.2}) and no attachment points, skipping appearance stream generation", width, height).into());
                }
                return Ok(());
            }
        }

        // Rect is valid - but for squiggly annotations, we must expand it to include wave height
        // Waves extend ±2.0 from baseline, plus line width 1.0, so we need at least 4.0 padding on each side
        if annotation_type == PdfPageAnnotationType::Squiggly {
            // We do NOT expand rect anymore to prevent scaling issues.
            // Just update width/height based on current rect.
            width = rect.right - rect.left;
            height = rect.top - rect.bottom;
        }
        
        // Rect is valid - but for highlight/underline/strikeout/squiggly, we should recalculate it from all attachment points
        // to ensure it includes all lines (especially when multiple attachment points span multiple lines)
        let attachment_count = self.len();
        if attachment_count > 0 && (annotation_type == PdfPageAnnotationType::Highlight || annotation_type == PdfPageAnnotationType::Underline || annotation_type == PdfPageAnnotationType::Strikeout || annotation_type == PdfPageAnnotationType::Squiggly) {
            // Recalculate rect from all attachment points to ensure it includes all lines
            let mut min_x = f32::MAX;
            let mut min_y = f32::MAX;
            let mut max_x = f32::MIN;
            let mut max_y = f32::MIN;
            
            for i in 0..attachment_count {
                if let Ok(quad) = self.get(i) {
                    min_x = min_x.min(quad.x1.value).min(quad.x2.value).min(quad.x3.value).min(quad.x4.value);
                    max_x = max_x.max(quad.x1.value).max(quad.x2.value).max(quad.x3.value).max(quad.x4.value);
                    min_y = min_y.min(quad.y1.value).min(quad.y2.value).min(quad.y3.value).min(quad.y4.value);
                    max_y = max_y.max(quad.y1.value).max(quad.y2.value).max(quad.y3.value).max(quad.y4.value);
                }
            }
            
            if min_x < max_x && min_y < max_y {
                #[cfg(target_arch = "wasm32")]
                let old_left = rect.left;
                #[cfg(target_arch = "wasm32")]
                let old_bottom = rect.bottom;
                #[cfg(target_arch = "wasm32")]
                let old_right = rect.right;
                #[cfg(target_arch = "wasm32")]
                let old_top = rect.top;
                
                // Update rect to include all attachment points
                rect.left = min_x;
                rect.bottom = min_y;
                rect.right = max_x;
                rect.top = max_y;
                width = max_x - min_x;
                height = max_y - min_y;
                
                // Set the updated rect in PDFium
                if self.bindings.is_true(self.bindings.FPDFAnnot_SetRect(self.annotation_handle, &rect)) {
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        let type_name = match annotation_type {
                            PdfPageAnnotationType::Highlight => "highlight",
                            PdfPageAnnotationType::Underline => "underline",
                            PdfPageAnnotationType::Strikeout => "strikeout",
                            PdfPageAnnotationType::Squiggly => "squiggly",
                            _ => "unknown",
                        };
                        console::log_1(&format!("📐 Recalculated rect for {}: left={:.2}->{:.2}, bottom={:.2}->{:.2}, right={:.2}->{:.2}, top={:.2}->{:.2}", 
                            type_name,
                            old_left, rect.left, old_bottom, rect.bottom, old_right, rect.right, old_top, rect.top).into());
                    }
                    
                    // For squiggly, apply wave padding after recalculating from attachment points
                    if annotation_type == PdfPageAnnotationType::Squiggly {
                        // Wave height is 2.0, so waves extend ±2.0 from baseline
                        // Add extra padding to ensure waves aren't clipped, especially at the bottom
                        let wave_padding = 7.0; // Increased from 5.0 to 7.0 to prevent clipping
                        #[cfg(target_arch = "wasm32")]
                        let old_bottom = rect.bottom;
                        #[cfg(target_arch = "wasm32")]
                        let old_top = rect.top;
                        
                        // Expand rect to accommodate wave padding
                        rect.bottom = rect.bottom - wave_padding;
                        rect.top = rect.top + wave_padding;
                        width = rect.right - rect.left;
                        height = rect.top - rect.bottom;
                        
                        // Set the expanded rect in PDFium
                        if self.bindings.is_true(self.bindings.FPDFAnnot_SetRect(self.annotation_handle, &rect)) {
                            // Re-fetch the rect to ensure PDFium has the updated bounds
                            let mut updated_rect = FS_RECTF {
                                left: 0.0,
                                bottom: 0.0,
                                right: 0.0,
                                top: 0.0,
                            };
                            if self.bindings.is_true(self.bindings.FPDFAnnot_GetRect(self.annotation_handle, &mut updated_rect)) {
                                rect = updated_rect;
                                width = rect.right - rect.left;
                                height = rect.top - rect.bottom;
                            }
                            
                            #[cfg(target_arch = "wasm32")]
                            {
                                use web_sys::console;
                                console::log_1(&format!("📐 Applied wave padding to squiggly rect: bottom={:.2}->{:.2}, top={:.2}->{:.2} (final: w={:.2}, h={:.2})", 
                                    old_bottom, rect.bottom, old_top, rect.top, width, height).into());
                            }
                        }
                    }
                }
            }
        }
        
        // Rect is valid - generate appearance stream manually for highlights
        // PDFium's automatic generation doesn't properly use attachment points for highlights,
        // so we need to generate it manually. We'll use the fill color from the /IC dictionary.
        
        // Get color from annotation dictionary
        let mut r: u32 = 0;
        let mut g: u32 = 0;
        let mut b: u32 = 0;
        let mut a: u32 = 0;
        let get_color_result = self.bindings.FPDFAnnot_GetColor(
            self.annotation_handle,
            FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color,
            &mut r,
            &mut g,
            &mut b,
            &mut a,
        );
        let has_color = self.bindings.is_true(get_color_result);
        
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   🎨 Color retrieval: FPDFAnnot_GetColor returned={}, has_color={}, r={}, g={}, b={}, a={}", 
                get_color_result, has_color, r, g, b, a).into());
        }
        
        // If FPDFAnnot_GetColor failed (e.g., because appearance stream exists), try extracting from appearance stream
        let color_from_stream = if !has_color {
            let extracted = Self::extract_color_from_appearance_stream(self.annotation_handle, self.bindings);
            if let Some(color) = extracted {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&format!("   ✅ Color extracted from appearance stream: r={}, g={}, b={}, a={}", 
                        color.red(), color.green(), color.blue(), color.alpha()).into());
                }
                Some(color)
            } else {
                None
            }
        } else {
            None
        };
        
        // Default colors based on annotation type
        let (default_r, default_g, default_b) = match annotation_type {
            PdfPageAnnotationType::Highlight => (1.0, 1.0, 0.0), // Yellow for highlights
            _ => (0.0, 0.0, 0.0), // Black for underline, strikeout, squiggly
        };
        
        // Use color from dictionary if available, otherwise from appearance stream, otherwise default
        let (color_r, color_g, color_b) = if has_color {
            (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        } else if let Some(color) = color_from_stream {
            (color.red() as f32 / 255.0, color.green() as f32 / 255.0, color.blue() as f32 / 255.0)
        } else {
            (default_r, default_g, default_b)
        };
        
        // CRITICAL: For highlight annotations, always use fixed 0.3 opacity
        // Set /ca and /CA BEFORE generating appearance stream so PDFium can create Resources dictionary
        // when it processes the appearance stream
        if annotation_type == PdfPageAnnotationType::Highlight {
            let alpha = 0.3; // Fixed opacity for all highlights
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"═══════════════════════════════════════════════════════════".into());
                console::log_1(&"🔧 SETTING /ca and /CA to 0.3 for highlight annotation".into());
                console::log_1(&format!("   Setting fill opacity (/ca) to: {:.4}", alpha).into());
            }
            let _ = self.bindings.FPDFAnnot_SetNumberValue(
                self.annotation_handle,
                "ca",
                alpha,
            );
            let _ = self.bindings.FPDFAnnot_SetNumberValue(
                self.annotation_handle,
                "CA",
                alpha,
            );
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   ✅ Fill opacity (/ca, /CA) set to: {:.4}", alpha).into());
            }
        }

        // Generate appearance stream based on annotation type
        let content_stream = match annotation_type {
            PdfPageAnnotationType::Highlight => {
                // Highlight: filled quadrilaterals for each attachment point
                // Always use 0.3 opacity for highlights
                let mut stream = String::new();
                stream.push_str("q\n");
                
                // Apply ExtGState with opacity (always 0.3 for highlights)
                // CRITICAL: For flattening to work, ExtGState must be defined in the appearance
                // stream's Resources dictionary. We reference /GS here to match what PDFium creates.
                stream.push_str("/GS gs\n");
                
                stream.push_str(&format!("1 0 0 1 {:.4} {:.4} cm\n", rect.left, rect.bottom));
                stream.push_str(&format!("{:.4} {:.4} {:.4} rg\n", color_r, color_g, color_b));
                
                if attachment_count > 0 {
                    // Draw quadrilaterals for each attachment point
                    for i in 0..attachment_count {
                        if let Ok(quad) = self.get(i) {
                            // Convert quadpoint coordinates to relative coordinates
                            let x1_rel = quad.x1.value - rect.left;
                            let y1_rel = quad.y1.value - rect.bottom;
                            let x2_rel = quad.x2.value - rect.left;
                            let y2_rel = quad.y2.value - rect.bottom;
                            let x3_rel = quad.x3.value - rect.left;
                            let y3_rel = quad.y3.value - rect.bottom;
                            let x4_rel = quad.x4.value - rect.left;
                            let y4_rel = quad.y4.value - rect.bottom;
                            
                            // Draw filled quadrilateral
                            stream.push_str(&format!("{:.4} {:.4} m\n", x1_rel, y1_rel));
                            stream.push_str(&format!("{:.4} {:.4} l\n", x2_rel, y2_rel));
                            stream.push_str(&format!("{:.4} {:.4} l\n", x3_rel, y3_rel));
                            stream.push_str(&format!("{:.4} {:.4} l\n", x4_rel, y4_rel));
                            stream.push_str("h\n"); // closepath
                            stream.push_str("f\n"); // fill
                        }
                    }
                } else {
                    // No attachment points - use annotation rect as fallback
                    stream.push_str(&format!("0 0 {:.4} {:.4} re\n", width, height));
                    stream.push_str("f\n"); // fill
                }
                
                stream.push_str("Q\n");
                stream
            },
            PdfPageAnnotationType::Underline | PdfPageAnnotationType::Strikeout => {
                // Underline/Strikeout: horizontal lines for each attachment point
                let mut stream = String::new();
                stream.push_str("q\n");
                stream.push_str(&format!("1 0 0 1 {:.4} {:.4} cm\n", rect.left, rect.bottom));
                stream.push_str(&format!("{:.4} {:.4} {:.4} RG\n", color_r, color_g, color_b));
                stream.push_str("1 w\n"); // line width
                
                if attachment_count > 0 {
                    for i in 0..attachment_count {
                        if let Ok(quad) = self.get(i) {
                            let x_left = quad.left().value - rect.left;
                            let x_right = quad.right().value - rect.left;
                            
                            let y = if annotation_type == PdfPageAnnotationType::Underline {
                                quad.bottom().value - rect.bottom // Bottom of quadpoint
                            } else {
                                // Strikeout: middle of quadpoint
                                let quad_bottom = quad.bottom().value - rect.bottom;
                                let quad_top = quad.top().value - rect.bottom;
                                (quad_bottom + quad_top) / 2.0
                            };
                            
                            // Draw horizontal line from left to right
                            stream.push_str(&format!("{:.4} {:.4} m\n", x_left, y));
                            stream.push_str(&format!("{:.4} {:.4} l\n", x_right, y));
                            stream.push_str("S\n"); // stroke
                        }
                    }
                } else {
                    // No attachment points - use annotation rect as fallback
                    let y = if annotation_type == PdfPageAnnotationType::Underline {
                        0.0 // Bottom of rect
                    } else {
                        height / 2.0 // Middle of rect
                    };
                    stream.push_str(&format!("0 {:.4} m\n", y));
                    stream.push_str(&format!("{:.4} {:.4} l\n", width, y));
                    stream.push_str("S\n"); // stroke
                }
                
                stream.push_str("Q\n");
                stream
            },
            PdfPageAnnotationType::Squiggly => {
                // Squiggly: wavy bezier curves along the bottom of each attachment point
                // Use a fixed wave period (3.5 units per wave) for consistent bezier curves across all lines
                const WAVE_PERIOD: f32 = 3.5; // Fixed period in points - ensures consistent wave pattern
                const WAVE_HEIGHT: f32 = 2.0; // Fixed wave height
                
                let mut stream = String::new();
                stream.push_str("q\n");
                // Translate coordinate system to annotation's bottom-left corner (matches Underline implementation)
                stream.push_str(&format!("1 0 0 1 {:.4} {:.4} cm\n", rect.left, rect.bottom));
                stream.push_str(&format!("{:.4} {:.4} {:.4} RG\n", color_r, color_g, color_b));
                stream.push_str("1 w\n"); // line width
                
                if attachment_count > 0 {
                    for i in 0..attachment_count {
                        if let Ok(quad) = self.get(i) {
                            // Use relative coordinates to match Underline implementation
                            let x_left = quad.left().value - rect.left;
                            let x_right = quad.right().value - rect.left;
                            let y_bottom = quad.bottom().value - rect.bottom;
                            let quad_width = x_right - x_left;
                            
                            // Use fixed wave period for consistent bezier curves
                            // Calculate number of complete waves that fit, ensuring at least 2 waves
                            let wave_count = ((quad_width / WAVE_PERIOD).ceil().max(2.0) as i32).min(100);
                            let wave_width = quad_width / wave_count as f32;
                            
                            // Start at left bottom
                            stream.push_str(&format!("{:.4} {:.4} m\n", x_left, y_bottom));
                            
                            for wave in 0..wave_count {
                                let wave_start_x = x_left + wave as f32 * wave_width;
                                let wave_end_x = x_left + (wave + 1) as f32 * wave_width;
                                
                                // Create a bezier curve for this wave segment (S-shape)
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
                        }
                    }
                } else {
                    // No attachment points - use annotation rect as fallback
                    // Use fixed wave period for consistent bezier curves
                    let wave_count = ((width / WAVE_PERIOD).ceil().max(2.0) as i32).min(100);
                    let wave_width = width / wave_count as f32;
                    
                    // Start at left bottom (relative to rect origin, so 0,0)
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
                }
                
                stream.push_str("Q\n");
                stream
            },
            _ => {
                // For other types, let PDFium handle it automatically
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&format!("🔄 Skipping manual appearance stream for {:?} - allowing PDFium to auto-generate (rect: w={:.2}, h={:.2}, attachment_count={})", 
                        annotation_type, width, height, attachment_count).into());
                }
                return Ok(());
            }
        };
        
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("🔄 Setting manual appearance stream for {:?} ({} bytes)", annotation_type, content_stream.len()).into());
            
            // Log preview of the content stream to verify /GS gs is included
            let preview = if content_stream.len() > 200 {
                format!("{}...", &content_stream[..200])
            } else {
                content_stream.clone()
            };
            console::log_1(&format!("   Content stream preview:\n{}", preview).into());
            console::log_1(&format!("   Contains '/GS gs': {}", content_stream.contains("/GS gs")).into());
        }
        
        // Set the appearance stream
        let result = self.bindings.FPDFAnnot_SetAP_str(
            self.annotation_handle,
            0, // Normal mode
            &content_stream,
        );
        
        if self.bindings.is_true(result) {
            // Set Appearance State
            let _ = self.bindings.FPDFAnnot_SetStringValue_str(self.annotation_handle, "AS", "/N");
            
            // Verify the appearance stream after setting it
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                if annotation_type == PdfPageAnnotationType::Highlight {
                    let alpha = 0.3; // Fixed opacity for highlights
                    console::log_1(&"═══════════════════════════════════════════════════════════".into());
                    console::log_1(&"🔍 VERIFYING APPEARANCE STREAM AFTER SetAP".into());
                    
                    // Get the appearance stream back to verify it contains /GS gs
                    let ap_len = self.bindings.FPDFAnnot_GetAP(
                        self.annotation_handle,
                        0, // Normal mode
                        std::ptr::null_mut(),
                        0,
                    );
                    if ap_len > 2 {
                        let mut buffer = vec![0u16; (ap_len / 2) as usize];
                        let read_result = self.bindings.FPDFAnnot_GetAP(
                            self.annotation_handle,
                            0, // Normal mode
                            buffer.as_mut_ptr() as *mut crate::bindgen::FPDF_WCHAR,
                            ap_len,
                        );
                        if read_result == ap_len {
                            if let Ok(ap_content) = String::from_utf16(&buffer[..buffer.len().saturating_sub(1)]) {
                                console::log_1(&format!("   Appearance stream length: {} bytes", ap_content.len()).into());
                                console::log_1(&format!("   Contains '/GS gs': {}", ap_content.contains("/GS gs")).into());
                                console::log_1(&format!("   Contains '/GS1 gs': {}", ap_content.contains("/GS1 gs")).into());
                            }
                        }
                    }
                    
                    console::log_1(&format!("   ⚠️  Required: Resources/ExtGState/GS/ca = {:.4}", alpha).into());
                    console::log_1(&"═══════════════════════════════════════════════════════════".into());
                }
            }
            
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"✅ Appearance stream successfully set".into());
            }
            
            Ok(())
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"⚠️  Failed to set appearance stream".into());
            }
            Ok(()) // Don't error, just continue
        }
    }

    /// Returns an iterator over all the attachment points in this [PdfPageAnnotationAttachmentPoints] collection.
    #[inline]
    pub fn iter(&self) -> PdfPageAnnotationAttachmentPointsIterator<'_> {
        PdfPageAnnotationAttachmentPointsIterator::new(self)
    }
}

/// An iterator over all the attachment points in a [PdfPageAnnotationAttachmentPoints] collection.
pub struct PdfPageAnnotationAttachmentPointsIterator<'a> {
    attachment_points: &'a PdfPageAnnotationAttachmentPoints<'a>,
    next_index: PdfPageAnnotationAttachmentPointIndex,
}

impl<'a> PdfPageAnnotationAttachmentPointsIterator<'a> {
    #[inline]
    pub(crate) fn new(attachment_points: &'a PdfPageAnnotationAttachmentPoints<'a>) -> Self {
        PdfPageAnnotationAttachmentPointsIterator {
            attachment_points,
            next_index: 0,
        }
    }
}

impl<'a> Iterator for PdfPageAnnotationAttachmentPointsIterator<'a> {
    type Item = PdfQuadPoints;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.attachment_points.get(self.next_index);

        self.next_index += 1;

        next.ok()
    }
}
