# Adding FPDFAnnot_SetLine() and FPDFAnnot_SetVertices() to pdfium-render Rust Bindings

This guide explains how to integrate the newly implemented `FPDFAnnot_SetLine()` and `FPDFAnnot_SetVertices()` functions into your Rust bindings for pdfium-render.

## Function Signatures

### C API Signatures

```c
// Sets the /L dictionary entry for line annotations
FPDF_BOOL FPDFAnnot_SetLine(
    FPDF_ANNOTATION annot,
    const FS_POINTF* start,
    const FS_POINTF* end
);

// Sets the /Vertices dictionary entry for polyline/polygon annotations
unsigned long FPDFAnnot_SetVertices(
    FPDF_ANNOTATION annot,
    const FS_POINTF* vertices,
    unsigned long count
);
```

## Step 1: Update FFI Bindings

### 1.1 Add to Bindings Trait

In your `src/bindings.rs` (or equivalent), add the function declarations to the `PdfiumLibraryBindings` trait:

```rust
/// Sets the starting and ending coordinates of a line annotation.
///
/// Sets the /L dictionary entry to [start.x, start.y, end.x, end.y].
///
///   `annot` - handle to an annotation.
///   `start` - starting point (must not be NULL).
///   `end`   - ending point (must not be NULL).
///
/// Returns `true` if successful.
#[cfg(feature = "pdfium_future")]
#[allow(non_snake_case)]
fn FPDFAnnot_SetLine(
    &self,
    annot: FPDF_ANNOTATION,
    start: *const FS_POINTF,
    end: *const FS_POINTF,
) -> FPDF_BOOL;

/// Sets the vertices of a polyline or polygon annotation.
///
/// Sets the /Vertices dictionary entry to [v0.x, v0.y, v1.x, v1.y, ...].
///
///   `annot`    - handle to an annotation.
///   `vertices` - array of points (must not be NULL).
///   `count`    - number of points in the array (must be > 0).
///
/// Returns the number of points set if successful, 0 otherwise.
#[cfg(feature = "pdfium_future")]
#[allow(non_snake_case)]
fn FPDFAnnot_SetVertices(
    &self,
    annot: FPDF_ANNOTATION,
    vertices: *const FS_POINTF,
    count: c_ulong,
) -> c_ulong;
```

### 1.2 Implement in Binding Files

#### Static Bindings (`src/bindings/static_bindings.rs`)

```rust
#[cfg(feature = "pdfium_future")]
impl PdfiumLibraryBindings for PdfiumLibraryBindingsStatic {
    // ... existing implementations ...

    fn FPDFAnnot_SetLine(
        &self,
        annot: FPDF_ANNOTATION,
        start: *const FS_POINTF,
        end: *const FS_POINTF,
    ) -> FPDF_BOOL {
        unsafe { ffi::FPDFAnnot_SetLine(annot, start, end) }
    }

    fn FPDFAnnot_SetVertices(
        &self,
        annot: FPDF_ANNOTATION,
        vertices: *const FS_POINTF,
        count: c_ulong,
    ) -> c_ulong {
        unsafe { ffi::FPDFAnnot_SetVertices(annot, vertices, count) }
    }
}
```

#### Dynamic Bindings (`src/bindings/dynamic.rs`)

```rust
#[cfg(feature = "pdfium_future")]
impl PdfiumLibraryBindings for PdfiumLibraryBindingsDynamic {
    // ... existing implementations ...

    fn FPDFAnnot_SetLine(
        &self,
        annot: FPDF_ANNOTATION,
        start: *const FS_POINTF,
        end: *const FS_POINTF,
    ) -> FPDF_BOOL {
        let func: unsafe extern "C" fn(
            FPDF_ANNOTATION,
            *const FS_POINTF,
            *const FS_POINTF,
        ) -> FPDF_BOOL = self.get_function("FPDFAnnot_SetLine")?;
        unsafe { func(annot, start, end) }
    }

    fn FPDFAnnot_SetVertices(
        &self,
        annot: FPDF_ANNOTATION,
        vertices: *const FS_POINTF,
        count: c_ulong,
    ) -> c_ulong {
        let func: unsafe extern "C" fn(
            FPDF_ANNOTATION,
            *const FS_POINTF,
            c_ulong,
        ) -> c_ulong = self.get_function("FPDFAnnot_SetVertices")?;
        unsafe { func(annot, vertices, count) }
    }
}
```

#### WASM Bindings (`src/bindings/wasm.rs`)

```rust
#[cfg(feature = "pdfium_future")]
impl PdfiumLibraryBindings for PdfiumLibraryBindingsWasm {
    // ... existing implementations ...

    fn FPDFAnnot_SetLine(
        &self,
        annot: FPDF_ANNOTATION,
        start: *const FS_POINTF,
        end: *const FS_POINTF,
    ) -> FPDF_BOOL {
        // WASM-specific implementation
        // Use wasm-bindgen or similar to call the function
        unsafe {
            // Example: wasm_bindgen::closure::Closure::wrap(...)
            // Adjust based on your WASM binding approach
        }
    }

    fn FPDFAnnot_SetVertices(
        &self,
        annot: FPDF_ANNOTATION,
        vertices: *const FS_POINTF,
        count: c_ulong,
    ) -> c_ulong {
        // WASM-specific implementation
        unsafe {
            // Example: wasm_bindgen::closure::Closure::wrap(...)
            // Adjust based on your WASM binding approach
        }
    }
}
```

