//! Defines types and builders for setting the visual appearance of signature fields
//! using vector path data or inline bitmap images.
//!
//! # Overview
//!
//! PDF signature fields have two independent components:
//! - **Visual appearance**: How the signature looks on the page (handled by this module)
//! - **Cryptographic signature**: The digital signature data that validates the document
//!
//! This module handles ONLY the visual appearance. Cryptographic signing must be
//! performed separately using appropriate signing libraries or infrastructure.
//!
//! # Example
//!
//! ```rust,ignore
//! // Create signature strokes (like pen strokes from a signature pad)
//! let stroke = SignatureStroke::new()
//!     .with_stroke_width(1.5)
//!     .with_color(PdfColor::new(0, 0, 80, 255)) // Dark blue ink
//!     .move_to(10.0, 20.0)
//!     .curve_to(15.0, 30.0, 25.0, 30.0, 30.0, 20.0)
//!     .line_to(50.0, 5.0);
//!
//! // Apply to signature field
//! signature_field.set_signature_appearance()
//!     .add_stroke(stroke)
//!     .apply()?;
//! ```

use crate::bindgen::FPDF_ANNOTATION;
use crate::bindings::PdfiumLibraryBindings;
use crate::error::{PdfiumError, PdfiumInternalError};
use crate::pdf::appearance_mode::PdfAppearanceMode;
use crate::pdf::color::PdfColor;

/// Inline bitmap configuration for signature rendering
#[derive(Debug, Clone)]
pub struct InlineBitmapConfig {
    /// Width of the bitmap in pixels
    pub width: u32,
    /// Height of the bitmap in pixels  
    pub height: u32,
    /// Field width in PDF points (for coordinate mapping)
    pub field_width: f32,
    /// Field height in PDF points (for coordinate mapping)
    pub field_height: f32,
}

/// A 2D point for signature path drawing, in PDF user space coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignaturePathPoint {
    pub x: f32,
    pub y: f32,
}

impl SignaturePathPoint {
    /// Creates a new point with the given coordinates.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A segment in a signature stroke path.
///
/// These map directly to PDF path operators:
/// - `MoveTo` -> `x y m`
/// - `LineTo` -> `x y l`  
/// - `CurveTo` -> `x1 y1 x2 y2 x3 y3 c` (cubic Bezier)
/// - `Close` -> `h`
#[derive(Debug, Clone)]
pub enum SignaturePathSegment {
    /// Move to point without drawing (starts new subpath).
    /// Equivalent to lifting the pen and moving to a new position.
    MoveTo(SignaturePathPoint),

    /// Draw a straight line to the given point.
    LineTo(SignaturePathPoint),

    /// Draw a cubic Bezier curve.
    /// - `control1`: First control point (affects curve leaving current point)
    /// - `control2`: Second control point (affects curve arriving at end)
    /// - `end`: The endpoint of the curve
    CurveTo {
        control1: SignaturePathPoint,
        control2: SignaturePathPoint,
        end: SignaturePathPoint,
    },

    /// Close the current subpath by drawing a line back to the start.
    Close,
}

/// A single stroke in a signature, representing one continuous pen movement.
///
/// A typical handwritten signature consists of multiple strokes - each time
/// the pen is lifted and put back down starts a new stroke.
#[derive(Debug, Clone)]
pub struct SignatureStroke {
    segments: Vec<SignaturePathSegment>,
    stroke_width: f32,
    stroke_color: PdfColor,
}

impl Default for SignatureStroke {
    fn default() -> Self {
        Self::new()
    }
}

impl SignatureStroke {
    /// Creates a new empty signature stroke with default styling.
    /// Default: 1.0pt black stroke.
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            stroke_width: 1.0,
            stroke_color: PdfColor::BLACK,
        }
    }

    /// Sets the stroke width in points.
    /// Typical values: 0.5 - 2.0 for signature strokes.
    pub fn with_stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    /// Sets the stroke color.
    /// Common choices: dark blue (traditional ink), black.
    pub fn with_color(mut self, color: PdfColor) -> Self {
        self.stroke_color = color;
        self
    }

    /// Moves to a point without drawing (lifts the pen).
    /// This should typically be the first operation in a stroke.
    pub fn move_to(mut self, x: f32, y: f32) -> Self {
        self.segments
            .push(SignaturePathSegment::MoveTo(SignaturePathPoint::new(x, y)));
        self
    }

    /// Draws a straight line to the given point.
    pub fn line_to(mut self, x: f32, y: f32) -> Self {
        self.segments
            .push(SignaturePathSegment::LineTo(SignaturePathPoint::new(x, y)));
        self
    }

    /// Draws a cubic Bezier curve to the given endpoint.
    ///
    /// # Arguments
    /// * `cx1`, `cy1` - First control point
    /// * `cx2`, `cy2` - Second control point  
    /// * `x`, `y` - End point of the curve
    pub fn curve_to(mut self, cx1: f32, cy1: f32, cx2: f32, cy2: f32, x: f32, y: f32) -> Self {
        self.segments.push(SignaturePathSegment::CurveTo {
            control1: SignaturePathPoint::new(cx1, cy1),
            control2: SignaturePathPoint::new(cx2, cy2),
            end: SignaturePathPoint::new(x, y),
        });
        self
    }

    /// Closes the current subpath by drawing a line back to the start.
    pub fn close(mut self) -> Self {
        self.segments.push(SignaturePathSegment::Close);
        self
    }

    /// Returns the segments in this stroke.
    pub fn segments(&self) -> &[SignaturePathSegment] {
        &self.segments
    }

    /// Returns the stroke width.
    pub fn stroke_width(&self) -> f32 {
        self.stroke_width
    }

    /// Returns the stroke color.
    pub fn stroke_color(&self) -> &PdfColor {
        &self.stroke_color
    }
}

