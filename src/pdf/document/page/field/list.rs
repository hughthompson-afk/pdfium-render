//! Defines the [PdfFormListBoxField] struct, exposing functionality related to a single
//! form field of type [PdfFormFieldType::ListBox].

use crate::bindgen::{FPDF_ANNOTATION, FPDF_FORMHANDLE};
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
    #[inline]
    pub fn set_value(&mut self, value: &str) -> Result<(), PdfiumError> {
        self.set_value_impl(value)
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
