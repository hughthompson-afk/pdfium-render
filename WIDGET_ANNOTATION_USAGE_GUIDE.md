# Widget Annotation Creation Usage Guide

This guide explains how to use the new widget annotation creation functions in `pdfium-render` to create interactive form fields in PDF documents.

## Overview

Widget annotations represent interactive form fields in PDF documents. The new functions allow you to create:
- Text fields
- Push buttons
- Checkboxes
- Radio buttons
- Combo boxes (dropdowns)
- List boxes
- Signature fields

## Prerequisites

1. **Enable the feature flag**: Make sure you have `pdfium_future` feature enabled in your `Cargo.toml`:
   ```toml
   [dependencies]
   pdfium-render = { version = "0.8", features = ["pdfium_future"] }
   ```

2. **Form fill environment**: Widget annotations require a form fill environment to be initialized. This typically happens automatically when you load a document that contains forms, but for new documents you may need to ensure the form environment exists.

## Basic Workflow

### Step 1: Create or Load a Document

```rust
use pdfium_render::prelude::*;

let pdfium = Pdfium::new();
let mut document = pdfium.create_new_pdf()?;
// OR load an existing document:
// let mut document = pdfium.load_pdf_from_file("existing.pdf", None)?;
```

### Step 2: Ensure AcroForm Exists

Before creating widget annotations, ensure the document has an AcroForm dictionary:

```rust
document.ensure_acro_form()?;
```

This creates the `/AcroForm` dictionary in the document catalog if it doesn't already exist. This is required for all form fields.

### Step 3: Create a Page

```rust
let mut page = document.pages_mut().create_page_at_start(
    PdfPagePaperSize::a4().width,
    PdfPagePaperSize::a4().height,
)?;
```

### Step 4: Initialize Form Fill Environment and Get Form Handle

**IMPORTANT**: For new documents, you must initialize the form fill environment before creating widget annotations. The form fill environment is required for all widget annotation operations.

**Recommended Approach**: Use the built-in `init_form_fill_environment()` method on `PdfDocument`:

```rust
// After ensure_acro_form()
document.ensure_acro_form()?;

// Initialize form fill environment (clean API, handles memory management)
let form_handle = document.init_form_fill_environment()?;

// Now you can create widget annotations
let widget = page.annotations_mut().create_widget_annotation(
    form_handle,
    "MyField",
    PdfFormFieldType::Text,
    PdfRect::new(100.0, 700.0, 300.0, 720.0),
)?;
```

**For existing documents with forms**, you can get the form handle from the document:

```rust
let form_handle = document.form()
    .map(|f| f.handle())
    .ok_or("Form not initialized")?;
```

**Note**: The `init_form_fill_environment()` method:
- Automatically handles memory management for the form fill environment
- Returns the existing form handle if already initialized (idempotent)
- Properly stores the form fill info structure to prevent memory leaks
- Works for both new and existing documents

### Step 5: Create Widget Annotations

Now you can create widget annotations using either method:

**Method 1: Through the annotations collection**
```rust
let widget = page.annotations_mut().create_widget_annotation(
    form_handle,
    "MyFieldName",
    PdfFormFieldType::Text,
    PdfRect::new(100.0, 700.0, 200.0, 720.0),
)?;
```

**Method 2: Through the page (convenience method)**
```rust
let widget = page.create_widget_annotation(
    form_handle,
    "MyFieldName",
    PdfFormFieldType::Text,
    PdfRect::new(100.0, 700.0, 200.0, 720.0),
)?;
```

## Form Field Types

### 1. Text Field

Creates a single-line or multi-line text input field.

```rust
let text_field = page.annotations_mut().create_widget_annotation(
    form_handle,
    "UserName",
    PdfFormFieldType::Text,
    PdfRect::new(100.0, 700.0, 300.0, 720.0), // left, bottom, right, top
)?;

// Access the form field to set properties
if let Some(field) = text_field.form_field() {
    if let Some(text_field) = field.as_text_field() {
        // Set default value, make it required, etc.
        // text_field.set_value("Default text")?;
    }
}
```

### 2. Push Button

Creates a clickable button that doesn't retain a value.

```rust
let push_button = page.annotations_mut().create_widget_annotation(
    form_handle,
    "SubmitButton",
    PdfFormFieldType::PushButton,
    PdfRect::new(100.0, 650.0, 200.0, 680.0),
)?;
```

### 3. Checkbox

Creates a checkbox that can be checked or unchecked.

