# Fork Changes: pdfium-render

This document details the significant changes, new features, and improvements made to this fork of `pdfium-render` since commit `a378b78df3f6f64114ddc8eac66c87658853e29e`.

## Overview

The primary focus of this fork has been to expand the capabilities of `pdfium-render` in several key areas:
1.  **Interactive Form Fields (Widget Annotations)**: Full support for creating and managing form fields (CRUD operations).
2.  **Advanced Annotation Support**: Implementation of many previously unsupported annotation types (Line, Polygon, Polyline, Circle, Ink, etc.).
3.  **Digital Signature Visuals**: A dedicated system for creating visual appearances for signature fields using vector paths.
4.  **WASM & Internal Improvements**: Robustness fixes for WASM environments and updated bindings for modern PDFium versions.

---

## 1. Interactive Form Fields (Widget Annotations)

This fork introduces comprehensive support for interactive form fields, allowing programmatic creation of PDF forms from scratch.

### Key Features:
-   **Field Creation**: Support for creating 7 types of form fields with advanced initial configuration.
    ```rust
    pub fn create_widget_annotation(
        &mut self,
        form_handle: FPDF_FORMHANDLE,
        field_name: &str,
        field_type: PdfFormFieldType,
        rect: PdfRect,
        options: Option<&[&str]>,
        max_length: Option<i32>,
        quadding: Option<i32>,
        default_appearance: Option<&str>,
        default_value: Option<&str>,
        additional_flags: Option<u32>,
    ) -> Result<PdfPageWidgetAnnotation, PdfiumError>
    ```
-   **CRUD Operations**: Full support for adding, reading, updating, and deleting widget annotations and their underlying form fields.
-   **Automatic Appearance Generation**: Text fields automatically generate their visual appearance streams (`/AP`) upon creation, ensuring they render correctly across all PDF viewers and flatten properly.
-   **AcroForm Management**: Added `document.ensure_acro_form()` to handle the necessary PDF dictionary setup for form-enabled documents.

### Peculiarities & Usage:
-   **Form Fill Environment**: For new documents, you **must** initialize the form fill environment using `document.init_form_fill_environment()`. This is required for widget annotations to function correctly.
    ```rust
    let pdfium = Pdfium::default();
    let mut document = pdfium.create_new_pdf()?;
    document.ensure_acro_form()?;
    let form_handle = document.init_form_fill_environment()?;
    ```
-   **Idempotency**: `init_form_fill_environment()` is idempotent and manages memory safely, particularly important in WASM environments.
-   **Read-Only Limitations**: Currently, PDFium's C API only provides read-only access to ComboBox and ListBox options. Adding options programmatically requires low-level dictionary manipulation (not yet exposed in the high-level API).

---

## 2. Advanced Annotation Support

Previously, many annotation types were read-only or unsupported. This fork adds creation and editing capabilities for several geometric and markup annotations.

### New Annotation Types:
-   **Line**: Set starting and ending points via `set_line()`.
    ```rust
    pub fn set_line(
        &mut self,
        start: PdfLinePoint,
        end: PdfLinePoint,
    ) -> Result<(), PdfiumError>
    ```
-   **Polygon & Polyline**: Set multiple vertices via `set_vertices()`.
    ```rust
    pub fn set_vertices(
        &mut self,
        vertices: &[(PdfPoints, PdfPoints)],
    ) -> Result<(), PdfiumError>
    ```
-   **Circle (Square/Ellipse)**: Support for creating and styling circular/elliptical annotations.
-   **Ink**: Support for "hand-drawn" paths with multiple strokes.
    ```rust
    // Add a stroke to an existing ink annotation
    pub fn add_ink_stroke(
        &mut self, 
        points: &[PdfInkStrokePoint]
    ) -> Result<usize, PdfiumError>

    // Remove all strokes
    pub fn remove_ink_list(&mut self) -> Result<(), PdfiumError>
    ```
-   **Watermark**: Support for page-level watermark annotations.
-   **Caret & Text Markup**: Improved handling and flattening for Highlight, Underline, Strikeout, and Squiggly annotations.
-   **Highlight Transparency Fix**: Addressed a widespread issue where highlight annotations would appear opaque or with incorrect transparency when flattened. The library now automatically injects an `ExtGState` with a fixed `0.3` opacity (`/ca` and `/CA`) into the appearance stream, matching standard PDF viewer behavior.

### Technical Detail:
-   Geometry setters (like `set_line` or `set_vertices`) update the underlying PDF dictionary entries (`/L`, `/Vertices`). Users may need to trigger an appearance rebuild if the visual representation doesn't update automatically in their viewer.

---

## 3. Digital Signature Visual Appearances