### 1.3 Update FFI Definitions

In your FFI definitions file (e.g., `src/ffi.rs` or generated bindings), ensure the functions are declared:

```rust
#[link(name = "pdfium")]
extern "C" {
    #[cfg(feature = "pdfium_future")]
    pub fn FPDFAnnot_SetLine(
        annot: FPDF_ANNOTATION,
        start: *const FS_POINTF,
        end: *const FS_POINTF,
    ) -> FPDF_BOOL;

    #[cfg(feature = "pdfium_future")]
    pub fn FPDFAnnot_SetVertices(
        annot: FPDF_ANNOTATION,
        vertices: *const FS_POINTF,
        count: c_ulong,
    ) -> c_ulong;
}
```

## Step 2: Update High-Level Rust API

### 2.1 For Line Annotations (`src/pdf/document/page/annotation/line.rs`)

Add the `set_line()` method to your line annotation implementation:

```rust
impl PdfLineAnnotation {
    /// Sets the starting and ending coordinates of this line annotation.
    ///
    /// This sets the `/L` dictionary entry in the annotation to `[start.x, start.y, end.x, end.y]`.
    /// The appearance stream (`/AP`) is not automatically updated; you must rebuild it separately
    /// if needed.
    ///
    /// # Arguments
    ///
    /// * `start` - The starting point of the line
    /// * `end` - The ending point of the line
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if successful, or an error if the annotation is not a line annotation
    /// or if the operation fails.
    pub fn set_line(
        &mut self,
        start: PdfPoints,
        end: PdfPoints,
    ) -> Result<(), PdfiumError> {
        let start_fs = FS_POINTF {
            x: start.x.value,
            y: start.y.value,
        };
        let end_fs = FS_POINTF {
            x: end.x.value,
            y: end.y.value,
        };

        if !self.bindings.is_true(
            self.bindings.FPDFAnnot_SetLine(self.handle, &start_fs, &end_fs)
        ) {
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        Ok(())
    }

    /// Sets the line coordinates using the existing `set_line_with_mode()` pattern if you have one.
    pub fn set_line_with_mode(
        &mut self,
        start: PdfPoints,
        end: PdfPoints,
        mode: PdfAnnotationRenderMode,
    ) -> Result<(), PdfiumError> {
        // Set the /L dictionary entry first
        self.set_line(start, end)?;
        
        // Then update the rect (if needed)
        let rect = PdfRect::new(
            start.x.min(end.x),
            start.y.min(end.y),
            start.x.max(end.x),
            start.y.max(end.y),
        );
        self.set_rect(rect)?;
        
        // Rebuild appearance stream if needed
        // (implementation depends on your existing pattern)
        
        Ok(())
    }
}
```

### 2.2 For Polyline/Polygon Annotations

#### Polyline (`src/pdf/document/page/annotation/polyline.rs`)

```rust
impl PdfPolylineAnnotation {
    /// Sets the vertices of this polyline annotation.
    ///
    /// This sets the `/Vertices` dictionary entry in the annotation to a flat array
    /// `[v0.x, v0.y, v1.x, v1.y, ...]` where `v0`, `v1`, etc. are the points in the
    /// vertices slice. The appearance stream (`/AP`) is not automatically updated;
    /// you must rebuild it separately if needed.
    ///
    /// # Arguments
    ///
    /// * `vertices` - Slice of `(x, y)` coordinate pairs defining the polyline path
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if successful, or an error if the annotation is not a polyline
    /// annotation or if the operation fails.
    pub fn set_vertices(
        &mut self,
        vertices: &[(f32, f32)],
    ) -> Result<(), PdfiumError> {
        if vertices.is_empty() {
            return Err(PdfiumError::InvalidArgument);
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

    /// Sets vertices using PdfPoints if that's your coordinate type.
    pub fn set_vertices_from_points(
        &mut self,
        vertices: &[PdfPoints],
    ) -> Result<(), PdfiumError> {
        let coords: Vec<(f32, f32)> = vertices
            .iter()
            .map(|pt| (pt.x.value, pt.y.value))
            .collect();
        self.set_vertices(&coords)
    }
}
```

#### Polygon (`src/pdf/document/page/annotation/polygon.rs`)

```rust
impl PdfPolygonAnnotation {
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
    pub fn set_vertices(
        &mut self,
        vertices: &[(f32, f32)],
    ) -> Result<(), PdfiumError> {
        if vertices.is_empty() {
            return Err(PdfiumError::InvalidArgument);
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
}
```

## Step 3: Update Type Definitions

Ensure you have the necessary type definitions. If not already present, add:

