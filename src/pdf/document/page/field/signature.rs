//! Defines the [PdfFormSignatureField] struct, exposing functionality related to a single
//! form field of type [PdfFormFieldType::Signature].

use crate::bindgen::{FPDF_ANNOTATION, FPDF_FORMHANDLE};
use crate::bindings::PdfiumLibraryBindings;
use crate::pdf::document::page::annotation::signature_appearance::SignatureAppearanceBuilder;
use crate::pdf::document::page::field::private::internal::PdfFormFieldPrivate;

#[cfg(doc)]
use {
    crate::pdf::document::form::PdfForm,
    crate::pdf::document::page::annotation::PdfPageAnnotationType,
    crate::pdf::document::page::field::{PdfFormField, PdfFormFieldType},
};

/// A single [PdfFormField] of type [PdfFormFieldType::Signature]. The form field object defines
/// an interactive data entry widget that allows the user to draw a signature.
///
/// Form fields in Pdfium are wrapped inside page annotations of type [PdfPageAnnotationType::Widget]
/// or [PdfPageAnnotationType::XfaWidget]. User-specified values can be retrieved directly from
/// each form field object by unwrapping the form field from the annotation, or in bulk from the
/// [PdfForm::field_values()] function.
pub struct PdfFormSignatureField<'a> {
    form_handle: FPDF_FORMHANDLE,
    annotation_handle: FPDF_ANNOTATION,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfFormSignatureField<'a> {
    pub(crate) fn from_pdfium(
        form_handle: FPDF_FORMHANDLE,
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfFormSignatureField {
            form_handle,
            annotation_handle,
            bindings,
        }
    }

    /// Returns the [PdfiumLibraryBindings] used by this [PdfFormSignatureField] object.
    #[inline]
    pub fn bindings(&self) -> &'a dyn PdfiumLibraryBindings {
        self.bindings
    }

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
    /// ```rust,ignore
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
        SignatureAppearanceBuilder::new(self.annotation_handle, self.bindings)
    }
}

impl<'a> PdfFormFieldPrivate<'a> for PdfFormSignatureField<'a> {
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