/// Builder for constructing and applying the visual appearance of a signature field.
///
/// This builder collects signature strokes and generates a PDF content stream
/// that renders them as vector paths or inline bitmaps. The resulting appearance 
/// is purely visual and does not affect cryptographic signature validity.
///
/// # Coordinate System
///
/// Coordinates are in PDF user space, where:
/// - Origin (0, 0) is at the bottom-left of the signature field
/// - X increases to the right
/// - Y increases upward
///
/// The coordinates should be relative to the signature field bounds.
/// Use [PdfPageAnnotationCommon::bounds()] to get the field dimensions.
///
/// [PdfPageAnnotationCommon::bounds()]: crate::pdf::document::page::annotation::PdfPageAnnotationCommon::bounds
pub struct SignatureAppearanceBuilder<'a> {
    bindings: &'a dyn PdfiumLibraryBindings,
    annotation_handle: FPDF_ANNOTATION,
    strokes: Vec<SignatureStroke>,
    bitmap_config: Option<InlineBitmapConfig>,
}

impl<'a> SignatureAppearanceBuilder<'a> {
    /// Creates a new builder for the given annotation.
    pub(crate) fn new(
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        Self {
            bindings,
            annotation_handle,
            strokes: Vec::new(),
            bitmap_config: None,
        }
    }

    /// Adds a stroke to the signature appearance.
    pub fn add_stroke(&mut self, stroke: SignatureStroke) -> &mut Self {
        self.strokes.push(stroke);
        self
    }

    /// Adds multiple strokes to the signature appearance.
    pub fn add_strokes(&mut self, strokes: impl IntoIterator<Item = SignatureStroke>) -> &mut Self {
        self.strokes.extend(strokes);
        self
    }

    /// Clears all strokes from the builder.
    pub fn clear(&mut self) -> &mut Self {
        self.strokes.clear();
        self
    }

    /// Sets the bitmap configuration for inline bitmap rendering.
    /// Call this before `apply_as_inline_bitmap()`.
    pub fn with_bitmap_config(&mut self, config: InlineBitmapConfig) -> &mut Self {
        self.bitmap_config = Some(config);
        self
    }

    /// Applies the signature appearance to the field.
    ///
    /// This sets the normal appearance stream (`/AP /N`) of the signature field.
    /// The appearance is purely visual and does not affect cryptographic signing.
    ///
    /// # Errors
    ///
    /// Returns an error if PDFium fails to set the appearance stream.
    pub fn apply(&self) -> Result<(), PdfiumError> {
        self.apply_with_mode(PdfAppearanceMode::Normal)
    }