```rust
#[repr(C)]
pub struct FS_POINTF {
    pub x: f32,
    pub y: f32,
}

pub type FPDF_ANNOTATION = *mut std::ffi::c_void;
pub type FPDF_BOOL = std::os::raw::c_int;
```

## Step 4: Testing

Add tests to verify the functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_line() {
        // Create a document and page
        let pdfium = Pdfium::new();
        let mut document = pdfium.create_new_pdf().unwrap();
        let mut page = document.pages_mut().create_page_at_start(
            PdfPagePaperSize::a4().width,
            PdfPagePaperSize::a4().height,
        ).unwrap();

        // Create a line annotation
        let mut annot = page.annotations_mut().create_line_annotation().unwrap();
        
        // Set line coordinates
        let start = PdfPoints::new(100.0, 200.0);
        let end = PdfPoints::new(300.0, 400.0);
        annot.set_line(start, end).unwrap();

        // Verify by reading back
        let retrieved = annot.line().unwrap();
        assert_eq!(retrieved.start.x.value, 100.0);
        assert_eq!(retrieved.start.y.value, 200.0);
        assert_eq!(retrieved.end.x.value, 300.0);
        assert_eq!(retrieved.end.y.value, 400.0);
    }

    #[test]
    fn test_set_vertices() {
        // Create a document and page
        let pdfium = Pdfium::new();
        let mut document = pdfium.create_new_pdf().unwrap();
        let mut page = document.pages_mut().create_page_at_start(
            PdfPagePaperSize::a4().width,
            PdfPagePaperSize::a4().height,
        ).unwrap();

        // Create a polygon annotation
        let mut annot = page.annotations_mut().create_polygon_annotation().unwrap();
        
        // Set vertices
        let vertices = vec![
            (100.0, 200.0),
            (300.0, 400.0),
            (500.0, 600.0),
        ];
        annot.set_vertices(&vertices).unwrap();

        // Verify by reading back
        let retrieved = annot.vertices().unwrap();
        assert_eq!(retrieved.len(), 3);
        assert_eq!(retrieved[0].x, 100.0);
        assert_eq!(retrieved[0].y, 200.0);
    }
}
```

## Step 5: Documentation

Update your crate documentation to mention the new functions:

```rust
//! # Annotation Geometry Setters
//!
//! The following functions allow you to set annotation geometry in the PDF dictionary:
//!
//! - [`PdfLineAnnotation::set_line()`] - Sets the `/L` dictionary entry for line annotations
//! - [`PdfPolylineAnnotation::set_vertices()`] - Sets the `/Vertices` dictionary entry for polyline annotations
//! - [`PdfPolygonAnnotation::set_vertices()`] - Sets the `/Vertices` dictionary entry for polygon annotations
//!
//! **Note:** These functions only update the dictionary entries. The appearance stream (`/AP`)
//! is not automatically updated. You may need to rebuild the appearance stream separately if
//! you want the visual representation to match the dictionary data.
```

## Important Notes

1. **Appearance Streams**: These functions only set dictionary entries. The appearance stream (`/AP`) is NOT automatically updated. If you need the visual representation to match, you'll need to rebuild the appearance stream separately.

2. **Coordinate System**: Ensure you're using the correct coordinate system (PDF default user space units).

3. **Validation**: The C API validates that:
   - For `SetLine`: The annotation must be of type `FPDF_ANNOT_LINE`
   - For `SetVertices`: The annotation must be of type `FPDF_ANNOT_POLYGON` or `FPDF_ANNOT_POLYLINE`
   - Pointers must not be null
   - For `SetVertices`: `count` must be > 0

4. **Error Handling**: Both functions return 0/false on failure. Make sure to check the return value and handle errors appropriately.

5. **Memory Safety**: The `vertices` array in `SetVertices` must remain valid for the duration of the function call. Using a `Vec<FS_POINTF>` is safe as it ensures the data stays in memory.

## Example Usage

```rust
use pdfium_render::prelude::*;

// Create a line annotation
let mut line_annot = page.annotations_mut().create_line_annotation().unwrap();
line_annot.set_line(
    PdfPoints::new(100.0, 200.0),  // start
    PdfPoints::new(300.0, 400.0),  // end
).unwrap();

// Create a polygon annotation
let mut polygon_annot = page.annotations_mut().create_polygon_annotation().unwrap();
polygon_annot.set_vertices(&[
    (100.0, 200.0),
    (300.0, 400.0),
    (500.0, 600.0),
    (100.0, 200.0),  // Close the polygon
]).unwrap();
```

## Checklist

- [ ] Add function declarations to `PdfiumLibraryBindings` trait
- [ ] Implement in static bindings
- [ ] Implement in dynamic bindings
- [ ] Implement in WASM bindings (if applicable)
- [ ] Update FFI definitions
- [ ] Add high-level API methods for line annotations
- [ ] Add high-level API methods for polyline annotations
- [ ] Add high-level API methods for polygon annotations
- [ ] Add tests
- [ ] Update documentation
- [ ] Verify cross-platform compatibility