```rust
let checkbox = page.annotations_mut().create_widget_annotation(
    form_handle,
    "AgreeToTerms",
    PdfFormFieldType::Checkbox,
    PdfRect::new(100.0, 600.0, 120.0, 620.0), // Square checkbox
)?;

// Access and configure the checkbox
if let Some(field) = checkbox.form_field() {
    if let Some(checkbox_field) = field.as_checkbox_field() {
        // checkbox_field.set_checked(true)?;
    }
}
```

### 4. Radio Button

Creates a radio button. Radio buttons with the same name form a group where only one can be selected.

```rust
// Create multiple radio buttons with the same name to form a group
let radio1 = page.annotations_mut().create_widget_annotation(
    form_handle,
    "Gender", // Same name = same group
    PdfFormFieldType::RadioButton,
    PdfRect::new(100.0, 550.0, 120.0, 570.0),
)?;

let radio2 = page.annotations_mut().create_widget_annotation(
    form_handle,
    "Gender", // Same name = same group
    PdfFormFieldType::RadioButton,
    PdfRect::new(150.0, 550.0, 170.0, 570.0),
)?;

// Access and configure radio buttons
if let Some(field) = radio1.form_field() {
    if let Some(radio_field) = field.as_radio_button_field() {
        // radio_field.set_checked(true)?;
    }
}
```

### 5. Combo Box (Dropdown)

Creates a dropdown list that may or may not allow text input.

```rust
let combo_box = page.annotations_mut().create_widget_annotation(
    form_handle,
    "Country",
    PdfFormFieldType::ComboBox,
    PdfRect::new(100.0, 500.0, 300.0, 520.0),
)?;

// Access and configure the combo box
if let Some(field) = combo_box.form_field() {
    if let Some(combo_field) = field.as_combo_box_field() {
        // Note: Options cannot be added programmatically yet.
        // The pdfium-render library currently only provides read-only access to options
        // via combo_field.options(). To add options, you would need to:
        // 1. Manipulate the PDF dictionary directly (complex, low-level)
        // 2. Wait for the API to be implemented in pdfium-render
        // 3. Create the form field with options already defined in the PDF
        
        // You can read existing options:
        // for option in combo_field.options().iter() {
        //     println!("Option: {:?}", option.label());
        // }
    }
}
```

**Limitation**: Currently, there is no API to add or modify options for combo boxes and list boxes after creation. PDFium only provides read-only functions for options (`FPDFAnnot_GetOptionCount`, `FPDFAnnot_GetOptionLabel`). To work around this, you would need to manipulate the PDF dictionary directly, which is complex and not recommended unless you're familiar with PDF internals.

### 6. List Box

Creates a scrollable list where one or more items can be selected.

```rust
let list_box = page.annotations_mut().create_widget_annotation(
    form_handle,
    "Languages",
    PdfFormFieldType::ListBox,
    PdfRect::new(100.0, 400.0, 300.0, 500.0), // Taller for list
)?;

// Access and configure the list box
if let Some(field) = list_box.form_field() {
    if let Some(list_field) = field.as_list_box_field() {
        // Note: Options cannot be added programmatically yet.
        // The pdfium-render library currently only provides read-only access to options
        // via list_field.options(). To add options, you would need to:
        // 1. Manipulate the PDF dictionary directly (complex, low-level)
        // 2. Wait for the API to be implemented in pdfium-render
        // 3. Create the form field with options already defined in the PDF
        
        // You can read existing options:
        // for option in list_field.options().iter() {
        //     println!("Option: {:?}", option.label());
        // }
    }
}
```

**Limitation**: Currently, there is no API to add or modify options for combo boxes and list boxes after creation. PDFium only provides read-only functions for options. See the Combo Box section above for more details.

### 7. Signature Field

Creates a signature field for digital signatures.

```rust
let signature_field = page.annotations_mut().create_widget_annotation(
    form_handle,
    "UserSignature",
    PdfFormFieldType::Signature,
    PdfRect::new(100.0, 300.0, 400.0, 400.0), // Larger area for signature
)?;

// Access and configure the signature field
if let Some(field) = signature_field.form_field() {
    if let Some(sig_field) = field.as_signature_field() {
        // Configure signature appearance, etc.
        // sig_field.set_signature_appearance(...)?;
    }
}
```

## Complete Example

Here's a complete example creating a form with multiple field types:

