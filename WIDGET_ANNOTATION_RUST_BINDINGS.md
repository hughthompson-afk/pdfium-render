# Adding Widget Annotation Creation Functions to pdfium-render Rust Bindings

This guide explains how to integrate the newly implemented `FPDF_EnsureAcroForm()` and `FPDFPage_CreateWidgetAnnot()` functions into your Rust bindings for pdfium-render.

## Function Signatures

### C API Signatures

```c
// Ensures the document has an /AcroForm dictionary in its catalog.
// Creates one if it doesn't exist.
FPDF_BOOL FPDF_EnsureAcroForm(FPDF_DOCUMENT document);

// Create a widget annotation (form field annotation) in |page|.
// This creates both the form field dictionary and the widget annotation,
// properly linking them together.
FPDF_ANNOTATION FPDFPage_CreateWidgetAnnot(
    FPDF_PAGE page,
    FPDF_FORMHANDLE form_handle,
    FPDF_BYTESTRING field_name,
    FPDF_BYTESTRING field_type,
    const FS_RECTF* rect);
```

## Step 1: Update FFI Bindings

### 1.1 Add to Bindings Trait

In your `src/bindings.rs` (or equivalent), add the function declarations to the `PdfiumLibraryBindings` trait:

```rust
/// Ensures the document has an /AcroForm dictionary in its catalog.
/// Creates one if it doesn't exist.
///
///   `document` - Handle to the document.
///
/// Returns `true` if AcroForm exists or was successfully created, `false` otherwise.
#[cfg(feature = "pdfium_future")]
#[allow(non_snake_case)]
fn FPDF_EnsureAcroForm(&self, document: FPDF_DOCUMENT) -> FPDF_BOOL;

/// Create a widget annotation (form field annotation) in |page|.
/// This creates both the form field dictionary and the widget annotation,
/// properly linking them together.
///
///   `page`        - Handle to the page.
///   `form_handle` - Handle to the form fill module (required).
///   `field_name`  - The field name (/T key), encoded in UTF-8.
///   `field_type`  - The field type (/FT key): "Tx" (text), "Btn" (button),
///                   "Ch" (choice), or "Sig" (signature).
///   `rect`        - Bounding rectangle for the widget annotation.
///
/// Returns a handle to the created widget annotation, or NULL on failure.
/// Must call FPDFPage_CloseAnnot() when done.
#[cfg(feature = "pdfium_future")]
#[allow(non_snake_case)]
fn FPDFPage_CreateWidgetAnnot(
    &self,
    page: FPDF_PAGE,
    form_handle: FPDF_FORMHANDLE,
    field_name: FPDF_BYTESTRING,
    field_type: FPDF_BYTESTRING,
    rect: *const FS_RECTF,
) -> FPDF_ANNOTATION;
```

### 1.2 Implement in Binding Files

#### Static Bindings (`src/bindings/static_bindings.rs`)

```rust
#[cfg(feature = "pdfium_future")]
impl PdfiumLibraryBindings for PdfiumLibraryBindingsStatic {
    // ... existing implementations ...

    fn FPDF_EnsureAcroForm(&self, document: FPDF_DOCUMENT) -> FPDF_BOOL {
        unsafe { ffi::FPDF_EnsureAcroForm(document) }
    }

    fn FPDFPage_CreateWidgetAnnot(
        &self,
        page: FPDF_PAGE,
        form_handle: FPDF_FORMHANDLE,
        field_name: FPDF_BYTESTRING,
        field_type: FPDF_BYTESTRING,
        rect: *const FS_RECTF,
    ) -> FPDF_ANNOTATION {
        unsafe {
            ffi::FPDFPage_CreateWidgetAnnot(page, form_handle, field_name, field_type, rect)
        }
    }
}
```

#### Dynamic Bindings (`src/bindings/dynamic.rs`)

```rust
#[cfg(feature = "pdfium_future")]
impl PdfiumLibraryBindings for PdfiumLibraryBindingsDynamic {
    // ... existing implementations ...

    fn FPDF_EnsureAcroForm(&self, document: FPDF_DOCUMENT) -> FPDF_BOOL {
        let func: unsafe extern "C" fn(FPDF_DOCUMENT) -> FPDF_BOOL =
            self.get_function("FPDF_EnsureAcroForm")?;
        unsafe { func(document) }
    }

    fn FPDFPage_CreateWidgetAnnot(
        &self,
        page: FPDF_PAGE,
        form_handle: FPDF_FORMHANDLE,
        field_name: FPDF_BYTESTRING,
        field_type: FPDF_BYTESTRING,
        rect: *const FS_RECTF,
    ) -> FPDF_ANNOTATION {
        let func: unsafe extern "C" fn(
            FPDF_PAGE,
            FPDF_FORMHANDLE,
            FPDF_BYTESTRING,
            FPDF_BYTESTRING,
            *const FS_RECTF,
        ) -> FPDF_ANNOTATION = self.get_function("FPDFPage_CreateWidgetAnnot")?;
        unsafe { func(page, form_handle, field_name, field_type, rect) }
    }
}
```

