use pdfium_render::prelude::*;

pub fn main() -> Result<(), PdfiumError> {
    // For general comments about pdfium-render and binding to Pdfium, see export.rs.

    let pdfium = Pdfium::default();

    let mut document = pdfium.load_pdf_from_file("test/form-test.pdf", None)?;

    match document.form() {
        Some(form) => println!(
            "PDF contains an embedded form of type {:#?}",
            form.form_type()
        ),
        None => println!("PDF does not contain an embedded form"),
    };

    // CRITICAL: Initialize form fill environment if not already initialized
    // This is required for form fill API functions to work properly
    let form_handle = document.init_form_fill_environment()?;

    // Form fields in Pdfium are wrapped within page annotation objects, specifically annotations
    // of type `PdfPageAnnotationType::Widget` or `PdfPageAnnotationType::XfaWidget` (depending on
    // the type of form embedded in the document). To retrieve the form field values, we iterate
    // over each annotation on each page in the document, examining just the annotations capable of
    // wrapping a form field.

    let pages = document.pages();

    for (page_index, page) in pages.iter().enumerate() {
        // CRITICAL: Call FORM_OnAfterLoadPage before interacting with form fields
        // This initializes the form fill environment for this page
        page.bindings().FORM_OnAfterLoadPage(page.page_handle(), form_handle);

        let annotations = page.annotations();

        for (annotation_index, mut annotation) in annotations.iter().enumerate() {
            // The PdfPageAnnotation::as_form_field() helper function handles the filtering out
            // of non-form-field-wrapping annotations for us.

            if let Some(field) = annotation.as_form_field_mut() {
                // Get the annotation handle and page handle for form fill API operations
                let annotation_handle = annotation.handle();
                let page_handle = page.page_handle();
                let bindings = page.bindings();

                if let Some(field) = field.as_radio_button_field_mut() {
                    // For radio buttons, use mouse click simulation through form fill API
                    // This properly handles radio button group behavior
                    println!(
                        "Page {}, radio button {}: {:?} currently has value: {}",
                        page_index,
                        annotation_index,
                        field.name(),
                        format!("{:?}", field.is_checked()),
                    );

                    // Get the annotation bounds for mouse click
                    let rect = annotation.bounds()?;
                    let center_x = (rect.left + rect.right) / 2.0;
                    let center_y = (rect.top + rect.bottom) / 2.0;

                    // Focus the annotation first
                    if bindings.is_true(bindings.FORM_SetFocusedAnnot(form_handle, annotation_handle)) {
                        // Simulate mouse click at center of annotation
                        // This will properly handle radio button group behavior
                        bindings.FORM_OnLButtonDown(
                            form_handle,
                            page_handle,
                            0, // No modifier keys
                            center_x,
                            center_y,
                        );
                        bindings.FORM_OnLButtonUp(
                            form_handle,
                            page_handle,
                            0, // No modifier keys
                            center_x,
                            center_y,
                        );
                        
                        // Kill focus after interaction
                        bindings.FORM_ForceToKillFocus(form_handle);
                    }

                    println!(
                        "Page {}, radio button {}: {:?} now has updated value: {}",
                        page_index,
                        annotation_index,
                        field.name(),
                        format!("{:?}", field.is_checked()),
                    );
                } else if let Some(field) = field.as_checkbox_field_mut() {
                    // For checkboxes, use mouse click simulation through form fill API
                    println!(
                        "Page {}, checkbox {}: {:?} currently has value: {}",
                        page_index,
                        annotation_index,
                        field.name(),
                        format!("{:?}", field.is_checked()),
                    );

                    // Get the annotation bounds for mouse click
                    let rect = annotation.bounds()?;
                    let center_x = (rect.left + rect.right) / 2.0;
                    let center_y = (rect.top + rect.bottom) / 2.0;

                    // Focus the annotation first
                    if bindings.is_true(bindings.FORM_SetFocusedAnnot(form_handle, annotation_handle)) {
                        // Simulate mouse click at center of annotation
                        bindings.FORM_OnLButtonDown(
                            form_handle,
                            page_handle,
                            0, // No modifier keys
                            center_x,
                            center_y,
                        );
                        bindings.FORM_OnLButtonUp(
                            form_handle,
                            page_handle,
                            0, // No modifier keys
                            center_x,
                            center_y,
                        );
                        
                        // Kill focus after interaction
                        bindings.FORM_ForceToKillFocus(form_handle);
                    }

                    println!(
                        "Page {}, checkbox {}: {:?} now has updated value: {}",
                        page_index,
                        annotation_index,
                        field.name(),
                        format!("{:?}", field.is_checked()),
                    );
                } else if let Some(field) = field.as_text_field_mut() {
                    // For text fields, use FORM_ReplaceSelection API
                    // This ensures appearance streams are properly regenerated
                    println!(
                        "Page {}, text field {}: {:?} currently has value: {}",
                        page_index,
                        annotation_index,
                        field.name(),
                        format!("{:?}", field.value()),
                    );

                    let new_value = field
                        .name()
                        .unwrap_or_else(|| format!("field-{}-{}", page_index, annotation_index));

                    // Use form fill API to set text field value
                    // Step 1: Focus the annotation
                    if bindings.is_true(bindings.FORM_SetFocusedAnnot(form_handle, annotation_handle)) {
                        // Step 2: Select all existing text
                        bindings.FORM_SelectAllText(form_handle, page_handle);

                        // Step 3: Replace selection with new value
                        // FORM_ReplaceSelection expects UTF-16LE encoded string
                        let utf16le_bytes = bindings.get_pdfium_utf16le_bytes_from_str(&new_value);
                        // Cast the byte pointer to u16 pointer (FPDF_WIDESTRING is *const u16)
                        let ws_text = utf16le_bytes.as_ptr() as *const std::os::raw::c_ushort;

                        bindings.FORM_ReplaceSelection(form_handle, page_handle, ws_text);

                        // Step 4: Kill focus to save the value and trigger appearance stream regeneration
                        bindings.FORM_ForceToKillFocus(form_handle);
                    }

                    println!(
                        "Page {}, text field {}: {:?} now has updated value: {}",
                        page_index,
                        annotation_index,
                        field.name(),
                        format!("{:?}", field.value()),
                    );
                }
            }
        }

        // CRITICAL: Call FORM_OnBeforeClosePage when done with the page
        // This cleans up form fill environment resources for this page
        page.bindings().FORM_OnBeforeClosePage(page.page_handle(), form_handle);
    }

    document.save_to_file("test/fill-form-test.pdf")
}
