//! Defines the [PdfFormComboBoxField] struct, exposing functionality related to a single
//! form field of type [PdfFormFieldType::ComboBox].

use crate::bindgen::{FPDF_ANNOTATION, FPDF_DOCUMENT, FPDF_FORMHANDLE};
use crate::bindings::PdfiumLibraryBindings;
use crate::error::PdfiumError;
use crate::pdf::document::page::field::options::PdfFormFieldOptions;
use crate::pdf::document::page::field::private::internal::{
    PdfFormFieldFlags, PdfFormFieldPrivate,
};

#[cfg(doc)]
use {
    crate::pdf::document::form::PdfForm,
    crate::pdf::document::page::annotation::PdfPageAnnotationType,
    crate::pdf::document::page::field::{PdfFormField, PdfFormFieldType},
};

/// A single [PdfFormField] of type [PdfFormFieldType::ComboBox]. The form field object defines
/// an interactive drop-down list widget that allows the user to either select a value
/// from a list of options or type a value into a text field.
///
/// Form fields in Pdfium are wrapped inside page annotations of type [PdfPageAnnotationType::Widget]
/// or [PdfPageAnnotationType::XfaWidget]. User-specified values can be retrieved directly from
/// each form field object by unwrapping the form field from the annotation, or in bulk from the
/// [PdfForm::field_values()] function.
pub struct PdfFormComboBoxField<'a> {
    form_handle: FPDF_FORMHANDLE,
    annotation_handle: FPDF_ANNOTATION,
    document_handle: Option<FPDF_DOCUMENT>,
    options: PdfFormFieldOptions<'a>,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfFormComboBoxField<'a> {
    #[inline]
    pub(crate) fn from_pdfium(
        form_handle: FPDF_FORMHANDLE,
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfFormComboBoxField {
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
        PdfFormComboBoxField {
            form_handle,
            annotation_handle,
            document_handle: Some(document_handle),
            options: PdfFormFieldOptions::from_pdfium(form_handle, annotation_handle, bindings),
            bindings,
        }
    }

    /// Returns the [PdfiumLibraryBindings] used by this [PdfFormComboBoxField] object.
    #[inline]
    pub fn bindings(&self) -> &'a dyn PdfiumLibraryBindings {
        self.bindings
    }

    /// Returns the collection of selectable options in this [PdfFormComboBoxField].
    pub fn options(&self) -> &PdfFormFieldOptions<'_> {
        &self.options
    }

    /// Returns the current value of this [PdfFormComboBoxField] object, if any.
    /// 
    /// For editable combo boxes (where [PdfFormComboBoxField::has_editable_text_box()] returns `true`),
    /// this returns the raw text value which may be a custom value not in the options list.
    /// For non-editable combo boxes, this returns the label of the currently selected option.
    #[inline]
    pub fn value(&self) -> Option<String> {
        // For editable combo boxes, use the raw value from the V entry
        // This handles custom values that aren't in the options list
        if self.has_editable_text_box() {
            self.value_impl()
        } else {
            // For non-editable combo boxes, find the selected option
            self.options()
                .iter()
                .find(|option| option.is_set())
                .and_then(|option| option.label().cloned())
        }
    }

    /// Sets the value of this [PdfFormComboBoxField] object.
    ///
    /// The value should match the label of one of the available options in this combo box.
    /// For combo boxes with an editable text box ([PdfFormComboBoxField::has_editable_text_box()]
    /// returns `true`), arbitrary text values may also be accepted.
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
            console::log_1(&"🔧 PdfFormComboBoxField::set_value() - ATTEMPTING FORM FILL API".into());
            console::log_1(&format!("   Setting value: '{}'", value).into());
        }

        // For editable combo boxes, treat like text fields
        if self.has_editable_text_box() {
            // Use text field approach for editable combo boxes
            return self.set_value_as_text_field(value);
        }

        // For non-editable combo boxes, find the option index and use form fill API
        self.set_value_via_form_fill(value)
    }

    /// Sets value for editable combo boxes using text field form fill API
    fn set_value_as_text_field(&mut self, value: &str) -> Result<(), PdfiumError> {
        // Similar to text fields - use FORM_ReplaceSelection
        // This is a simplified version - we could share code with text fields
        self.set_value_impl(value) // For now, fall back to old method for editable combo boxes
    }

    /// Sets the value using PDFium's form fill API for non-editable combo boxes
    fn set_value_via_form_fill(&mut self, value: &str) -> Result<(), PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"   🔄 Attempting to use form fill API for combo box...".into());
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
                    console::warn_1(&format!("   ⚠️ Option '{}' not found in combo box options", value).into());
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
                self.set_string_value("M", &crate::utils::dates::date_time_to_pdf_string(chrono::Utc::now()))?;

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

    /// Returns `true` if this [PdfFormComboBoxField] also includes an editable text box.
    /// If `false`, this combo box field only includes a drop-down list.
    #[inline]
    pub fn has_editable_text_box(&self) -> bool {
        self.get_flags_impl()
            .contains(PdfFormFieldFlags::ChoiceEdit)
    }

    #[cfg(any(feature = "pdfium_future", feature = "pdfium_7350"))]
    /// Controls whether or not this [PdfFormComboBoxField] includes an editable text box
    /// in addition to a drop-down list.
    #[inline]
    pub fn set_has_editable_text_box(
        &mut self,
        has_editable_text_box: bool,
    ) -> Result<(), PdfiumError> {
        self.update_one_flag_impl(PdfFormFieldFlags::ChoiceEdit, has_editable_text_box)
    }

    /// Returns `true` if the option items of this [PdfFormComboBoxField] should be sorted
    /// alphabetically.
    ///
    /// This flag is intended for use by form authoring tools, not by PDF viewer applications.
    #[inline]
    pub fn is_sorted(&self) -> bool {
        self.get_flags_impl()
            .contains(PdfFormFieldFlags::ChoiceSort)
    }

    #[cfg(any(feature = "pdfium_future", feature = "pdfium_7350"))]
    /// Controls whether or not the option items of this [PdfFormComboBoxField] should be
    /// sorted alphabetically.
    ///
    /// This flag is intended for use by form authoring tools, not by PDF viewer applications.
    #[inline]
    pub fn set_is_sorted(&mut self, is_sorted: bool) -> Result<(), PdfiumError> {
        self.update_one_flag_impl(PdfFormFieldFlags::ChoiceSort, is_sorted)
    }

    /// Returns `true` if more than one of the option items in this [PdfFormComboBoxField]
    /// may be selected simultaneously. If `false`, only one item at a time may be selected.
    ///
    /// This flag was added in PDF version 1.4.
    pub fn is_multiselect(&self) -> bool {
        self.get_flags_impl()
            .contains(PdfFormFieldFlags::ChoiceMultiSelect)
    }

    #[cfg(any(feature = "pdfium_future", feature = "pdfium_7350"))]
    /// Controls whether more than one of the option items in this [PdfFormComboBoxField]
    /// may be selected simultaneously.
    ///
    /// This flag was added in PDF version 1.4.
    pub fn set_is_multiselect(&mut self, is_multiselect: bool) -> Result<(), PdfiumError> {
        self.update_one_flag_impl(PdfFormFieldFlags::ChoiceMultiSelect, is_multiselect)
    }

    /// Returns `true` if text entered into the editable text box included in this
    /// [PdfFormComboBoxField] should be spell checked.
    ///
    /// This flag is meaningful only if the [PdfFormComboBoxField::has_editable_text_box()]
    /// flag is also `true`.
    ///
    /// This flag was added in PDF version 1.4.
    pub fn is_spell_checked(&self) -> bool {
        !self
            .get_flags_impl()
            .contains(PdfFormFieldFlags::TextDoNotSpellCheck)
    }

    #[cfg(any(feature = "pdfium_future", feature = "pdfium_7350"))]
    /// Controls whether or not text entered into the editable text box included in this
    /// [PdfFormComboBoxField] should be spell checked.
    ///
    /// This flag was added in PDF version 1.4.
    pub fn set_is_spell_checked(&mut self, is_spell_checked: bool) -> Result<(), PdfiumError> {
        self.update_one_flag_impl(PdfFormFieldFlags::TextDoNotSpellCheck, !is_spell_checked)
    }

    /// Returns `true` if any new value is committed to this [PdfFormComboBoxField]
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
    /// Controls whether or not any new value is committed to this [PdfFormComboBoxField]
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

impl<'a> PdfFormFieldPrivate<'a> for PdfFormComboBoxField<'a> {
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