#### WASM Bindings (`src/bindings/wasm.rs`)

```rust
#[cfg(feature = "pdfium_future")]
impl PdfiumLibraryBindings for PdfiumLibraryBindingsWasm {
    // ... existing implementations ...

    fn FPDF_EnsureAcroForm(&self, document: FPDF_DOCUMENT) -> FPDF_BOOL {
        // WASM-specific implementation
        // Use wasm-bindgen or similar to call the function
        unsafe {
            // Example: wasm_bindgen::closure::Closure::wrap(...)
            // Adjust based on your WASM binding approach
        }
    }

    fn FPDFPage_CreateWidgetAnnot(
        &self,
        page: FPDF_PAGE,
        form_handle: FPDF_FORMHANDLE,
        field_name: FPDF_BYTESTRING,
        field_type: FPDF_BYTESTRING,
        rect: *const FS_RECTF,
    ) -> FPDF_ANNOTATION {
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
    pub fn FPDF_EnsureAcroForm(document: FPDF_DOCUMENT) -> FPDF_BOOL;

    #[cfg(feature = "pdfium_future")]
    pub fn FPDFPage_CreateWidgetAnnot(
        page: FPDF_PAGE,
        form_handle: FPDF_FORMHANDLE,
        field_name: FPDF_BYTESTRING,
        field_type: FPDF_BYTESTRING,
        rect: *const FS_RECTF,
    ) -> FPDF_ANNOTATION;
}
```

## Step 2: Update High-Level Rust API

### 2.1 For Document (`src/pdf/document.rs`)

Add the `ensure_acro_form()` method to your document implementation:

```rust
impl PdfDocument {
    /// Ensures the document has an /AcroForm dictionary in its catalog.
    /// Creates one if it doesn't exist.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if AcroForm exists or was successfully created,
    /// or an error if the operation fails.
    #[cfg(feature = "pdfium_future")]
    pub fn ensure_acro_form(&self) -> Result<(), PdfiumError> {
        if !self.bindings.is_true(
            self.bindings.FPDF_EnsureAcroForm(self.document_handle())
        ) {
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }
        Ok(())
    }
}
```

### 2.2 For Widget Annotations (`src/pdf/document/page/annotation/widget.rs`)

Create a new widget annotation module or add to existing annotation creation:

