//! Defines the [PdfPagePolygonAnnotation] struct, exposing functionality related to a single
//! user annotation of type [PdfPageAnnotationType::Polygon].

use crate::bindgen::{
    FPDF_ANNOTATION, FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color,
    FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_InteriorColor, FPDF_DOCUMENT, FPDF_PAGE, FS_POINTF,
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
use crate::pdf::points::PdfPoints;
use std::os::raw::c_ulong;

#[cfg(doc)]
use crate::pdf::document::page::annotation::{PdfPageAnnotation, PdfPageAnnotationType};

/// A single [PdfPageAnnotation] of type [PdfPageAnnotationType::Polygon].
pub struct PdfPagePolygonAnnotation<'a> {
    handle: FPDF_ANNOTATION,
    objects: PdfPageAnnotationObjects<'a>,
    attachment_points: PdfPageAnnotationAttachmentPoints<'a>,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfPagePolygonAnnotation<'a> {
    pub(crate) fn from_pdfium(
        document_handle: FPDF_DOCUMENT,
        page_handle: FPDF_PAGE,
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfPagePolygonAnnotation {
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

    /// Returns the vertices of this polygon annotation.
    ///
    /// Returns an empty vector if the annotation is not a polygon annotation or if
    /// the vertices cannot be retrieved.
    pub fn get_vertices(&self) -> Vec<PdfPoints> {
        // First, get the number of vertices by calling with a null buffer
        let vertex_count = self
            .bindings
            .FPDFAnnot_GetVertices(self.handle, std::ptr::null_mut(), 0);

        if vertex_count == 0 {
            return Vec::new();
        }

        // Allocate buffer and retrieve vertices
        let mut buffer: Vec<FS_POINTF> = vec![FS_POINTF { x: 0.0, y: 0.0 }; vertex_count as usize];

        let retrieved = self.bindings.FPDFAnnot_GetVertices(
            self.handle,
            buffer.as_mut_ptr(),
            vertex_count,
        );

        if retrieved == 0 {
            return Vec::new();
        }

        // Convert FS_POINTF to PdfPoints pairs
        // Note: This returns a flat list of x, y coordinates
        // We'll return them as pairs: [x1, y1, x2, y2, ...]
        buffer
            .into_iter()
            .take(retrieved as usize)
            .flat_map(|p| vec![PdfPoints::new(p.x), PdfPoints::new(p.y)])
            .collect()
    }

    /// Returns the vertices of this polygon annotation as coordinate pairs.
    ///
    /// Returns an empty vector if the annotation is not a polygon annotation or if
    /// the vertices cannot be retrieved.
    pub fn get_vertices_as_pairs(&self) -> Vec<(PdfPoints, PdfPoints)> {
        // First, get the number of vertices by calling with a null buffer
        let vertex_count = self
            .bindings
            .FPDFAnnot_GetVertices(self.handle, std::ptr::null_mut(), 0);

        if vertex_count == 0 {
            return Vec::new();
        }

        // Allocate buffer and retrieve vertices
        let mut buffer: Vec<FS_POINTF> = vec![FS_POINTF { x: 0.0, y: 0.0 }; vertex_count as usize];

        let retrieved = self.bindings.FPDFAnnot_GetVertices(
            self.handle,
            buffer.as_mut_ptr(),
            vertex_count,
        );

        if retrieved == 0 {
            return Vec::new();
        }

        // Convert FS_POINTF to (x, y) pairs
        buffer
            .into_iter()
            .take(retrieved as usize)
            .map(|p| (PdfPoints::new(p.x), PdfPoints::new(p.y)))
            .collect()
    }

    /// Sets the vertices of this polygon annotation.
    ///
    /// This sets the `/Vertices` dictionary entry in the annotation to a flat array
    /// `[v0.x, v0.y, v1.x, v1.y, ...]`. For polygon annotations, the path should be
    /// closed (first and last points should typically be the same, or the viewer will
    /// close it automatically). The appearance stream (`/AP`) is not automatically
    /// updated; you must rebuild it separately if needed.
    ///
    /// # Arguments
    ///
    /// * `vertices` - Slice of `(x, y)` coordinate pairs defining the polygon path
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if successful, or an error if the annotation is not a polygon
    /// annotation or if the operation fails.
    ///
    /// # Note
    ///
    /// This method only updates the dictionary entry. To also update the appearance stream,
    /// use [`set_vertices()`](Self::set_vertices) instead.
    #[cfg(feature = "pdfium_future")]
    pub fn set_vertices_geometry(
        &mut self,
        vertices: &[(f32, f32)],
    ) -> Result<(), PdfiumError> {
        if vertices.is_empty() {
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        let vertices_fs: Vec<FS_POINTF> = vertices
            .iter()
            .map(|(x, y)| FS_POINTF { x: *x, y: *y })
            .collect();

        let count = self.bindings.FPDFAnnot_SetVertices(
            self.handle,
            vertices_fs.as_ptr(),
            vertices_fs.len() as c_ulong,
        );

        if count == 0 {
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        Ok(())
    }

    /// Returns the stroke width of this polygon annotation.
    ///
    /// Returns the width from the `/BS/W` dictionary entry, or `1.0` if not set (per PDF specification default).
    #[cfg(feature = "pdfium_future")]
    pub fn stroke_width(&self) -> Result<f32, PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"🔍 PdfPagePolygonAnnotation::stroke_width() - Starting".into());
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

    /// Returns the stroke width of this polygon annotation.
    ///
    /// Returns the default value of `1.0` when the `pdfium_future` feature is not enabled.
    #[cfg(not(feature = "pdfium_future"))]
    pub fn stroke_width(&self) -> Result<f32, PdfiumError> {
        Ok(1.0)
    }

    /// Sets the stroke width of this polygon annotation.
    ///
    /// The width is stored in the `/BS/W` dictionary entry per PDF specification.
    /// If the vertices are already set, the appearance stream will be rebuilt
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
    /// - Rebuilding the appearance stream fails (if vertices are already set)
    #[cfg(feature = "pdfium_future")]
    pub fn set_stroke_width(&mut self, width: f32) -> Result<(), PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("🔧 PdfPagePolygonAnnotation::set_stroke_width() - width: {:.4}", width).into());
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

        // If vertices are already set, rebuild appearance stream with new width
        let vertices = self.get_vertices_as_pairs();
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   Vertices count: {}", vertices.len()).into());
        }
        if !vertices.is_empty() {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"   Rebuilding appearance stream with new width".into());
            }
            self.set_vertices(&vertices)?;
        }

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"✅ set_stroke_width() completed".into());
        }

        Ok(())
    }

    /// Sets the stroke width of this polygon annotation.
    ///
    /// Returns an error when the `pdfium_future` feature is not enabled.
    #[cfg(not(feature = "pdfium_future"))]
    pub fn set_stroke_width(&mut self, _width: f32) -> Result<(), PdfiumError> {
        Err(PdfiumError::PdfiumLibraryInternalError(
            PdfiumInternalError::Unknown,
        ))
    }

    /// Sets the vertices of this polygon annotation using an appearance stream.
    ///
    /// The vertices should be provided as coordinate pairs (x, y). The polygon will be
    /// drawn as a closed path with both stroke and fill (if fill color is set).
    pub fn set_vertices(&mut self, vertices: &[(PdfPoints, PdfPoints)]) -> Result<(), PdfiumError> {
        self.set_vertices_with_mode(vertices, PdfAppearanceMode::Normal)
    }

    /// Sets the vertices of this polygon annotation with a specific appearance mode.
    pub fn set_vertices_with_mode(
        &mut self,
        vertices: &[(PdfPoints, PdfPoints)],
        mode: PdfAppearanceMode,
    ) -> Result<(), PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&"🔧 PdfPagePolygonAnnotation::set_vertices_with_mode()".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&format!("   Vertices count: {}", vertices.len()).into());
            for (i, (x, y)) in vertices.iter().enumerate() {
                console::log_1(&format!("   Vertex {}: ({:.2}, {:.2})", i, x.value, y.value).into());
            }
            console::log_1(&format!("   Appearance mode: {:?}", mode).into());
        }

        if vertices.is_empty() {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"❌ ERROR: Empty vertices list".into());
            }
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        // STEP 1: Set the /Vertices dictionary entry first (source of truth)
        #[cfg(feature = "pdfium_future")]
        {
            let vertices_fs: Vec<FS_POINTF> = vertices
                .iter()
                .map(|(x, y)| FS_POINTF { x: x.value, y: y.value })
                .collect();
            
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"📝 Step 1: Setting /Vertices dictionary entry (source of truth)".into());
            }
            
            let count = self.bindings.FPDFAnnot_SetVertices(
                self.handle,
                vertices_fs.as_ptr(),
                vertices_fs.len() as c_ulong,
            );
            
            if count == 0 {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&"❌ ERROR: FPDFAnnot_SetVertices failed".into());
                }
                return Err(PdfiumError::PdfiumLibraryInternalError(
                    PdfiumInternalError::Unknown,
                ));
            }
            
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("✅ /Vertices dictionary entry set successfully ({} points)", count).into());
            }
        }

        // STEP 2: Set the annotation's Rect (required by PDFium before setting appearance stream).
        // Calculate bounding rectangle from all vertices
        let mut min_x = vertices[0].0.value;
        let mut max_x = vertices[0].0.value;
        let mut min_y = vertices[0].1.value;
        let mut max_y = vertices[0].1.value;
        
        for (x, y) in vertices.iter() {
            min_x = min_x.min(x.value);
            max_x = max_x.max(x.value);
            min_y = min_y.min(y.value);
            max_y = max_y.max(y.value);
        }
        
        // Get stroke width to add half of it as padding to prevent clipping
        let stroke_width = match self.stroke_width() {
            Ok(w) => w,
            Err(_) => 1.0, // Default stroke width
        };
        
        // Add half the stroke width on each side to prevent clipping
        // Also ensure minimum padding of 1.0 for valid dimensions
        let padding = (stroke_width / 2.0).max(1.0);
        let rect = crate::bindgen::FS_RECTF {
            left: min_x - padding,
            bottom: min_y - padding,
            right: max_x + padding,
            top: max_y + padding,
        };

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"📐 Setting annotation rect before building appearance stream".into());
            console::log_1(&format!("   Calculated rect: left={:.2}, bottom={:.2}, right={:.2}, top={:.2}",
                rect.left, rect.bottom, rect.right, rect.top).into());
        }

        let set_rect_result = self.bindings.FPDFAnnot_SetRect(self.handle, &rect);
        
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   FPDFAnnot_SetRect returned: {} (1=success, 0=failure)", set_rect_result).into());
        }

        if !self.bindings.is_true(set_rect_result) {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"❌ ERROR: FPDFAnnot_SetRect failed".into());
                console::log_1(&"═══════════════════════════════════════════════════════════".into());
            }
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        // STEP 3: Read and preserve stroke and fill colors from dictionary OR existing appearance stream
        // (FPDFAnnot_SetAP_str may clear/lock the /C and /IC dictionary entries)
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"💾 Reading stroke and fill colors from dictionaries before building appearance stream".into());
        }
        let mut preserved_r: u32 = 0;
        let mut preserved_g: u32 = 0;
        let mut preserved_b: u32 = 0;
        let mut preserved_a: u32 = 0;
        let has_existing_stroke = self.bindings.is_true(self.bindings.FPDFAnnot_GetColor(
            self.handle,
            FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color,
            &mut preserved_r,
            &mut preserved_g,
            &mut preserved_b,
            &mut preserved_a,
        ));
        let mut preserved_fr: u32 = 0;
        let mut preserved_fg: u32 = 0;
        let mut preserved_fb: u32 = 0;
        let mut preserved_fa: u32 = 0;
        let has_existing_fill = self.bindings.is_true(self.bindings.FPDFAnnot_GetColor(
            self.handle,
            FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_InteriorColor,
            &mut preserved_fr,
            &mut preserved_fg,
            &mut preserved_fb,
            &mut preserved_fa,
        ));
        
        // If dictionaries are empty, try to extract colors from existing appearance stream
        let preserved_stroke_color = if has_existing_stroke {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   ✅ Stroke color found in /C dictionary: r={}, g={}, b={}, a={}", 
                    preserved_r, preserved_g, preserved_b, preserved_a).into());
            }
            Some(PdfColor::new(preserved_r as u8, preserved_g as u8, preserved_b as u8, preserved_a as u8))
        } else {
            // Try to extract from appearance stream
            let (stroke_from_stream, _) = self.extract_colors_from_appearance_stream(mode);
            if let Some(stroke) = stroke_from_stream {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&format!("   ✅ Stroke color extracted from appearance stream: r={}, g={}, b={}, a={}", 
                        stroke.red(), stroke.green(), stroke.blue(), stroke.alpha()).into());
                }
            } else {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&"   ⚠️  No stroke color in /C dictionary or appearance stream, will use default BLACK".into());
                }
            }
            stroke_from_stream
        };
        
        let preserved_fill_color = if has_existing_fill {
            // Check if the fill color is actually transparent (alpha = 0)
            // OR if it's a default black/gray color that PDFium sets by default
            if preserved_fa == 0 {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&"   ⚠️  Fill color has alpha=0, treating as transparent".into());
                }
                None
            } else if preserved_fr == 0 && preserved_fg == 0 && preserved_fb == 0 && preserved_fa == 255 {
                // PDFium default black fill - treat as transparent
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&"   ⚠️  Fill color is PDFium default black, treating as transparent".into());
                }
                None
            } else if preserved_fr == preserved_fg && preserved_fg == preserved_fb && (preserved_fa == 128 || preserved_fa == 191 || preserved_fa == 255) {
                // PDFium default gray fill - treat as transparent
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&"   ⚠️  Fill color is PDFium default gray, treating as transparent".into());
                }
                None
            } else {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&format!("   ✅ Fill color found in /IC dictionary: r={}, g={}, b={}, a={}", 
                        preserved_fr, preserved_fg, preserved_fb, preserved_fa).into());
                }
                Some(PdfColor::new(preserved_fr as u8, preserved_fg as u8, preserved_fb as u8, preserved_fa as u8))
            }
        } else {
            // Try to extract from appearance stream
            let (_, fill_from_stream) = self.extract_colors_from_appearance_stream(mode);
            if let Some(fill) = fill_from_stream {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&format!("   ✅ Fill color extracted from appearance stream: r={}, g={}, b={}, a={}", 
                        fill.red(), fill.green(), fill.blue(), fill.alpha()).into());
                }
            } else {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&"   ⚠️  No fill color in /IC dictionary or appearance stream".into());
                }
            }
            fill_from_stream
        };
        
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"   These colors will be used in appearance stream".into());
        }

        let content_stream_result = self.build_polygon_appearance_stream_with_colors(vertices, preserved_stroke_color, preserved_fill_color);

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

        // Set the Appearance State (/AS) to match the mode
        let mode_str = match mode {
            PdfAppearanceMode::Normal => "/N",
            PdfAppearanceMode::RollOver => "/R",
            PdfAppearanceMode::Down => "/D",
        };
        let _as_result = self
            .bindings
            .FPDFAnnot_SetStringValue_str(self.handle, "AS", mode_str);

        // STEP 4: Try to restore colors after setting appearance stream
        // Note: This may fail if PDFium locks the color dictionary when appearance stream exists.
        // If it fails, the colors are already embedded in the appearance stream, which is fine.
        if let Some(stroke_color) = preserved_stroke_color {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"🔄 Attempting to restore stroke color to /C dictionary after setting appearance stream".into());
                console::log_1(&"   (This may fail - color is already in appearance stream)".into());
            }
            let _restore_result = self.bindings.FPDFAnnot_SetColor(
                self.handle,
                FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color,
                stroke_color.red() as std::os::raw::c_uint,
                stroke_color.green() as std::os::raw::c_uint,
                stroke_color.blue() as std::os::raw::c_uint,
                stroke_color.alpha() as std::os::raw::c_uint,
            );
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                if self.bindings.is_true(_restore_result) {
                    console::log_1(&format!("   ✅ Stroke color restored to dictionary: r={}, g={}, b={}, a={}", 
                        stroke_color.red(), stroke_color.green(), stroke_color.blue(), stroke_color.alpha()).into());
                } else {
                    console::log_1(&format!("   ⚠️  Stroke color restore failed (expected - color is in appearance stream: r={}, g={}, b={}, a={})", 
                        stroke_color.red(), stroke_color.green(), stroke_color.blue(), stroke_color.alpha()).into());
                }
            }
        }
        if let Some(fill_color) = preserved_fill_color {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"🔄 Attempting to restore fill color to /IC dictionary after setting appearance stream".into());
                console::log_1(&"   (This may fail - color is already in appearance stream)".into());
            }
            let _restore_result = self.bindings.FPDFAnnot_SetColor(
                self.handle,
                FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_InteriorColor,
                fill_color.red() as std::os::raw::c_uint,
                fill_color.green() as std::os::raw::c_uint,
                fill_color.blue() as std::os::raw::c_uint,
                fill_color.alpha() as std::os::raw::c_uint,
            );
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                if self.bindings.is_true(_restore_result) {
                    console::log_1(&format!("   ✅ Fill color restored to dictionary: r={}, g={}, b={}, a={}", 
                        fill_color.red(), fill_color.green(), fill_color.blue(), fill_color.alpha()).into());
                } else {
                    console::log_1(&format!("   ⚠️  Fill color restore failed (expected - color is in appearance stream: r={}, g={}, b={}, a={})", 
                        fill_color.red(), fill_color.green(), fill_color.blue(), fill_color.alpha()).into());
                }
            }
        }

        if self.bindings.is_true(result) {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"✅ set_vertices_with_mode completed successfully".into());
                console::log_1(&"═══════════════════════════════════════════════════════════".into());
            }
            Ok(())
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"❌ ERROR: FPDFAnnot_SetAP_str returned false".into());
                console::log_1(&"═══════════════════════════════════════════════════════════".into());
            }
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Extracts stroke and fill colors from existing appearance stream by parsing RG and rg commands.
    /// Returns (stroke_color, fill_color) tuple, where each can be None if not found.
    fn extract_colors_from_appearance_stream(&self, mode: PdfAppearanceMode) -> (Option<PdfColor>, Option<PdfColor>) {
        // Get the appearance stream content
        let buffer_length = self.bindings.FPDFAnnot_GetAP(
            self.handle,
            mode.as_pdfium(),
            std::ptr::null_mut(),
            0,
        );

        if buffer_length == 0 {
            return (None, None);
        }

        let mut buffer = vec![0u16; (buffer_length / 2 + 1) as usize];
        let result = self.bindings.FPDFAnnot_GetAP(
            self.handle,
            mode.as_pdfium(),
            buffer.as_mut_ptr() as *mut crate::bindgen::FPDF_WCHAR,
            buffer_length,
        );

        if result == 0 {
            return (None, None);
        }

        // Convert UTF-16LE to String
        let stream_content = String::from_utf16_lossy(&buffer[..((result / 2) as usize).saturating_sub(1)]);
        
        use std::str::FromStr;
        
        // Extract stroke color from RG command: "r g b RG"
        let stroke_color = if let Some(rg_pos) = stream_content.find(" RG") {
            let before_rg = &stream_content[..rg_pos];
            let parts: Vec<&str> = before_rg.split_whitespace().collect();
            
            if parts.len() >= 3 {
                let r_str = parts[parts.len().saturating_sub(3)];
                let g_str = parts[parts.len().saturating_sub(2)];
                let b_str = parts[parts.len().saturating_sub(1)];
                
                if let (Ok(r_val), Ok(g_val), Ok(b_val)) = (
                    f64::from_str(r_str),
                    f64::from_str(g_str),
                    f64::from_str(b_str),
                ) {
                    let r = (r_val * 255.0).clamp(0.0, 255.0) as u8;
                    let g = (g_val * 255.0).clamp(0.0, 255.0) as u8;
                    let b = (b_val * 255.0).clamp(0.0, 255.0) as u8;
                    let a = 255u8;
                    Some(PdfColor::new(r, g, b, a))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // Extract fill color from rg command: "r g b rg"
        let fill_color = if let Some(rg_pos) = stream_content.find(" rg") {
            let before_rg = &stream_content[..rg_pos];
            let parts: Vec<&str> = before_rg.split_whitespace().collect();
            
            if parts.len() >= 3 {
                let r_str = parts[parts.len().saturating_sub(3)];
                let g_str = parts[parts.len().saturating_sub(2)];
                let b_str = parts[parts.len().saturating_sub(1)];
                
                if let (Ok(r_val), Ok(g_val), Ok(b_val)) = (
                    f64::from_str(r_str),
                    f64::from_str(g_str),
                    f64::from_str(b_str),
                ) {
                    let r = (r_val * 255.0).clamp(0.0, 255.0) as u8;
                    let g = (g_val * 255.0).clamp(0.0, 255.0) as u8;
                    let b = (b_val * 255.0).clamp(0.0, 255.0) as u8;
                    let a = 255u8;
                    Some(PdfColor::new(r, g, b, a))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        (stroke_color, fill_color)
    }

    /// Builds the PDF content stream string for drawing the polygon.
    fn build_polygon_appearance_stream(
        &self,
        vertices: &[(PdfPoints, PdfPoints)],
    ) -> Result<String, PdfiumError> {
        self.build_polygon_appearance_stream_with_colors(vertices, None, None)
    }

    /// Builds the PDF content stream string for drawing the polygon, optionally using preserved colors.
    fn build_polygon_appearance_stream_with_colors(
        &self,
        vertices: &[(PdfPoints, PdfPoints)],
        preserved_stroke_color: Option<PdfColor>,
        preserved_fill_color: Option<PdfColor>,
    ) -> Result<String, PdfiumError> {
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

        // Get stroke color - use preserved color if provided, otherwise read from dictionary
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            if preserved_stroke_color.is_some() || preserved_fill_color.is_some() {
                console::log_1(&"🎨 build_polygon_appearance_stream() - Using preserved colors".into());
            } else {
                console::log_1(&"🎨 build_polygon_appearance_stream() - Reading colors from dictionaries".into());
            }
        }
        
        let stroke_color = if let Some(preserved) = preserved_stroke_color {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   ✅ Using preserved stroke color: r={}, g={}, b={}, a={}", 
                    preserved.red(), preserved.green(), preserved.blue(), preserved.alpha()).into());
            }
            preserved
        } else {
            // Try to read from dictionary
            let mut r: u32 = 0;
            let mut g: u32 = 0;
            let mut b: u32 = 0;
            let mut a: u32 = 0;
            
            let get_stroke_color_result = self.bindings.FPDFAnnot_GetColor(
                self.handle,
                FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_Color,
                &mut r,
                &mut g,
                &mut b,
                &mut a,
            );
            
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   FPDFAnnot_GetColor(/C) returned: {} (1=success, 0=failure)", get_stroke_color_result).into());
                console::log_1(&format!("   Stroke color values read from /C dictionary: r={}, g={}, b={}, a={}", r, g, b, a).into());
            }
            
            if self.bindings.is_true(get_stroke_color_result) {
                let color = PdfColor::new(r as u8, g as u8, b as u8, a as u8);
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&format!("   ✅ Using stroke color from /C dictionary: {:?}", color).into());
                }
                color
            } else {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&"   ⚠️  No stroke color in /C dictionary, using default BLACK".into());
                }
                PdfColor::BLACK
            }
        };

        // Get fill color - use preserved color if provided, otherwise read from dictionary
        let fill_color = if let Some(preserved) = preserved_fill_color {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   ✅ Using preserved fill color: r={}, g={}, b={}, a={}", 
                    preserved.red(), preserved.green(), preserved.blue(), preserved.alpha()).into());
            }
            Some(preserved)
        } else {
            // Try to read from dictionary
            let mut fr: u32 = 0;
            let mut fg: u32 = 0;
            let mut fb: u32 = 0;
            let mut fa: u32 = 0;
            
            let get_fill_color_result = self.bindings.FPDFAnnot_GetColor(
                self.handle,
                FPDFANNOT_COLORTYPE_FPDFANNOT_COLORTYPE_InteriorColor,
                &mut fr,
                &mut fg,
                &mut fb,
                &mut fa,
            );

            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   FPDFAnnot_GetColor(/IC) returned: {} (1=success, 0=failure)", get_fill_color_result).into());
                console::log_1(&format!("   Fill color values read from /IC dictionary: r={}, g={}, b={}, a={}", fr, fg, fb, fa).into());
            }

            if self.bindings.is_true(get_fill_color_result) {
                // Check if the fill color is actually transparent (alpha = 0)
                // OR if it's a default black/gray color that PDFium sets by default
                if fa == 0 {
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&"   ⚠️  Fill color has alpha=0, treating as transparent".into());
                    }
                    None
                } else if fr == 0 && fg == 0 && fb == 0 && fa == 255 {
                    // PDFium default black fill - treat as transparent
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&"   ⚠️  Fill color is PDFium default black, treating as transparent".into());
                    }
                    None
                } else if fr == fg && fg == fb && (fa == 128 || fa == 191 || fa == 255) {
                    // PDFium default gray fill - treat as transparent
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&"   ⚠️  Fill color is PDFium default gray, treating as transparent".into());
                    }
                    None
                } else {
                    let color = PdfColor::new(fr as u8, fg as u8, fb as u8, fa as u8);
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&format!("   ✅ Using fill color from /IC dictionary: {:?}", color).into());
                    }
                    Some(color)
                }
            } else {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&"   ⚠️  No fill color in /IC dictionary".into());
                }
                None
            }
        };
        
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   Final stroke color to apply: r={}, g={}, b={}, a={}", 
                stroke_color.red(), stroke_color.green(), stroke_color.blue(), stroke_color.alpha()).into());
            if let Some(fill) = &fill_color {
                console::log_1(&format!("   Final fill color to apply: r={}, g={}, b={}, a={}", 
                    fill.red(), fill.green(), fill.blue(), fill.alpha()).into());
            } else {
                console::log_1(&"   No fill color to apply".into());
            }
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
        }

        // Get line width from /BS/W dictionary or default to 1.0
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"📏 build_polygon_appearance_stream() - Getting stroke width".into());
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
            Err(e) => {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&format!("   ⚠️  Error: {:?}, using default 1.0", e).into());
                }
                1.0
            }
        };

        // Get border style from /BS/S dictionary (default to "S" for solid)
        let border_style_str = {
            #[cfg(feature = "pdfium_future")]
            {
                let mut border_style_buffer = vec![0u8; 16];
                let style_len = self.bindings.FPDFAnnot_GetBSStyle(
                    self.handle,
                    border_style_buffer.as_mut_ptr() as *mut std::os::raw::c_char,
                    border_style_buffer.len() as std::os::raw::c_ulong,
                );
                if style_len > 0 && style_len <= border_style_buffer.len() as std::os::raw::c_ulong {
                    let style_bytes = &border_style_buffer[..(style_len as usize - 1)]; // Exclude null terminator
                    match std::str::from_utf8(style_bytes) {
                        Ok(s) => s.to_string(),
                        Err(_) => "S".to_string()
                    }
                } else {
                    "S".to_string()
                }
            }
            #[cfg(not(feature = "pdfium_future"))]
            {
                "S".to_string() // Default to solid when feature not enabled
            }
        };

        // Get dash pattern from /BS/D dictionary if style is "D" (dashed)
        let (dash_length, gap_length, dash_phase) = if border_style_str == "D" {
            #[cfg(feature = "pdfium_future")]
            {
                let mut dash: f32 = 3.0;
                let mut gap: f32 = 3.0;
                let mut phase: f32 = 0.0;
                let dash_result = self.bindings.FPDFAnnot_GetBSDash(
                    self.handle,
                    &mut dash,
                    &mut gap,
                    &mut phase,
                );
                if self.bindings.is_true(dash_result) {
                    (dash, gap, phase)
                } else {
                    (3.0, 3.0, 0.0) // Default dash pattern
                }
            }
            #[cfg(not(feature = "pdfium_future"))]
            {
                (3.0, 3.0, 0.0) // Default dash pattern when feature not enabled
            }
        } else {
            (0.0, 0.0, 0.0) // Not dashed, no dash pattern
        };

        let mut stream = String::new();

        // Save graphics state
        stream.push_str("q\n");

        // Translate coordinate system to annotation's bottom-left corner
        stream.push_str(&format!("1 0 0 1 {:.4} {:.4} cm\n", left, bottom));

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

        // Set dash pattern if border style is "D" (dashed)
        if border_style_str == "D" && dash_length > 0.0 {
            stream.push_str(&format!("[{:.4} {:.4}] {:.4} d\n", dash_length, gap_length, dash_phase));
        }

        // Build the polygon path
        // CRITICAL: After translating to the rect's bottom-left, coordinates must be
        // relative to that translation. Convert from absolute page coordinates to
        // coordinates relative to the annotation rect.
        if let Some((first_x, first_y)) = vertices.first() {
            // Move to first vertex (relative to annotation rect)
            let first_x_rel = first_x.value - left;
            let first_y_rel = first_y.value - bottom;
            stream.push_str(&format!("{:.4} {:.4} m\n", first_x_rel, first_y_rel));

            // Line to each subsequent vertex (relative to annotation rect)
            for (x, y) in vertices.iter().skip(1) {
                let x_rel = x.value - left;
                let y_rel = y.value - bottom;
                stream.push_str(&format!("{:.4} {:.4} l\n", x_rel, y_rel));
            }

            // Close the path
            stream.push_str("h\n");

            // Fill and/or stroke based on fill color presence and transparency
            // Default behavior: transparent fill (stroke only) when no fill color is set or alpha is 0
            if fill_color.is_some() && fill_color.as_ref().map(|c| c.alpha() > 0).unwrap_or(false) {
                stream.push_str("B\n"); // Fill and stroke
            } else {
                stream.push_str("S\n"); // Stroke only (transparent fill)
            }
        }

        // Restore graphics state
        stream.push_str("Q\n");

        Ok(stream)
    }
}

impl<'a> PdfPageAnnotationPrivate<'a> for PdfPagePolygonAnnotation<'a> {
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