    /// Helper to convert FPDF_OBJECT_TYPE to string for debugging
    #[cfg(target_arch = "wasm32")]
    fn object_type_name(obj_type: i32) -> &'static str {
        match obj_type {
            0 => "UNKNOWN",
            1 => "BOOLEAN", 
            2 => "NUMBER",
            3 => "STRING",
            4 => "NAME",
            5 => "ARRAY",
            6 => "DICTIONARY",
            7 => "STREAM",
            8 => "NULLOBJ",
            9 => "REFERENCE",
            _ => "INVALID",
        }
    }

    /// Helper to read a string value from annotation dictionary
    #[cfg(target_arch = "wasm32")]
    fn get_string_value(&self, key: &str) -> Option<String> {
        // First get the required buffer length
        let len = self.bindings.FPDFAnnot_GetStringValue(
            self.annotation_handle,
            key,
            std::ptr::null_mut(),
            0,
        );
        
        if len <= 2 {
            return None; // Empty or not found
        }
        
        // Allocate buffer and read value
        let mut buffer: Vec<u16> = vec![0; (len / 2 + 1) as usize];
        let read_len = self.bindings.FPDFAnnot_GetStringValue(
            self.annotation_handle,
            key,
            buffer.as_mut_ptr() as *mut u16,
            len,
        );
        
        if read_len > 2 {
            Some(String::from_utf16_lossy(&buffer[..((read_len / 2) as usize).saturating_sub(1)]))
        } else {
            None
        }
    }

    /// Applies the signature appearance with a specific appearance mode.
    ///
    /// Most signatures only need the Normal appearance. RollOver and Down
    /// appearances are used for interactive hover/click states.
    pub fn apply_with_mode(&self, mode: PdfAppearanceMode) -> Result<(), PdfiumError> {
        let content_stream = self.build_content_stream();

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&"🔧 SignatureAppearanceBuilder::apply_with_mode() - DETAILED DEBUG".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&format!("   Strokes count: {}", self.strokes.len()).into());
            console::log_1(&format!("   Content stream length: {} chars", content_stream.len()).into());
            // Log first 300 chars of content stream
            let preview: String = content_stream.chars().take(300).collect();
            console::log_1(&format!("   Content stream preview:\n{}", preview).into());
            
            // === COMPREHENSIVE ANNOTATION DICTIONARY ANALYSIS ===
            console::log_1(&"".into());
            console::log_1(&"📋 ANNOTATION DICTIONARY STRUCTURE (BEFORE SetAP):".into());
            console::log_1(&"─────────────────────────────────────────────────".into());
            
            // Check annotation subtype
            let subtype = self.bindings.FPDFAnnot_GetSubtype(self.annotation_handle);
            console::log_1(&format!("   /Subtype: {} (20=Widget)", subtype).into());
            
            // Check annotation flags
            let flags = self.bindings.FPDFAnnot_GetFlags(self.annotation_handle);
            console::log_1(&format!("   /F (flags): {} (binary: {:b})", flags, flags).into());
            
            // === CHECK ALL RELEVANT KEYS ===
            let keys_to_check = [
                ("AP", "Appearance dictionary"),
                ("Parent", "Parent form field reference"),
                ("T", "Field name"),
                ("TU", "Alternate field name"),
                ("FT", "Field type"),
                ("V", "Field value"),
                ("DV", "Default value"),
                ("Ff", "Field flags"),
                ("Rect", "Bounding rectangle"),
                ("P", "Page reference"),
                ("DR", "Default resources"),
                ("DA", "Default appearance string"),
                ("MK", "Widget appearance characteristics"),
            ];
            
            console::log_1(&"".into());
            console::log_1(&"   Key presence and types:".into());
            for (key, description) in keys_to_check.iter() {
                let has_key = self.bindings.FPDFAnnot_HasKey(self.annotation_handle, key);
                if self.bindings.is_true(has_key) {
                    let value_type = self.bindings.FPDFAnnot_GetValueType(self.annotation_handle, key);
                    let type_name = Self::object_type_name(value_type);
                    
                    // Try to get string value if it's a string/name type
                    let value_preview = if value_type == 3 || value_type == 4 {
                        self.get_string_value(key)
                            .map(|s| format!(" = \"{}\"", s.chars().take(50).collect::<String>()))
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    
                    console::log_1(&format!("   ✓ /{}: {} ({}){}", key, value_type, type_name, value_preview).into());
                } else {
                    console::log_1(&format!("   ✗ /{}: NOT PRESENT ({})", key, description).into());
                }
            }
            
            // === TRY TO GET LINKED PARENT ANNOTATION ===
            console::log_1(&"".into());
            console::log_1(&"   Checking linked annotations:".into());
            let linked_parent = self.bindings.FPDFAnnot_GetLinkedAnnot(self.annotation_handle, "Parent");
            if !linked_parent.is_null() {
                console::log_1(&"   ✓ FPDFAnnot_GetLinkedAnnot('Parent') returned valid handle".into());
                // Check parent's type
                let parent_subtype = self.bindings.FPDFAnnot_GetSubtype(linked_parent);
                console::log_1(&format!("     Parent annotation subtype: {}", parent_subtype).into());
                // Close the linked annotation handle
                self.bindings.FPDFPage_CloseAnnot(linked_parent);
            } else {
                console::log_1(&"   ✗ FPDFAnnot_GetLinkedAnnot('Parent') returned NULL".into());
                console::log_1(&"     This suggests annotation dict may be:".into());
                console::log_1(&"     a) A merged field/widget dictionary, OR".into());
                console::log_1(&"     b) A DIRECT object (not indirect) in /Annots array".into());
            }
            
            // === CHECK AP DICTIONARY IN DETAIL ===
            console::log_1(&"".into());
            console::log_1(&"   Appearance Dictionary (/AP) analysis:".into());
            let has_ap = self.bindings.FPDFAnnot_HasKey(self.annotation_handle, "AP");
            if self.bindings.is_true(has_ap) {
                // Check the type of AP/N (normal appearance)
                let ap_type = self.bindings.FPDFAnnot_GetValueType(self.annotation_handle, "AP");
                console::log_1(&format!("   /AP type: {} ({})", ap_type, Self::object_type_name(ap_type)).into());
                
                // Get AP stream length for Normal appearance
                let ap_before_len = self.bindings.FPDFAnnot_GetAP(
                    self.annotation_handle,
                    mode.as_pdfium(),
                    std::ptr::null_mut(),
                    0,
                );
                console::log_1(&format!("   /AP/N stream content length: {} bytes", ap_before_len).into());
                
                if ap_before_len > 2 {
                    let mut buffer: Vec<u16> = vec![0; (ap_before_len / 2 + 1) as usize];
                    let read_len = self.bindings.FPDFAnnot_GetAP(
                        self.annotation_handle,
                        mode.as_pdfium(),
                        buffer.as_mut_ptr() as *mut u16,
                        ap_before_len,
                    );
                    if read_len > 0 {
                        let ap_str = String::from_utf16_lossy(&buffer[..((read_len / 2) as usize).saturating_sub(1)]);
                        let ap_preview: String = ap_str.chars().take(100).collect();
                        console::log_1(&format!("   /AP/N content preview: \"{}\"", ap_preview).into());
                    }
                }
            } else {
                console::log_1(&"   /AP not present - will be created by SetAP".into());
            }
            
            // === CHECK ANNOTATION OBJECT COUNT ===
            let obj_count_before = self.bindings.FPDFAnnot_GetObjectCount(self.annotation_handle);
            console::log_1(&format!("   Annotation page objects count: {}", obj_count_before).into());
            
            console::log_1(&"".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&"📝 CALLING FPDFAnnot_SetAP_str()...".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
        }

        let result = self.bindings.FPDFAnnot_SetAP_str(
            self.annotation_handle,
            mode.as_pdfium(),
            &content_stream,
        );

        // Fix: Explicitly set the Appearance State (/AS) to match the mode (usually "/N")
        // This ensures viewers know which appearance stream to display.
        // Use leading slash like checkboxes do (e.g., "/Yes", "/Off") - PDF name objects need the slash.
        let mode_str = match mode {
            PdfAppearanceMode::Normal => "/N",
            PdfAppearanceMode::RollOver => "/R",
            PdfAppearanceMode::Down => "/D",
        };
        let as_result = self.bindings.FPDFAnnot_SetStringValue_str(self.annotation_handle, "AS", mode_str);
        
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   Setting /AS to '{}': {}", mode_str, 
                if self.bindings.is_true(as_result) { "✅ success" } else { "❌ failed" }).into());
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   FPDFAnnot_SetAP_str returned: {} (1=success, 0=failure)", 
                if self.bindings.is_true(result) { 1 } else { 0 }).into());
            
            // === POST-SETAP ANALYSIS ===
            console::log_1(&"".into());
            console::log_1(&"📋 ANNOTATION DICTIONARY STRUCTURE (AFTER SetAP):".into());
            console::log_1(&"─────────────────────────────────────────────────".into());
            
            // Re-check AP key
            let has_ap_after = self.bindings.FPDFAnnot_HasKey(self.annotation_handle, "AP");
            console::log_1(&format!("   Has /AP key: {}", self.bindings.is_true(has_ap_after)).into());
            
            if self.bindings.is_true(has_ap_after) {
                let ap_type_after = self.bindings.FPDFAnnot_GetValueType(self.annotation_handle, "AP");
                console::log_1(&format!("   /AP type: {} ({})", ap_type_after, Self::object_type_name(ap_type_after)).into());
            }
            
            // Check AP AFTER setting to verify it was stored
            let ap_after_len = self.bindings.FPDFAnnot_GetAP(
                self.annotation_handle,
                mode.as_pdfium(),
                std::ptr::null_mut(),
                0,
            );
            console::log_1(&format!("   /AP/N stream content length AFTER: {} bytes", ap_after_len).into());
            
            if ap_after_len > 2 {
                // Allocate buffer and read the AP back
                let mut buffer: Vec<u16> = vec![0; (ap_after_len / 2 + 1) as usize];
                let read_len = self.bindings.FPDFAnnot_GetAP(
                    self.annotation_handle,
                    mode.as_pdfium(),
                    buffer.as_mut_ptr() as *mut u16,
                    ap_after_len,
                );
                if read_len > 0 {
                    let ap_str = String::from_utf16_lossy(&buffer[..((read_len / 2) as usize).saturating_sub(1)]);
                    let ap_preview: String = ap_str.chars().take(150).collect();
                    console::log_1(&format!("   ✅ AP content verified:\n   {}", ap_preview).into());
                }
            } else {
                console::warn_1(&format!("   ⚠️ AP appears empty after SetAP! Length={}", ap_after_len).into());
            }
            
            // Check object count after
            let obj_count_after = self.bindings.FPDFAnnot_GetObjectCount(self.annotation_handle);
            console::log_1(&format!("   Annotation page objects count: {}", obj_count_after).into());
            
            console::log_1(&"".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&"🔍 SUMMARY:".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&format!("   - SetAP return value: {}", if self.bindings.is_true(result) { "SUCCESS" } else { "FAILED" }).into());
            console::log_1(&format!("   - AP content length change: 34 bytes -> {} bytes", ap_after_len).into());
            console::log_1(&"   - NOTE: If save fails, the annotation dictionary may be a".into());
            console::log_1(&"     DIRECT object in /Annots array, not an indirect object.".into());
            console::log_1(&"     PDFium's serializer may not properly handle this case.".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
        }

        if self.bindings.is_true(result) {
            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Gets the annotation rectangle (position on page)
    fn get_annotation_rect(&self) -> (f32, f32, f32, f32) {
        let mut rect = crate::bindgen::FS_RECTF {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        };
        
        let success = self.bindings.FPDFAnnot_GetRect(self.annotation_handle, &mut rect);
        
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("📐 FPDFAnnot_GetRect: success={}, left={:.2}, bottom={:.2}, right={:.2}, top={:.2}",
                if self.bindings.is_true(success) { "true" } else { "false" },
                rect.left, rect.bottom, rect.right, rect.top).into());
        }
        
        (rect.left, rect.bottom, rect.right, rect.top)
    }

    /// Builds the PDF content stream string from all strokes.
    fn build_content_stream(&self) -> String {
        let mut stream = String::with_capacity(self.strokes.len() * 100);

        // Get the annotation rectangle to translate coordinates
        let (left, bottom, right, top) = self.get_annotation_rect();
        let width = right - left;
        let height = top - bottom;

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("📝 Building content stream:").into());
            console::log_1(&format!("   Annotation rect: left={:.2}, bottom={:.2}, right={:.2}, top={:.2}", left, bottom, right, top).into());
            console::log_1(&format!("   Size: {:.2} x {:.2}", width, height).into());
        }

        // Save graphics state
        stream.push_str("q\n");
        
        // CRITICAL: Translate coordinate system to annotation's bottom-left corner
        // The appearance stream's BBox is at the annotation's page position,
        // but drawing commands are relative to local origin (0,0).
        // Without this translation, content would be drawn at the wrong position
        // and clipped out of view.
        stream.push_str(&format!("1 0 0 1 {:.4} {:.4} cm\n", left, bottom));
        
        // Round line caps (pen-like appearance)
        stream.push_str("1 J\n");
        // Round line joins
        stream.push_str("1 j\n");

        for stroke in &self.strokes {
            // Set stroke color (RGB)
            let r = stroke.stroke_color.red() as f32 / 255.0;
            let g = stroke.stroke_color.green() as f32 / 255.0;
            let b = stroke.stroke_color.blue() as f32 / 255.0;
            stream.push_str(&format!("{:.4} {:.4} {:.4} RG\n", r, g, b));

            // Set line width
            stream.push_str(&format!("{:.4} w\n", stroke.stroke_width));

            // Build path from segments
            for segment in &stroke.segments {
                match segment {
                    SignaturePathSegment::MoveTo(p) => {
                        stream.push_str(&format!("{:.4} {:.4} m\n", p.x, p.y));
                    }
                    SignaturePathSegment::LineTo(p) => {
                        stream.push_str(&format!("{:.4} {:.4} l\n", p.x, p.y));
                    }
                    SignaturePathSegment::CurveTo {
                        control1,
                        control2,
                        end,
                    } => {
                        stream.push_str(&format!(
                            "{:.4} {:.4} {:.4} {:.4} {:.4} {:.4} c\n",
                            control1.x, control1.y, control2.x, control2.y, end.x, end.y,
                        ));
                    }
                    SignaturePathSegment::Close => {
                        stream.push_str("h\n");
                    }
                }
            }

            // Stroke the path
            stream.push_str("S\n");
        }

        // Restore graphics state
        stream.push_str("Q\n");

        stream
    }

    /// Applies the signature appearance as an inline bitmap image.
    /// 
    /// This is an experimental method to test if inline images serialize
    /// correctly when vector paths don't. The signature strokes are rasterized
    /// to a grayscale bitmap and embedded directly in the content stream.
    ///
    /// You must call `with_bitmap_config()` before calling this method.
    ///
    /// # Errors
    ///
    /// Returns an error if bitmap config is not set or if PDFium fails.
    pub fn apply_as_inline_bitmap(&self) -> Result<(), PdfiumError> {
        self.apply_as_inline_bitmap_with_mode(PdfAppearanceMode::Normal)
    }

    /// Applies the signature as an inline bitmap with a specific appearance mode.
    pub fn apply_as_inline_bitmap_with_mode(&self, mode: PdfAppearanceMode) -> Result<(), PdfiumError> {
        let config = self.bitmap_config.as_ref().ok_or_else(|| {
            PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::Unknown)
        })?;

        // Rasterize strokes to a grayscale bitmap
        let bitmap = self.rasterize_strokes(config);
        
        // Build content stream with inline image
        let content_stream = self.build_inline_bitmap_content_stream(config, &bitmap);

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&"🖼️ INLINE BITMAP SIGNATURE - DETAILED DIAGNOSTIC".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&format!("   Bitmap size: {}x{} pixels", config.width, config.height).into());
            console::log_1(&format!("   Bitmap data size: {} bytes", bitmap.len()).into());
            console::log_1(&format!("   Content stream length: {} chars", content_stream.len()).into());
            let preview: String = content_stream.chars().take(200).collect();
            console::log_1(&format!("   Content stream preview:\n{}", preview).into());
            
            // === COMPREHENSIVE ANNOTATION DICTIONARY ANALYSIS ===
            console::log_1(&"".into());
            console::log_1(&"📋 ANNOTATION DICTIONARY STRUCTURE (BEFORE SetAP):".into());
            console::log_1(&"─────────────────────────────────────────────────".into());
            
            // Check annotation subtype
            let subtype = self.bindings.FPDFAnnot_GetSubtype(self.annotation_handle);
            console::log_1(&format!("   /Subtype: {} (20=Widget)", subtype).into());
            
            // Check annotation flags
            let flags = self.bindings.FPDFAnnot_GetFlags(self.annotation_handle);
            console::log_1(&format!("   /F (flags): {} (binary: {:b})", flags, flags).into());
            
            // === CHECK ALL RELEVANT KEYS ===
            let keys_to_check = [
                ("AP", "Appearance dictionary"),
                ("Parent", "Parent form field reference - ABSENT = merged dict"),
                ("T", "Field name (should be on merged or parent)"),
                ("TU", "Alternate field name"),
                ("FT", "Field type (Sig for signature)"),
                ("V", "Field value/signature dict"),
                ("DV", "Default value"),
                ("Ff", "Field flags"),
                ("Rect", "Bounding rectangle"),
                ("P", "Page reference"),
                ("DR", "Default resources"),
                ("DA", "Default appearance string"),
                ("MK", "Widget appearance characteristics"),
                ("Kids", "Child fields (if any)"),
            ];
            
            console::log_1(&"".into());
            console::log_1(&"   Dictionary key analysis:".into());
            let mut has_field_keys = false;
            for (key, description) in keys_to_check.iter() {
                let has_key = self.bindings.FPDFAnnot_HasKey(self.annotation_handle, key);
                if self.bindings.is_true(has_key) {
                    let value_type = self.bindings.FPDFAnnot_GetValueType(self.annotation_handle, key);
                    let type_name = Self::object_type_name(value_type);
                    
                    // Track if we find form field keys (indicates merged dict)
                    if *key == "FT" || *key == "T" || *key == "V" {
                        has_field_keys = true;
                    }
                    
                    // Try to get string value if it's a string/name type
                    let value_preview = if value_type == 3 || value_type == 4 {
                        self.get_string_value(key)
                            .map(|s| format!(" = \"{}\"", s.chars().take(50).collect::<String>()))
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    
                    console::log_1(&format!("   ✓ /{}: {} ({}){}", key, value_type, type_name, value_preview).into());
                } else {
                    console::log_1(&format!("   ✗ /{}: NOT PRESENT - {}", key, description).into());
                }
            }
            
            // === MERGED DICTIONARY DETECTION ===
            console::log_1(&"".into());
            console::log_1(&"🔍 STRUCTURE TYPE DETECTION:".into());
            let has_parent = self.bindings.FPDFAnnot_HasKey(self.annotation_handle, "Parent");
            if !self.bindings.is_true(has_parent) && has_field_keys {
                console::log_1(&"   ⚠️ MERGED FIELD/WIDGET DICTIONARY DETECTED!".into());
                console::log_1(&"   - No /Parent key but has form field keys (FT, T, V)".into());
                console::log_1(&"   - This dict contains both field AND widget properties".into());
                console::log_1(&"   - If this dict is a DIRECT object in /Annots array,".into());
                console::log_1(&"     then modifications may NOT be serialized correctly!".into());
            } else if !self.bindings.is_true(has_parent) {
                console::log_1(&"   ❓ No /Parent key and no form field keys".into());
                console::log_1(&"   - Unusual structure for a signature field".into());
            } else {
                console::log_1(&"   ✓ Standard split field/widget structure".into());
            }
            
            // === TRY TO GET LINKED PARENT ANNOTATION ===
            console::log_1(&"".into());
            console::log_1(&"   Checking FPDFAnnot_GetLinkedAnnot with various keys:".into());
            for link_key in ["Parent", "P", "Kids"].iter() {
                let linked = self.bindings.FPDFAnnot_GetLinkedAnnot(self.annotation_handle, link_key);
                if !linked.is_null() {
                    let linked_subtype = self.bindings.FPDFAnnot_GetSubtype(linked);
                    console::log_1(&format!("   ✓ Linked '{}': got handle (subtype={})", link_key, linked_subtype).into());
                    self.bindings.FPDFPage_CloseAnnot(linked);
                } else {
                    console::log_1(&format!("   ✗ Linked '{}': NULL", link_key).into());
                }
            }
            
            // === CHECK AP DICTIONARY IN DETAIL ===
            console::log_1(&"".into());
            let has_ap = self.bindings.FPDFAnnot_HasKey(self.annotation_handle, "AP");
            if self.bindings.is_true(has_ap) {
                let ap_type = self.bindings.FPDFAnnot_GetValueType(self.annotation_handle, "AP");
                console::log_1(&format!("   /AP type: {} ({})", ap_type, Self::object_type_name(ap_type)).into());
            }
            
            // Check AP stream length for Normal appearance
            let ap_before_len = self.bindings.FPDFAnnot_GetAP(
                self.annotation_handle,
                mode.as_pdfium(),
                std::ptr::null_mut(),
                0,
            );
            console::log_1(&format!("   /AP/N stream content: {} bytes BEFORE", ap_before_len).into());
            
            // Check object count before
            let obj_count_before = self.bindings.FPDFAnnot_GetObjectCount(self.annotation_handle);
            console::log_1(&format!("   Page objects in annotation: {}", obj_count_before).into());
            
            console::log_1(&"".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&"📝 CALLING FPDFAnnot_SetAP_str()...".into());
            console::log_1(&format!("   Setting {} bytes of content stream", content_stream.len()).into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
        }

        let result = self.bindings.FPDFAnnot_SetAP_str(
            self.annotation_handle,
            mode.as_pdfium(),
            &content_stream,
        );

        // Fix: Explicitly set the Appearance State (/AS) to match the mode (usually "/N")
        // This ensures viewers know which appearance stream to display.
        // Use leading slash like checkboxes do (e.g., "/Yes", "/Off") - PDF name objects need the slash.
        let mode_str = match mode {
            PdfAppearanceMode::Normal => "/N",
            PdfAppearanceMode::RollOver => "/R",
            PdfAppearanceMode::Down => "/D",
        };
        let as_result = self.bindings.FPDFAnnot_SetStringValue_str(self.annotation_handle, "AS", mode_str);
        
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   Setting /AS to '{}': {}", mode_str, 
                if self.bindings.is_true(as_result) { "✅ success" } else { "❌ failed" }).into());
        }

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            let success = self.bindings.is_true(result);
            console::log_1(&format!("   FPDFAnnot_SetAP_str returned: {} ({})", 
                if success { 1 } else { 0 },
                if success { "SUCCESS" } else { "FAILED" }
            ).into());
            
            // === POST-SETAP ANALYSIS ===
            console::log_1(&"".into());
            console::log_1(&"📋 POST-SETAP ANALYSIS:".into());
            console::log_1(&"─────────────────────────────────────────────────".into());
            
            // Re-check AP key type after SetAP
            let has_ap_after = self.bindings.FPDFAnnot_HasKey(self.annotation_handle, "AP");
            console::log_1(&format!("   Has /AP key: {}", self.bindings.is_true(has_ap_after)).into());
            
            if self.bindings.is_true(has_ap_after) {
                let ap_type_after = self.bindings.FPDFAnnot_GetValueType(self.annotation_handle, "AP");
                console::log_1(&format!("   /AP type: {} ({})", ap_type_after, Self::object_type_name(ap_type_after)).into());
            }
            
            let ap_after_len = self.bindings.FPDFAnnot_GetAP(
                self.annotation_handle,
                mode.as_pdfium(),
                std::ptr::null_mut(),
                0,
            );
            console::log_1(&format!("   /AP/N stream content: {} bytes AFTER", ap_after_len).into());
            
            if ap_after_len > 2 {
                let mut buffer: Vec<u16> = vec![0; (ap_after_len / 2 + 1) as usize];
                let read_len = self.bindings.FPDFAnnot_GetAP(
                    self.annotation_handle,
                    mode.as_pdfium(),
                    buffer.as_mut_ptr() as *mut u16,
                    ap_after_len,
                );
                if read_len > 0 {
                    let ap_str = String::from_utf16_lossy(&buffer[..((read_len / 2) as usize).saturating_sub(1)]);
                    let ap_preview: String = ap_str.chars().take(100).collect();
                    console::log_1(&format!("   ✅ AP content verified: \"{}...\"", ap_preview).into());
                }
            }
            
            // Check object count after
            let obj_count_after = self.bindings.FPDFAnnot_GetObjectCount(self.annotation_handle);
            console::log_1(&format!("   Page objects in annotation: {}", obj_count_after).into());
            
            console::log_1(&"".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&"🔍 DIAGNOSTIC SUMMARY:".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&format!("   SetAP: {}", if success { "✅ SUCCESS" } else { "❌ FAILED" }).into());
            console::log_1(&format!("   AP content: {} bytes stored in memory", ap_after_len).into());
            console::log_1(&"".into());
            console::log_1(&"   ⚠️ KNOWN ISSUE: If saved PDF is smaller than original,".into());
            console::log_1(&"   the appearance stream data is NOT being serialized.".into());
            console::log_1(&"".into());
            console::log_1(&"   PROBABLE CAUSE: The annotation dictionary is either:".into());
            console::log_1(&"   1. A DIRECT object in /Annots (not tracked for save)".into());
            console::log_1(&"   2. A merged field/widget dict without indirect reference".into());
            console::log_1(&"".into());
            console::log_1(&"   PDFium's serializer (FPDF_SaveWithVersion) may not".into());
            console::log_1(&"   properly traverse and serialize all referenced objects".into());
            console::log_1(&"   when the annotation itself is a direct/inline object.".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
        }

        if self.bindings.is_true(result) {
            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Rasterizes signature strokes to a grayscale bitmap.
    /// Returns a Vec<u8> where each byte is a grayscale pixel (0=black, 255=white).
    fn rasterize_strokes(&self, config: &InlineBitmapConfig) -> Vec<u8> {
        let width = config.width as usize;
        let height = config.height as usize;
        
        // Start with white background
        let mut bitmap = vec![255u8; width * height];
        
        // Scale factors from PDF coordinates to bitmap pixels
        let scale_x = config.width as f32 / config.field_width;
        let scale_y = config.height as f32 / config.field_height;
        
        for stroke in &self.strokes {
            let stroke_width = (stroke.stroke_width * scale_x).max(1.0);
            let half_width = (stroke_width / 2.0).ceil() as i32;
            
            // Get grayscale value from stroke color (inverted: 0=ink, 255=white)
            let gray = 255 - ((stroke.stroke_color.red() as u32 
                + stroke.stroke_color.green() as u32 
                + stroke.stroke_color.blue() as u32) / 3) as u8;
            let ink = gray.min(50); // Dark ink
            
            let mut current_x = 0.0f32;
            let mut current_y = 0.0f32;
            
            for segment in stroke.segments() {
                match segment {
                    SignaturePathSegment::MoveTo(p) => {
                        current_x = p.x * scale_x;
                        // Flip Y: PDF Y-up to bitmap Y-down
                        current_y = (config.field_height - p.y) * scale_y;
                    }
                    SignaturePathSegment::LineTo(p) => {
                        let end_x = p.x * scale_x;
                        let end_y = (config.field_height - p.y) * scale_y;
                        
                        // Draw line using Bresenham-like algorithm with thickness
                        self.draw_thick_line(
                            &mut bitmap, width, height,
                            current_x, current_y, end_x, end_y,
                            half_width, ink,
                        );
                        
                        current_x = end_x;
                        current_y = end_y;
                    }
                    SignaturePathSegment::CurveTo { control1, control2, end } => {
                        // Approximate Bezier curve with line segments
                        let steps = 10;
                        let start_x = current_x;
                        let start_y = current_y;
                        
                        for i in 1..=steps {
                            let t = i as f32 / steps as f32;
                            let t2 = t * t;
                            let t3 = t2 * t;
                            let mt = 1.0 - t;
                            let mt2 = mt * mt;
                            let mt3 = mt2 * mt;
                            
                            // Cubic Bezier formula
                            let bx = mt3 * (start_x / scale_x)
                                + 3.0 * mt2 * t * control1.x
                                + 3.0 * mt * t2 * control2.x
                                + t3 * end.x;
                            let by = mt3 * ((config.field_height - start_y / scale_y))
                                + 3.0 * mt2 * t * control1.y
                                + 3.0 * mt * t2 * control2.y
                                + t3 * end.y;
                            
                            let next_x = bx * scale_x;
                            let next_y = (config.field_height - by) * scale_y;
                            
                            self.draw_thick_line(
                                &mut bitmap, width, height,
                                current_x, current_y, next_x, next_y,
                                half_width, ink,
                            );
                            
                            current_x = next_x;
                            current_y = next_y;
                        }
                    }
                    SignaturePathSegment::Close => {
                        // Close handled by path structure
                    }
                }
            }
        }
        
        bitmap
    }
    
    /// Draws a thick line on the bitmap using simple circle stamps.
    fn draw_thick_line(
        &self,
        bitmap: &mut [u8],
        width: usize,
        height: usize,
        x0: f32, y0: f32,
        x1: f32, y1: f32,
        half_width: i32,
        ink: u8,
    ) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt();
        let steps = (len * 2.0).max(1.0) as i32;
        
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let px = (x0 + dx * t) as i32;
            let py = (y0 + dy * t) as i32;
            
            // Draw a filled circle at this point
            for dy in -half_width..=half_width {
                for dx in -half_width..=half_width {
                    if dx * dx + dy * dy <= half_width * half_width {
                        let x = px + dx;
                        let y = py + dy;
                        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                            let idx = y as usize * width + x as usize;
                            // Blend with existing (darker wins)
                            bitmap[idx] = bitmap[idx].min(ink);
                        }
                    }
                }
            }
        }
    }
    
    /// Builds a content stream with an inline grayscale image.
    fn build_inline_bitmap_content_stream(&self, config: &InlineBitmapConfig, bitmap: &[u8]) -> String {
        let mut stream = String::new();
        
        // Get the annotation rectangle to translate coordinates
        let (left, bottom, _right, _top) = self.get_annotation_rect();
        
        // Save graphics state
        stream.push_str("q\n");
        
        // Transform: translate to annotation position, then scale the image
        // cm operator: a b c d e f - transformation matrix
        // Combined matrix: scale by field dimensions AND translate to (left, bottom)
        stream.push_str(&format!(
            "{:.4} 0 0 {:.4} {:.4} {:.4} cm\n",
            config.field_width,
            config.field_height,
            left,
            bottom
        ));
        
        // Begin inline image
        stream.push_str("BI\n");
        stream.push_str(&format!("/W {}\n", config.width));
        stream.push_str(&format!("/H {}\n", config.height));
        stream.push_str("/BPC 8\n");  // 8 bits per component
        stream.push_str("/CS /G\n");   // Grayscale colorspace
        stream.push_str("ID ");        // Image data follows
        
        // Encode bitmap data as ASCII hex (simple, always works)
        // Note: We could use ASCIIHexDecode filter, but raw hex inline is simpler
        for byte in bitmap {
            stream.push_str(&format!("{:02X}", byte));
        }
        
        stream.push_str("\nEI\n");  // End inline image
        
        // Restore graphics state
        stream.push_str("Q\n");
        
        stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_stroke_builder() {
        let stroke = SignatureStroke::new()
            .with_stroke_width(1.5)
            .with_color(PdfColor::BLUE)
            .move_to(10.0, 20.0)
            .line_to(30.0, 40.0)
            .curve_to(35.0, 45.0, 40.0, 45.0, 45.0, 40.0)
            .close();

        assert_eq!(stroke.stroke_width(), 1.5);
        assert_eq!(stroke.segments().len(), 4);
    }

    #[test]
    fn test_signature_path_point() {
        let point = SignaturePathPoint::new(10.5, 20.5);
        assert_eq!(point.x, 10.5);
        assert_eq!(point.y, 20.5);
    }
}