```rust
use crate::prelude::*;

/// Field types for widget annotations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfWidgetFieldType {
    /// Text field
    Text,
    /// Button field (checkbox, radio button, or push button)
    Button,
    /// Choice field (listbox or combobox)
    Choice,
    /// Signature field
    Signature,
}

impl PdfWidgetFieldType {
    fn as_str(&self) -> &'static str {
        match self {
            PdfWidgetFieldType::Text => "Tx",
            PdfWidgetFieldType::Button => "Btn",
            PdfWidgetFieldType::Choice => "Ch",
            PdfWidgetFieldType::Signature => "Sig",
        }
    }
}

impl PdfPage {
    /// Creates a widget annotation (form field annotation) on this page.
    ///
    /// This creates both the form field dictionary and the widget annotation,
    /// properly linking them together. The widget annotation represents an
    /// interactive form field (text field, checkbox, radio button, etc.).
    ///
    /// # Arguments
    ///
    /// * `form_handle` - Handle to the form fill module (required).
    /// * `field_name` - The field name (e.g., "MyTextField").
    /// * `field_type` - The type of form field to create.
    /// * `rect` - Bounding rectangle for the widget annotation.
    ///
    /// # Returns
    ///
    /// Returns `Ok(PdfWidgetAnnotation)` if successful, or an error if the
    /// operation fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pdfium_render::prelude::*;
    ///
    /// let pdfium = Pdfium::new();
    /// let mut document = pdfium.create_new_pdf().unwrap();
    /// document.ensure_acro_form().unwrap();
    ///
    /// let mut page = document.pages_mut().create_page_at_start(
    ///     PdfPagePaperSize::a4().width,
    ///     PdfPagePaperSize::a4().height,
    /// ).unwrap();
    ///
    /// let form_handle = document.form_handle().unwrap();
    /// let rect = PdfRect::new(100.0, 700.0, 200.0, 720.0);
    ///
    /// let widget = page.annotations_mut().create_widget_annotation(
    ///     form_handle,
    ///     "MyTextField",
    ///     PdfWidgetFieldType::Text,
    ///     rect,
    /// ).unwrap();
    /// ```
    #[cfg(feature = "pdfium_future")]
    pub fn create_widget_annotation(
        &mut self,
        form_handle: FPDF_FORMHANDLE,
        field_name: &str,
        field_type: PdfWidgetFieldType,
        rect: PdfRect,
    ) -> Result<PdfWidgetAnnotation, PdfiumError> {
        // Ensure AcroForm exists
        self.document().ensure_acro_form()?;

        // Convert field name to C string
        let field_name_cstr = CString::new(field_name)
            .map_err(|_| PdfiumError::InvalidArgument)?;
        
        let field_type_str = field_type.as_str();
        let field_type_cstr = CString::new(field_type_str)
            .map_err(|_| PdfiumError::InvalidArgument)?;

        // Convert rect to FS_RECTF
        let fs_rect = FS_RECTF {
            left: rect.left.value,
            top: rect.top.value,
            right: rect.right.value,
            bottom: rect.bottom.value,
        };

        // Create the widget annotation
        let annot_handle = self.bindings().FPDFPage_CreateWidgetAnnot(
            self.page_handle(),
            form_handle,
            field_name_cstr.as_ptr(),
            field_type_cstr.as_ptr(),
            &fs_rect,
        );

        if annot_handle.is_null() {
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        // Wrap in PdfWidgetAnnotation type
        // (Adjust based on your annotation type structure)
        Ok(PdfWidgetAnnotation::from_handle(
            annot_handle,
            self.bindings(),
        ))
    }
}
```

### 2.3 For Annotations Collection (`src/pdf/document/page/annotation/mod.rs` or similar)

Add a convenience method to the annotations collection:

```rust
impl PdfPageAnnotations {
    /// Creates a widget annotation (form field annotation) on the page.
    ///
    /// # Arguments
    ///
    /// * `form_handle` - Handle to the form fill module.
    /// * `field_name` - The field name.
    /// * `field_type` - The type of form field.
    /// * `rect` - Bounding rectangle.
    ///
    /// # Returns
    ///
    /// Returns the created widget annotation.
    #[cfg(feature = "pdfium_future")]
    pub fn create_widget_annotation(
        &mut self,
        form_handle: FPDF_FORMHANDLE,
        field_name: &str,
        field_type: PdfWidgetFieldType,
        rect: PdfRect,
    ) -> Result<PdfWidgetAnnotation, PdfiumError> {
        self.page_mut().create_widget_annotation(
            form_handle,
            field_name,
            field_type,
            rect,
        )
    }
}
```

## Step 3: Type Definitions

Ensure you have the necessary type definitions. If not already present, add:

```rust
// Field type constants (if using string-based approach instead of enum)
pub const PDF_FIELD_TYPE_TEXT: &str = "Tx";
pub const PDF_FIELD_TYPE_BUTTON: &str = "Btn";
pub const PDF_FIELD_TYPE_CHOICE: &str = "Ch";
pub const PDF_FIELD_TYPE_SIGNATURE: &str = "Sig";
```

## Step 4: Testing

Add tests to verify the functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_acro_form() {
        let pdfium = Pdfium::new();
        let mut document = pdfium.create_new_pdf().unwrap();
        
        // Initially, document should not have AcroForm
        // After calling ensure_acro_form, it should exist
        assert!(document.ensure_acro_form().is_ok());
        
        // Calling again should still succeed
        assert!(document.ensure_acro_form().is_ok());
    }

    #[test]
    fn test_create_widget_annotation_text_field() {
        let pdfium = Pdfium::new();
        let mut document = pdfium.create_new_pdf().unwrap();
        document.ensure_acro_form().unwrap();
        
        let mut page = document.pages_mut().create_page_at_start(
            PdfPagePaperSize::a4().width,
            PdfPagePaperSize::a4().height,
        ).unwrap();

        let form_handle = document.form_handle().unwrap();
        let rect = PdfRect::new(100.0, 700.0, 200.0, 720.0);
        
        let widget = page.annotations_mut().create_widget_annotation(
            form_handle,
            "MyTextField",
            PdfWidgetFieldType::Text,
            rect,
        ).unwrap();

        // Verify it's a widget annotation
        assert_eq!(widget.subtype(), PdfAnnotationSubtype::Widget);
        
        // Verify form field type
        // (Adjust based on your form field API)
        // assert_eq!(widget.form_field_type(), Some(PdfFormFieldType::Text));
    }

    #[test]
    fn test_create_widget_annotation_checkbox() {
        let pdfium = Pdfium::new();
        let mut document = pdfium.create_new_pdf().unwrap();
        document.ensure_acro_form().unwrap();
        
        let mut page = document.pages_mut().create_page_at_start(
            PdfPagePaperSize::a4().width,
            PdfPagePaperSize::a4().height,
        ).unwrap();

        let form_handle = document.form_handle().unwrap();
        let rect = PdfRect::new(100.0, 700.0, 120.0, 720.0);
        
        let widget = page.annotations_mut().create_widget_annotation(
            form_handle,
            "MyCheckbox",
            PdfWidgetFieldType::Button,
            rect,
        ).unwrap();

        assert_eq!(widget.subtype(), PdfAnnotationSubtype::Widget);
    }
}
```

