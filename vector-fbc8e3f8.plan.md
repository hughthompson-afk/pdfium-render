<!-- fbc8e3f8-3261-4344-8376-7dd6c537a524 87b380a9-0c96-4029-b039-f3faa88d1897 -->
# Vector Path Signature Appearance Implementation Plan

## Background and Purpose

This implementation adds support for setting the **visual appearance** of PDF signature fields using vector path data (similar to SVG paths). This is specifically designed for digital signature workflows where:

1. **The visual representation** (handwritten signature strokes) is handled by this API
2. **The cryptographic signature** (PKCS#7/CAdES data, ByteRange, etc.) is handled separately by external signing infrastructure

**Important**: This API sets ONLY the appearance stream (`/AP`) of the signature field. It does NOT:

- Create or modify the signature dictionary (`/V`)
- Handle cryptographic signing operations
- Validate or invalidate existing signatures

The visual appearance and cryptographic signature are independent in PDF - you can set a visual representation before or after cryptographic signing without affecting signature validity.

---

## Files to Create

### 1. New Module: `src/pdf/document/page/annotation/signature_appearance.rs`

This module contains types and builders specifically for signature visual appearances.

#### 1.1 Path Segment Types

````rust
//! Defines types and builders for setting the visual appearance of signature fields
//! using vector path data.
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
//! ```rust
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

/// A 2D point for signature path drawing, in PDF user space coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignaturePathPoint {
    pub x: f32,
    pub y: f32,
}

impl SignaturePathPoint {
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
````

#### 1.2 Signature Stroke Definition

```rust
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
        self.segments.push(SignaturePathSegment::MoveTo(
            SignaturePathPoint::new(x, y)
        ));
        self
    }

    /// Draws a straight line to the given point.
    pub fn line_to(mut self, x: f32, y: f32) -> Self {
        self.segments.push(SignaturePathSegment::LineTo(
            SignaturePathPoint::new(x, y)
        ));
        self
    }

    /// Draws a cubic Bezier curve to the given endpoint.
    ///
    /// # Arguments
    /// * `cx1`, `cy1` - First control point
    /// * `cx2`, `cy2` - Second control point  
    /// * `x`, `y` - End point of the curve
    pub fn curve_to(
        mut self,
        cx1: f32, cy1: f32,
        cx2: f32, cy2: f32,
        x: f32, y: f32,
    ) -> Self {
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
```

#### 1.3 Signature Appearance Builder

```rust
/// Builder for constructing and applying the visual appearance of a signature field.
///
/// This builder collects signature strokes and generates a PDF content stream
/// that renders them as vector paths. The resulting appearance is purely visual
/// and does not affect cryptographic signature validity.
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
pub struct SignatureAppearanceBuilder<'a> {
    bindings: &'a dyn PdfiumLibraryBindings,
    annotation_handle: FPDF_ANNOTATION,
    strokes: Vec<SignatureStroke>,
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
        }
    }

    /// Adds a stroke to the signature appearance.
    pub fn add_stroke(&mut self, stroke: SignatureStroke) -> &mut Self {
        self.strokes.push(stroke);
        self
    }

    /// Adds multiple strokes to the signature appearance.
    pub fn add_strokes(
        &mut self, 
        strokes: impl IntoIterator<Item = SignatureStroke>
    ) -> &mut Self {
        self.strokes.extend(strokes);
        self
    }

    /// Clears all strokes from the builder.
    pub fn clear(&mut self) -> &mut Self {
        self.strokes.clear();
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

    /// Applies the signature appearance with a specific appearance mode.
    ///
    /// Most signatures only need the Normal appearance. RollOver and Down
    /// appearances are used for interactive hover/click states.
    pub fn apply_with_mode(&self, mode: PdfAppearanceMode) -> Result<(), PdfiumError> {
        let content_stream = self.build_content_stream();

        if self.bindings.is_true(
            self.bindings.FPDFAnnot_SetAP_str(
                self.annotation_handle,
                mode.as_pdfium(),
                &content_stream,
            )
        ) {
            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Builds the PDF content stream string from all strokes.
    fn build_content_stream(&self) -> String {
        let mut stream = String::with_capacity(self.strokes.len() * 100);

        // Save graphics state
        stream.push_str("q\n");
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
                    SignaturePathSegment::CurveTo { control1, control2, end } => {
                        stream.push_str(&format!(
                            "{:.4} {:.4} {:.4} {:.4} {:.4} {:.4} c\n",
                            control1.x, control1.y,
                            control2.x, control2.y,
                            end.x, end.y,
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
}
```

---

## Files to Modify

### 2. Update `src/pdf/document/page/annotation.rs`

Add module declaration near the top with other annotation modules:

```rust
pub mod signature_appearance;  // Add this line
```

### 3. Update `src/pdf/document/page/field/signature.rs`

This is the key change - adding the dedicated signature appearance method.

Add imports at the top:

```rust
use crate::pdf::document::page::annotation::signature_appearance::SignatureAppearanceBuilder;
```

Add new method to `impl<'a> PdfFormSignatureField<'a>`:

````rust
/// Returns a builder for setting the visual appearance of this signature field.
///
/// # Visual vs Cryptographic Signatures
///
/// PDF signatures have two independent components:
///
/// 1. **Visual appearance** (this API): The graphical representation shown on the
///    page - typically handwritten strokes, an image, or text like "Signed by..."
///
/// 2. **Cryptographic signature** (NOT this API): The digital signature data that
///    cryptographically validates the document integrity and signer identity.
///
/// This method sets ONLY the visual appearance. The cryptographic signature must
/// be created separately using appropriate signing infrastructure.
///
/// # Coordinate System
///
/// Stroke coordinates are relative to the signature field's bounding box:
/// - Origin (0, 0) is at the bottom-left corner of the field
/// - The field dimensions can be obtained from the parent widget annotation's bounds
///
/// # Example
///
/// ```rust
/// use pdfium_render::prelude::*;
///
/// // Signature strokes captured from a signature pad or touch input
/// let stroke1 = SignatureStroke::new()
///     .with_stroke_width(1.2)
///     .with_color(PdfColor::new(0, 0, 100, 255)) // Dark blue ink
///     .move_to(5.0, 25.0)
///     .curve_to(10.0, 35.0, 20.0, 35.0, 30.0, 25.0)
///     .curve_to(35.0, 20.0, 40.0, 10.0, 50.0, 15.0);
///
/// let stroke2 = SignatureStroke::new()
///     .with_stroke_width(1.2)
///     .with_color(PdfColor::new(0, 0, 100, 255))
///     .move_to(55.0, 20.0)
///     .line_to(70.0, 20.0);
///
/// // Apply the visual signature
/// signature_field.set_signature_appearance()
///     .add_stroke(stroke1)
///     .add_stroke(stroke2)
///     .apply()?;
///
/// // The cryptographic signature would be applied separately
/// // using your signing infrastructure
/// ```
pub fn set_signature_appearance(&self) -> SignatureAppearanceBuilder<'a> {
    SignatureAppearanceBuilder::new(
        self.annotation_handle,
        self.bindings,
    )
}
````