```rust
use pdfium_render::prelude::*;

fn create_sample_form() -> Result<(), PdfiumError> {
    // Create a new PDF document
    let pdfium = Pdfium::new();
    let mut document = pdfium.create_new_pdf()?;
    
    // Ensure AcroForm exists
    document.ensure_acro_form()?;
    
    // Initialize form fill environment (REQUIRED for new documents)
    // This method handles all the complexity internally
    let form_handle = document.init_form_fill_environment()?;
    
    // Create a page
    let mut page = document.pages_mut().create_page_at_start(
        PdfPagePaperSize::a4().width,
        PdfPagePaperSize::a4().height,
    )?;
    
    // Create a text field for name
    let name_field = page.annotations_mut().create_widget_annotation(
        form_handle,
        "FullName",
        PdfFormFieldType::Text,
        PdfRect::new(100.0, 750.0, 400.0, 770.0),
    )?;
    
    // Create a text field for email
    let email_field = page.annotations_mut().create_widget_annotation(
        form_handle,
        "Email",
        PdfFormFieldType::Text,
        PdfRect::new(100.0, 720.0, 400.0, 740.0),
    )?;
    
    // Create a checkbox
    let newsletter_checkbox = page.annotations_mut().create_widget_annotation(
        form_handle,
        "SubscribeNewsletter",
        PdfFormFieldType::Checkbox,
        PdfRect::new(100.0, 690.0, 120.0, 710.0),
    )?;
    
    // Create radio buttons for gender (same name = same group)
    let male_radio = page.annotations_mut().create_widget_annotation(
        form_handle,
        "Gender",
        PdfFormFieldType::RadioButton,
        PdfRect::new(100.0, 660.0, 120.0, 680.0),
    )?;
    
    let female_radio = page.annotations_mut().create_widget_annotation(
        form_handle,
        "Gender",
        PdfFormFieldType::RadioButton,
        PdfRect::new(150.0, 660.0, 170.0, 680.0),
    )?;
    
    // Create a combo box for country
    let country_combo = page.annotations_mut().create_widget_annotation(
        form_handle,
        "Country",
        PdfFormFieldType::ComboBox,
        PdfRect::new(100.0, 630.0, 300.0, 650.0),
    )?;
    
    // Create a push button
    let submit_button = page.annotations_mut().create_widget_annotation(
        form_handle,
        "Submit",
        PdfFormFieldType::PushButton,
        PdfRect::new(100.0, 600.0, 200.0, 630.0),
    )?;
    
    // Save the document
    document.save_to_file("form_example.pdf")?;
    
    Ok(())
}
```

## Field Name Guidelines

- **Unique names**: Each field should have a unique name unless you're creating a radio button group
- **Radio button groups**: Radio buttons with the same name belong to the same group - only one can be selected
- **Valid characters**: Field names should be valid PDF names (avoid special characters that might cause issues)
- **UTF-8 encoding**: Field names are encoded as UTF-8

## Rectangle Coordinates

The `PdfRect` uses PDF coordinate system:
- Origin (0, 0) is at the **bottom-left** corner
- X increases to the right
- Y increases upward
- `PdfRect::new(bottom, left, top, right)` - note the order!

Example:
```rust
PdfRect::new(
    100.0,  // bottom (y-coordinate of bottom edge)
    50.0,   // left (x-coordinate of left edge)
    120.0,  // top (y-coordinate of top edge)
    250.0,  // right (x-coordinate of right edge)
)
```

## Accessing and Configuring Form Fields

After creating a widget annotation, you can access the underlying form field:

```rust
let widget = page.annotations_mut().create_widget_annotation(
    form_handle,
    "MyField",
    PdfFormFieldType::Text,
    rect,
)?;

// Get the form field
if let Some(field) = widget.form_field() {
    // Check the field type
    match field.field_type() {
        PdfFormFieldType::Text => {
            if let Some(text_field) = field.as_text_field() {
                // Configure text field properties
                // text_field.set_value("Default value")?;
                // text_field.set_max_length(50)?;
            }
        }
        PdfFormFieldType::Checkbox => {
            if let Some(checkbox) = field.as_checkbox_field() {
                // checkbox.set_checked(true)?;
            }
        }
        // ... handle other types
        _ => {}
    }
    
    // Common properties available on all fields
    // field.set_is_required(true)?;
    // field.set_is_read_only(false)?;
}
```

## Error Handling

The functions return `Result` types, so handle errors appropriately:

```rust
match page.annotations_mut().create_widget_annotation(
    form_handle,
    "MyField",
    PdfFormFieldType::Text,
    rect,
) {
    Ok(widget) => {
        println!("Widget created successfully!");
    }
    Err(PdfiumError::UnknownFormFieldType) => {
        eprintln!("Cannot create widget with Unknown field type");
    }
    Err(PdfiumError::PdfiumLibraryInternalError(_)) => {
        eprintln!("PDFium internal error occurred");
    }
    Err(e) => {
        eprintln!("Other error: {:?}", e);
    }
}
```

