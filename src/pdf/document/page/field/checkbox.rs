//! Defines the [PdfFormCheckboxField] struct, exposing functionality related to a single
//! form field of type [PdfFormFieldType::Checkbox].

use crate::bindgen::{FPDF_ANNOTATION, FPDF_FORMHANDLE};
use crate::bindings::PdfiumLibraryBindings;
use crate::error::PdfiumError;
use crate::pdf::document::page::field::private::internal::PdfFormFieldPrivate;

#[cfg(doc)]
use {
    crate::pdf::document::form::PdfForm,
    crate::pdf::document::page::annotation::PdfPageAnnotationType,
    crate::pdf::document::page::field::{PdfFormField, PdfFormFieldType},
};

/// A single [PdfFormField] of type [PdfFormFieldType::Checkbox]. The form field object defines
/// an interactive checkbox widget that can be toggled by the user.
///
/// Form fields in Pdfium are wrapped inside page annotations of type [PdfPageAnnotationType::Widget]
/// or [PdfPageAnnotationType::XfaWidget]. User-specified values can be retrieved directly from
/// each form field object by unwrapping the form field from the annotation, or in bulk from the
/// [PdfForm::field_values()] function.
pub struct PdfFormCheckboxField<'a> {
    form_handle: FPDF_FORMHANDLE,
    annotation_handle: FPDF_ANNOTATION,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfFormCheckboxField<'a> {
    #[inline]
    pub(crate) fn from_pdfium(
        form_handle: FPDF_FORMHANDLE,
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfFormCheckboxField {
            form_handle,
            annotation_handle,
            bindings,
        }
    }

    /// Returns the [PdfiumLibraryBindings] used by this [PdfFormCheckboxField] object.
    #[inline]
    pub fn bindings(&self) -> &'a dyn PdfiumLibraryBindings {
        self.bindings
    }

    /// Returns the index of this [PdfFormCheckboxField] in its control group.
    ///
    /// Control groups are used to group related interactive fields together. Checkboxes and
    /// radio buttons can be grouped such that only a single button can be selected within
    /// the control group. Each field within the group has a unique group index.
    #[inline]
    pub fn index_in_group(&self) -> u32 {
        self.index_in_group_impl()
    }

    /// Returns the value set for the control group containing this [PdfFormCheckboxField].
    ///
    /// Control groups are used to group related interactive fields together. Checkboxes and
    /// radio buttons can be grouped such that only a single button can be selected within
    /// the control group. In this case, a single value can be shared by the group, indicating
    /// the value of the currently selected field within the group.
    #[inline]
    pub fn group_value(&self) -> Option<String> {
        self.value_impl()
    }

    /// Returns `true` if this [PdfFormCheckboxField] object has its checkbox checked.
    #[inline]
    pub fn is_checked(&self) -> Result<bool, PdfiumError> {
        // The PDF Reference manual, version 1.7, states that an appearance stream of "Yes"
        // can be used to indicate a selected checkbox. However, PDFs can use custom export
        // values (e.g., "1", "On", or custom names), so we compare against the actual
        // export value when available.

        // Get the export value for this checkbox (the "on" state value)
        let on_value = self.export_value_impl();

        // Helper to check if a value matches the "on" state
        let is_on = |value: &str| -> bool {
            let normalized = value.trim_start_matches('/');
            if let Some(ref export_val) = on_value {
                let normalized_export = export_val.trim_start_matches('/');
                normalized == normalized_export
            } else {
                // Fallback: check for standard "Yes" value
                normalized == "Yes"
            }
        };

        match self.appearance_stream_impl().as_ref() {
            Some(appearance_stream) => {
                // Appearance streams are in use. Compare against the export value.
                Ok(is_on(appearance_stream))
            }
            None => {
                // Appearance streams are not in use. We can fall back to using Pdfium's
                // FPDFAnnot_IsChecked() implementation.

                match self.is_checked_impl() {
                    Ok(true) => Ok(true),
                    Ok(false) => match self.group_value() {
                        Some(value) => Ok(is_on(&value)),
                        _ => Ok(false),
                    },
                    Err(err) => match self.group_value() {
                        Some(value) => Ok(is_on(&value)),
                        _ => Err(err),
                    },
                }
            }
        }
    }

    /// Checks or clears the checkbox of this [PdfFormCheckboxField] object.
    #[inline]
    pub fn set_checked(&mut self, is_checked: bool) -> Result<(), PdfiumError> {
        // Get the actual export value for this checkbox (what it sends when checked).
        // Fall back to "Yes" if not defined (standard PDF convention).
        let on_value = self
            .export_value_impl()
            .unwrap_or_else(|| "Yes".to_string());
        let off_value = "Off";

        // Normalize: strip any leading slash from the export value
        let normalized_on = on_value.trim_start_matches('/');

        // Determine which value to use based on desired state
        let (value_with_slash, value_without_slash) = if is_checked {
            (format!("/{}", normalized_on), normalized_on.to_string())
        } else {
            (format!("/{}", off_value), off_value.to_string())
        };

        // Set both the appearance stream selector (AS) and the value (V).
        // The AS key selects which visual appearance from the AP dictionary to display.
        // The V key is the actual form field value that gets saved/submitted.
        // The appearance streams already exist in the PDF - we just select which one to use.
        self.set_string_value("AS", &value_with_slash)?;
        self.set_value_impl(&value_without_slash)
    }
}

impl<'a> PdfFormFieldPrivate<'a> for PdfFormCheckboxField<'a> {
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