## Step 5: Documentation

Update your crate documentation to mention the new functions:

```rust
//! # Widget Annotation Creation
//!
//! The following functions allow you to create widget annotations (form fields):
//!
//! - [`PdfDocument::ensure_acro_form()`] - Ensures the document has an /AcroForm dictionary
//! - [`PdfPage::create_widget_annotation()`] - Creates a widget annotation linked to a form field
//!
//! Widget annotations represent interactive form fields such as:
//! - Text fields (`PdfWidgetFieldType::Text`)
//! - Checkboxes and radio buttons (`PdfWidgetFieldType::Button`)
//! - List boxes and combo boxes (`PdfWidgetFieldType::Choice`)
//! - Signature fields (`PdfWidgetFieldType::Signature`)
//!
//! **Note:** A form fill environment must be initialized before creating widget annotations.
//! Use `FPDFDOC_InitFormFillEnvironment()` or your library's equivalent to create a form handle.
```

## Important Notes

1. **Form Fill Environment Required**: Widget annotations require a form fill environment (`FPDF_FORMHANDLE`). Make sure to initialize the form fill environment before creating widget annotations.

2. **Field Type Validation**: The C API validates field types. Only "Tx", "Btn", "Ch", and "Sig" are accepted. Invalid types will return `nullptr`.

3. **AcroForm Creation**: `FPDF_EnsureAcroForm()` should be called before creating widget annotations, though `FPDFPage_CreateWidgetAnnot()` will create it automatically if needed.

4. **Memory Management**: Widget annotations created with `FPDFPage_CreateWidgetAnnot()` must be closed with `FPDFPage_CloseAnnot()` when done, just like other annotations.

5. **Form Field Configuration**: After creating a widget annotation, you can configure it using existing form field APIs:
   - `FPDFAnnot_SetFormFieldFlags()` - Set field flags
   - `FPDFAnnot_SetStringValue()` - Set field value (for text fields)
   - `FPDFAnnot_SetRect()` - Adjust rectangle

## Example Usage

```rust
use pdfium_render::prelude::*;

// Create a new PDF document
let pdfium = Pdfium::new();
let mut document = pdfium.create_new_pdf().unwrap();

// Ensure AcroForm exists
document.ensure_acro_form().unwrap();

// Create a page
let mut page = document.pages_mut().create_page_at_start(
    PdfPagePaperSize::a4().width,
    PdfPagePaperSize::a4().height,
).unwrap();

// Get form fill handle (assuming your library provides this)
let form_handle = document.form_handle().unwrap();

// Create a text field widget
let text_field_rect = PdfRect::new(100.0, 700.0, 200.0, 720.0);
let mut text_widget = page.annotations_mut().create_widget_annotation(
    form_handle,
    "MyTextField",
    PdfWidgetFieldType::Text,
    text_field_rect,
).unwrap();

// Create a checkbox widget
let checkbox_rect = PdfRect::new(100.0, 650.0, 120.0, 670.0);
let mut checkbox_widget = page.annotations_mut().create_widget_annotation(
    form_handle,
    "MyCheckbox",
    PdfWidgetFieldType::Button,
    checkbox_rect,
).unwrap();

// Configure the text field (using existing APIs)
// text_widget.set_form_field_flags(...);
// text_widget.set_value("Default text");
```

## Checklist

- [ ] Add function declarations to `PdfiumLibraryBindings` trait
- [ ] Implement in static bindings
- [ ] Implement in dynamic bindings
- [ ] Implement in WASM bindings (if applicable)
- [ ] Update FFI definitions
- [ ] Add high-level API method `ensure_acro_form()` to `PdfDocument`
- [ ] Add high-level API method `create_widget_annotation()` to `PdfPage`
- [ ] Add convenience method to annotations collection
- [ ] Add `PdfWidgetFieldType` enum or constants
- [ ] Add tests
- [ ] Update documentation
- [ ] Verify cross-platform compatibility