## WASM-Specific Notes

If you're using `pdfium-render` in a WASM environment (like the error messages suggest), the form fill environment initialization works the same way:

1. **Form Fill Environment**: The form fill environment must be initialized before creating widgets. Use `document.init_form_fill_environment()` as shown in Step 4.

2. **Memory Management**: The `init_form_fill_environment()` method properly handles memory management in WASM environments, so you don't need to worry about it.

3. **Error Messages**: If you see "Form handle not available" in WASM, it means you need to call `document.init_form_fill_environment()` after `ensure_acro_form()`.

## Important Notes

1. **Form Handle Required**: Widget annotations require a valid form handle. Make sure the form fill environment is initialized before creating widgets. **For new documents, you MUST call `document.init_form_fill_environment()` after `ensure_acro_form()`**. This method properly handles memory management and is idempotent (safe to call multiple times).

2. **AcroForm Must Exist**: Always call `document.ensure_acro_form()` before creating widget annotations.

3. **Field Type Unknown**: You cannot create a widget with `PdfFormFieldType::Unknown` - this will return an error.

4. **Automatic Flag Configuration**: The implementation automatically sets the correct flags for each field type (e.g., `ButtonIsPushButton` for push buttons, `ChoiceCombo` for combo boxes).

5. **Content Regeneration**: If the page has `AutomaticOnEveryChange` content regeneration strategy, content will be automatically regenerated after creating the widget.

6. **Memory Management**: Widget annotations are managed by the `PdfPageWidgetAnnotation` wrapper - no manual cleanup needed.

## Troubleshooting

**Problem**: "Form handle not available" or "Form fill environment may not be initialized"
- **Solution**: For new documents, you MUST initialize the form fill environment. The `ensure_acro_form()` function only creates the AcroForm dictionary but does not initialize the form fill environment. Call `document.init_form_fill_environment()` after `ensure_acro_form()` to initialize it properly.

**Problem**: "UnknownFormFieldType error"
- **Solution**: Don't use `PdfFormFieldType::Unknown` - use one of the 7 valid types (Text, PushButton, Checkbox, RadioButton, ComboBox, ListBox, Signature).

**Problem**: Widgets not appearing in PDF viewer
- **Solution**: 
  - Ensure the rectangle coordinates are correct (PDF uses bottom-left origin)
  - Make sure the form fill environment is properly initialized
  - Check that AcroForm was created successfully
  - Verify the form handle is valid

**Problem**: Radio buttons not grouping correctly
- **Solution**: Radio buttons with the same field name belong to the same group. Make sure all radio buttons in a group use the exact same name string.

## Known Limitations

### Options for Combo Boxes and List Boxes

**Current Status**: The `pdfium-render` library (and PDFium itself) currently only provides **read-only** access to form field options. There is no API to add or modify options after creating a combo box or list box widget annotation.

**Why**: PDFium's C API only includes functions to read options:
- `FPDFAnnot_GetOptionCount` - get the number of options
- `FPDFAnnot_GetOptionLabel` - get an option's label
- `FPDFAnnot_IsOptionSelected` - check if an option is selected

There are no corresponding functions to add or set options.

**Workarounds**:
1. **PDF Dictionary Manipulation** (Advanced): You could manipulate the PDF dictionary directly to add an `/Opt` array to the form field dictionary. This requires deep knowledge of PDF structure and is error-prone.
2. **Pre-defined Options**: Create form fields in PDFs that already have options defined, then load and modify those PDFs.
3. **Wait for API**: This functionality may be added to `pdfium-render` or PDFium in the future.

**Recommendation**: For now, if you need combo boxes or list boxes with options, consider:
- Creating them in a PDF authoring tool first, then loading that PDF
- Using text fields with validation instead
- Implementing dictionary manipulation if you're comfortable with PDF internals

## Next Steps

After creating widget annotations, you can:
- Set default values (for text fields, checkboxes, radio buttons)
- Configure field properties (required, read-only, etc.)
- Read existing options from combo boxes and list boxes (read-only)
- Set up field validation
- Configure appearance streams
- Add JavaScript actions

**Note**: Adding options to combo boxes and list boxes is not currently supported. See the "Known Limitations" section above.

Refer to the `pdfium-render` documentation for more details on form field manipulation.