A major addition is the `SignatureAppearanceBuilder`, designed specifically for digital signature workflows.

### The Problem Solved:
In PDF, the *visual representation* (the ink on the page) and the *cryptographic signature* (the PKCS#7 data) are independent. This fork provides a high-level API to build the visual "ink" part using vector paths, which is much cleaner and more scalable than using images.

### Features:
-   **Vector Strokes**: Define signatures using `MoveTo`, `LineTo`, and `CurveTo` (Cubic Bezier) operators.
    ```rust
    pub fn move_to(mut self, x: f32, y: f32) -> Self
    pub fn line_to(mut self, x: f32, y: f32) -> Self
    pub fn curve_to(mut self, cx1: f32, cy1: f32, cx2: f32, cy2: f32, x: f32, y: f32) -> Self
    ```
-   **Styling**: Customize stroke width and color (e.g., "Dark Blue" ink).
    ```rust
    pub fn with_stroke_width(mut self, width: f32) -> Self
    pub fn with_color(mut self, color: PdfColor) -> Self
    ```
-   **Independent of Crypto**: Can be applied before or after cryptographic signing without invalidating the signature.

### Example Usage:
```rust
signature_field.set_signature_appearance()
    .add_stroke(SignatureStroke::new()
        .with_stroke_width(1.5)
        .with_color(PdfColor::new(0, 0, 80, 255))
        .move_to(10.0, 20.0)
        .curve_to(15.0, 30.0, 25.0, 30.0, 30.0, 20.0))
    .apply()?;
```

---

## 4. WASM & Platform Improvements

Significant effort was put into making the library more robust in WebAssembly (WASM) environments.

-   **Memory Management**: Fixed issues where the form fill environment would leak memory or cause crashes in WASM due to improper lifecycle management.
-   **Dynamic & Static Bindings**: Updated both dynamic and static binding implementations to support the new `pdfium_future` functions.
-   **Feature Flags**: Introduced more granular feature flags (e.g., `pdfium_7543`, `pdfium_future`) to allow users to opt-in to the latest PDFium capabilities while maintaining compatibility.

---

## 5. Flattening & Rendering

Improved the `flatten()` implementation for:
-   **Widget Annotations**: Forms can now be "burned" into the page content effectively.
-   **Text Markup**: Annotations like highlights and underlines now flatten with much higher fidelity.
-   **Highlight Transparency Fix**: Addressed a widespread issue where highlight annotations would appear opaque or with incorrect transparency when flattened. The library now automatically injects an `ExtGState` with a fixed `0.3` opacity (`/ca` and `/CA`) into the appearance stream, matching standard PDF viewer behavior.
-   **Stamp Annotations**: Fixed issues with stamp annotation appearance streams not being generated correctly in some edge cases.

---

### Page Object & Helper Improvements:
-   **Circle Path Objects**: Added `create_path_object_circle_at()` for easy creation of circular path objects with stroke and fill.
-   **Markup Helpers**: Added convenience methods like `create_squiggly_annotation_under_object()`, `create_strikeout_annotation_through_object()`, and `create_highlight_annotation_over_object()`.
-   **Appearance Stream Debugging**: New `debug_appearance_streams()` method on the annotations collection to help developers inspect the internal PDF content streams of any annotation.

## 6. Bug Fixes & Stability

-   **Path Points Fix**: Resolved a critical issue where path points for certain annotations were not being written correctly, sometimes leading to memory corruption or "garbage" data in the PDF.
-   **Image Extract improvements**: Refined how images are extracted from page objects to be more reliable.
-   **WASM Form Handle**: Fixed a common "Form handle not available" error in WASM by providing a more structured initialization flow.
## Summary of New API Methods

| Area | Method | Description |
| :--- | :--- | :--- |
| **Document** | `ensure_acro_form()` | Prepares document for form fields. |
| **Document** | `init_form_fill_environment()` | Initializes form fill logic (required for widgets). |
| **Page** | `create_widget_annotation()` | Creates a new form field widget. |
| **Signature Field** | `set_signature_appearance()` | Entry point for the vector signature builder. |
| **Geometric Annots** | `set_line_geometry()`, `set_vertices_geometry()` | Sets the geometric data for Line/Poly annotations. |
| **Ink Annotations** | `add_ink_stroke()` | Adds a stroke path to an ink annotation. |

## Important Considerations

1.  **Coordinate System**: All coordinates follow the standard PDF coordinate system (origin at bottom-left).
2.  **Feature Flag**: Many of these features require the `pdfium_future` feature flag to be enabled in your `Cargo.toml`.
3.  **PDFium Version**: These changes are best utilized with a recent version of PDFium (Build 6000+).

---