### 4. Update `src/pdf/document/page/annotation/widget.rs` (Optional but recommended)

Also expose the builder on the widget annotation for flexibility:

Add import:

```rust
use crate::pdf::document::page::annotation::signature_appearance::SignatureAppearanceBuilder;
```

Add method:

```rust
/// Returns a builder for setting the visual appearance of this widget annotation.
///
/// For signature fields, prefer using [PdfFormSignatureField::set_signature_appearance()]
/// which provides better discoverability and documentation for the signature use case.
///
/// This method is useful when you need to set appearances on widget annotations
/// that are not signature fields, or when working with the annotation directly.
pub fn appearance_builder(&self) -> SignatureAppearanceBuilder<'a> {
    SignatureAppearanceBuilder::new(
        self.annotation_handle,
        self.bindings,
    )
}
```

### 5. Update `src/lib.rs` prelude

Add the new types to the prelude exports (around line 115-135):

```rust
pdf::document::page::annotation::signature_appearance::*,
```

---

## Optional Enhancement: SVG Path Parser

For convenience when working with SVG-based signature capture:

````rust
impl SignatureAppearanceBuilder<'_> {
    /// Parses an SVG path data string and adds it as a stroke.
    ///
    /// Supports the following SVG path commands:
    /// - `M`/`m`: MoveTo (absolute/relative)
    /// - `L`/`l`: LineTo (absolute/relative)
    /// - `C`/`c`: Cubic Bezier CurveTo (absolute/relative)
    /// - `Z`/`z`: ClosePath
    ///
    /// # Example
    ///
    /// ```rust
    /// builder.add_svg_path(
    ///     "M 10 20 C 15 30 25 30 30 20 L 50 5",
    ///     1.5,
    ///     PdfColor::BLACK
    /// );
    /// ```
    pub fn add_svg_path(
        &mut self,
        svg_d: &str,
        stroke_width: f32,
        color: PdfColor,
    ) -> Result<&mut Self, PdfiumError> {
        let stroke = Self::parse_svg_path(svg_d, stroke_width, color)?;
        self.strokes.push(stroke);
        Ok(self)
    }

    fn parse_svg_path(
        svg_d: &str,
        stroke_width: f32,
        color: PdfColor
    ) -> Result<SignatureStroke, PdfiumError> {
        // Implementation: tokenize svg_d and convert to SignaturePathSegments
        // Handle both absolute (M, L, C, Z) and relative (m, l, c, z) commands
        // Track current position for relative commands
        // Return error for unsupported commands
        todo!()
    }
}
````

---

## Summary of Changes

| File | Change |

|------|--------|

| `src/pdf/document/page/annotation/signature_appearance.rs` | **NEW** - Core types and builder |

| `src/pdf/document/page/annotation.rs` | Add `pub mod signature_appearance;` |

| `src/pdf/document/page/field/signature.rs` | Add `set_signature_appearance()` method |

| `src/pdf/document/page/annotation/widget.rs` | Add `appearance_builder()` method (optional) |

| `src/lib.rs` | Export new types in prelude |

---

## Testing Requirements

1. **Unit test**: Verify `build_content_stream()` generates correct PDF operator syntax
2. **Unit test**: Verify stroke builder chain methods work correctly
3. **Integration test**: Load PDF with signature field, set appearance, save, reload, verify appearance persists
4. **Integration test**: Verify setting appearance does not affect existing form field properties

### To-dos

- [ ] Create new file src/pdf/document/page/annotation/appearance.rs
- [ ] Define PdfPathPoint and PdfPathSegment enum with MoveTo/LineTo/CurveTo/Close
- [ ] Define PdfAppearanceStroke struct with builder methods
- [ ] Define PdfAppearanceStreamBuilder struct with add_stroke and apply methods
- [ ] Implement build_content_stream() to generate PDF path operators
- [ ] Add pub mod appearance to src/pdf/document/page/annotation.rs
- [ ] Add appearance_builder() method to PdfPageWidgetAnnotation
- [ ] Export new types from src/lib.rs prelude
- [ ] Add unit and integration tests for appearance stream generation