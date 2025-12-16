//! Defines the [PdfFormListBoxField] struct, exposing functionality related to a single
//! form field of type [PdfFormFieldType::ListBox].

use crate::bindgen::{FPDF_ANNOTATION, FPDF_DOCUMENT, FPDF_FORMHANDLE};
use crate::bindings::PdfiumLibraryBindings;
use crate::error::PdfiumError;
use crate::pdf::document::page::field::options::PdfFormFieldOptions;
use crate::pdf::document::page::field::private::internal::{
    PdfFormFieldFlags, PdfFormFieldPrivate,
};
use chrono::Utc;

#[cfg(doc)]
use {
    crate::pdf::document::form::PdfForm,
    crate::pdf::document::page::annotation::PdfPageAnnotationType,
    crate::pdf::document::page::field::{PdfFormField, PdfFormFieldType},
};

/// A single [PdfFormField] of type [PdfFormFieldType::ListBox]. The form field object defines
/// an interactive drop-down list widget that allows the user to select a value from
/// a list of options.
///
/// Form fields in Pdfium are wrapped inside page annotations of type [PdfPageAnnotationType::Widget]
/// or [PdfPageAnnotationType::XfaWidget]. User-specified values can be retrieved directly from
/// each form field object by unwrapping the form field from the annotation, or in bulk from the
/// [PdfForm::field_values()] function.
pub struct PdfFormListBoxField<'a> {
    form_handle: FPDF_FORMHANDLE,
    annotation_handle: FPDF_ANNOTATION,
    document_handle: Option<FPDF_DOCUMENT>,
    options: PdfFormFieldOptions<'a>,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfFormListBoxField<'a> {
    #[inline]
    pub(crate) fn from_pdfium(
        form_handle: FPDF_FORMHANDLE,
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfFormListBoxField {
            form_handle,
            annotation_handle,
            document_handle: None,
            options: PdfFormFieldOptions::from_pdfium(form_handle, annotation_handle, bindings),
            bindings,
        }
    }

    #[inline]
    pub(crate) fn from_pdfium_with_document(
        form_handle: FPDF_FORMHANDLE,
        annotation_handle: FPDF_ANNOTATION,
        document_handle: FPDF_DOCUMENT,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfFormListBoxField {
            form_handle,
            annotation_handle,
            document_handle: Some(document_handle),
            options: PdfFormFieldOptions::from_pdfium(form_handle, annotation_handle, bindings),
            bindings,
        }
    }

    /// Returns the [PdfiumLibraryBindings] used by this [PdfFormListBoxField] object.
    #[inline]
    pub fn bindings(&self) -> &'a dyn PdfiumLibraryBindings {
        self.bindings
    }

    /// Returns the collection of selectable options in this [PdfFormListBoxField].
    pub fn options(&self) -> &PdfFormFieldOptions<'_> {
        &self.options
    }

    /// Returns the current value(s) of this [PdfFormListBoxField] object.
    /// 
    /// For multi-select list boxes (where [PdfFormListBoxField::is_multiselect()] returns `true`),
    /// this returns a comma-separated string of all selected option labels.
    /// For single-select list boxes, this returns the label of the currently selected option.
    #[inline]
    pub fn value(&self) -> Option<String> {
        let selected: Vec<String> = self.options()
            .iter()
            .filter_map(|option| {
                if option.is_set() {
                    option.label().cloned()
                } else {
                    None
                }
            })
            .collect();
        
        if selected.is_empty() {
            None
        } else if self.is_multiselect() {
            // For multi-select, return comma-separated values
            Some(selected.join(","))
        } else {
            // For single-select, return the first (and only) selected value
            selected.first().cloned()
        }
    }

    /// Sets the value of this [PdfFormListBoxField] object.
    ///
    /// The value should match the label of one of the available options in this list box.
    /// For list boxes that support multiple selections ([PdfFormListBoxField::is_multiselect()]
    /// returns `true`), this method sets a single selected value; use [PdfFormListBoxField::set_values()]
    /// for multi-selection scenarios.
    /// 
    /// This method attempts to use PDFium's form fill API to set the value, which ensures
    /// appearance streams are properly regenerated. If the form fill API cannot be used,
    /// it falls back to direct annotation manipulation.
    #[inline]
    pub fn set_value(&mut self, value: &str) -> Result<(), PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&"🔧 PdfFormListBoxField::set_value() - ATTEMPTING FORM FILL API".into());
            console::log_1(&format!("   Setting value: '{}'", value).into());
        }

        // Find the option index that matches the value
        let option_index = self.options()
            .iter()
            .find(|option| {
                option.label()
                    .map(|label| label == value)
                    .unwrap_or(false)
            })
            .map(|option| option.index());

        let option_index = match option_index {
            Some(idx) => idx,
            None => {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::warn_1(&format!("   ⚠️ Option '{}' not found in list box options", value).into());
                    console::warn_1(&"   ⚠️ Falling back to direct annotation manipulation".into());
                }
                return self.set_value_impl(value);
            }
        };

        // Find page and annotation handles
        let result_opt = self.find_page_and_annotation_handle_for_annotation();

        if let Some((page_handle, page_annotation_handle)) = result_opt {
            let result = self.set_value_via_form_fill_with_page_and_annotation(
                option_index as i32,
                page_handle,
                page_annotation_handle,
            );
            // Close handles
            self.bindings().FPDFPage_CloseAnnot(page_annotation_handle);
            self.bindings().FPDF_ClosePage(page_handle);
            result
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::warn_1(&"   ⚠️ Could not find page handle for annotation".into());
                console::warn_1(&"   ⚠️ Falling back to direct annotation manipulation".into());
            }
            self.set_value_impl(value)
        }
    }

    /// Sets the value using form fill API with known page and annotation handles
    fn set_value_via_form_fill_with_page_and_annotation(
        &mut self,
        option_index: i32,
        page_handle: crate::bindgen::FPDF_PAGE,
        annotation_handle: crate::bindgen::FPDF_ANNOTATION,
    ) -> Result<(), PdfiumError> {
        let form_handle = self.form_handle();
        let bindings = self.bindings();

        // Ensure FORM_OnAfterLoadPage is called
        bindings.FORM_OnAfterLoadPage(page_handle, form_handle);

        // Step 1: Focus the annotation
        if bindings.is_true(bindings.FORM_SetFocusedAnnot(form_handle, annotation_handle)) {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"   ✅ FORM_SetFocusedAnnot succeeded".into());
            }

            // Step 2: Set the option as selected using FORM_SetIndexSelected
            if bindings.is_true(bindings.FORM_SetIndexSelected(
                form_handle,
                page_handle,
                option_index,
                bindings.TRUE(),
            )) {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&format!("   ✅ FORM_SetIndexSelected({}) succeeded", option_index).into());
                }

                // Step 3: Kill focus to save the value and trigger appearance stream regeneration
                bindings.FORM_ForceToKillFocus(form_handle);

                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&"   ✅ FORM_ForceToKillFocus called - appearance stream should regenerate".into());
                    console::log_1(&"   ✅ FORM FILL API METHOD COMPLETE".into());
                    console::log_1(&"═══════════════════════════════════════════════════════════".into());
                }

                // Also update modification date
                self.set_string_value("M", &crate::utils::dates::date_time_to_pdf_string(Utc::now()))?;

                Ok(())
            } else {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::warn_1(&format!("   ⚠️ FORM_SetIndexSelected({}) failed", option_index).into());
                }
                self.set_value_impl("") // Fall back
            }
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::warn_1(&"   ⚠️ FORM_SetFocusedAnnot failed".into());
            }
            self.set_value_impl("") // Fall back
        }
    }

    /// Attempts to find the page handle and annotation handle that contains this annotation.
    /// Similar to PdfFormTextField::find_page_and_annotation_handle_for_annotation
    fn find_page_and_annotation_handle_for_annotation(
        &self,
    ) -> Option<(crate::bindgen::FPDF_PAGE, crate::bindgen::FPDF_ANNOTATION)> {
        let document_handle = self.document_handle?;
        let annotation_handle = self.annotation_handle();
        let bindings = self.bindings();

        // Get the rectangle of the annotation we're looking for
        let mut target_rect = crate::bindgen::FS_RECTF {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        };
        if !bindings.is_true(bindings.FPDFAnnot_GetRect(annotation_handle, &mut target_rect)) {
            return None;
        }

        let page_count = bindings.FPDF_GetPageCount(document_handle);
        for i in 0..page_count {
            let page_handle = bindings.FPDF_LoadPage(document_handle, i);
            if !page_handle.is_null() {
                let annot_count = bindings.FPDFPage_GetAnnotCount(page_handle);
                for j in 0..annot_count {
                    let current_annot_handle = bindings.FPDFPage_GetAnnot(page_handle, j);
                    if !current_annot_handle.is_null() {
                        let mut current_rect = crate::bindgen::FS_RECTF {
                            left: 0.0,
                            top: 0.0,
                            right: 0.0,
                            bottom: 0.0,
                        };
                        
                        if bindings.is_true(bindings.FPDFAnnot_GetRect(current_annot_handle, &mut current_rect)) {
                            const EPSILON: f32 = 0.01;
                            let rects_match = 
                                (current_rect.left - target_rect.left).abs() < EPSILON &&
                                (current_rect.top - target_rect.top).abs() < EPSILON &&
                                (current_rect.right - target_rect.right).abs() < EPSILON &&
                                (current_rect.bottom - target_rect.bottom).abs() < EPSILON;

                            if rects_match {
                                return Some((page_handle, current_annot_handle));
                            }
                        }
                        bindings.FPDFPage_CloseAnnot(current_annot_handle);
                    }
                }
                bindings.FPDF_ClosePage(page_handle);
            }
        }
        None
    }

    /// Sets multiple selected values for this [PdfFormListBoxField] object.
    ///
    /// This method is intended for list boxes that support multiple selections
    /// ([PdfFormListBoxField::is_multiselect()] returns `true`). The values should match
    /// the labels of available options in this list box.
    ///
    /// **Note:** This implementation has limitations. For multi-select list boxes, the PDF
    /// specification requires the "V" entry to be an array, but PDFium's API only allows
    /// setting string values. This method attempts to work around this by setting the first
    /// value, which may not fully support multi-select. Proper multi-select support would
    /// require direct PDF dictionary manipulation to set V as an array.
    pub fn set_values(&mut self, values: &[&str]) -> Result<(), PdfiumError> {
        if values.is_empty() {
            // Clear all selections by setting empty value
            return self.set_value_impl("");
        }
        
        if values.len() == 1 {
            // Single value - use the standard set_value
            return self.set_value_impl(values[0]);
        }
        
        // For multiple values, PDFium's API limitation means we can only set one value
        // as a string. The PDF spec requires V to be an array for multi-select, but
        // FPDFAnnot_SetStringValue only sets strings. We'll set the first value for now.
        // TODO: Implement proper multi-select by directly manipulating the PDF dictionary
        // to set V as an array of option values or indices.
        self.set_value_impl(values[0])
    }

    /// Returns `true` if the option items of this [PdfFormListBoxField] should be sorted
    /// alphabetically.
    ///
    /// This flag is intended for use by form authoring tools, not by PDF viewer applications.
    #[inline]
    pub fn is_sorted(&self) -> bool {
        self.get_flags_impl()
            .contains(PdfFormFieldFlags::ChoiceSort)
    }

    #[cfg(any(feature = "pdfium_future", feature = "pdfium_7350"))]
    /// Controls whether or not the option items of this [PdfFormListBoxField] should be
    /// sorted alphabetically.
    ///
    /// This flag is intended for use by form authoring tools, not by PDF viewer applications.
    #[inline]
    pub fn set_is_sorted(&mut self, is_sorted: bool) -> Result<(), PdfiumError> {
        self.update_one_flag_impl(PdfFormFieldFlags::ChoiceSort, is_sorted)
    }

    /// Returns `true` if more than one of the option items in this [PdfFormListBoxField]
    /// may be selected simultaneously. If `false`, only one item at a time may be selected.
    ///
    /// This flag was added in PDF version 1.4.
    pub fn is_multiselect(&self) -> bool {
        self.get_flags_impl()
            .contains(PdfFormFieldFlags::ChoiceMultiSelect)
    }

    #[cfg(any(feature = "pdfium_future", feature = "pdfium_7350"))]
    /// Controls whether more than one of the option items in this [PdfFormListBoxField]
    /// may be selected simultaneously.
    ///
    /// This flag was added in PDF version 1.4.
    pub fn set_is_multiselect(&mut self, is_multiselect: bool) -> Result<(), PdfiumError> {
        self.update_one_flag_impl(PdfFormFieldFlags::ChoiceMultiSelect, is_multiselect)
    }

    /// Returns `true` if any new value is committed to this [PdfFormListBoxField]
    /// as soon as a selection is made with the pointing device. This option enables
    /// applications to perform an action once a selection is made, without requiring
    /// the user to exit the field. If `false`, any new value is not committed until the
    /// user exits the field.
    ///
    /// This flag was added in PDF version 1.5.
    pub fn is_commit_on_selection_change(&self) -> bool {
        self.get_flags_impl()
            .contains(PdfFormFieldFlags::ChoiceCommitOnSelectionChange)
    }

    #[cfg(any(feature = "pdfium_future", feature = "pdfium_7350"))]
    /// Controls whether or not any new value is committed to this [PdfFormListBoxField]
    /// as soon as a selection is made with the pointing device.
    ///
    /// This flag was added in PDF version 1.5.
    pub fn set_is_commit_on_selection_change(
        &mut self,
        is_commit_on_selection_change: bool,
    ) -> Result<(), PdfiumError> {
        self.update_one_flag_impl(
            PdfFormFieldFlags::ChoiceCommitOnSelectionChange,
            is_commit_on_selection_change,
        )
    }
}

impl<'a> PdfFormFieldPrivate<'a> for PdfFormListBoxField<'a> {
    #[inline]
    fn form_handle(&self) -> FPDF_FORMHANDLE {
        self.form_handle
    }

    #[inline]
    fn annotation_handle(&self) -> FPDF_ANNOTATION {
        self.annotation_handle
    }

    #[inline]
    fn bindings(&self) -> &dyn PdfiumLibraryBindings {
        self.bindings
    }
}
