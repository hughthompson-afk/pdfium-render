//! Defines the [PdfPageFreeTextAnnotation] struct, exposing functionality related to a single
//! user annotation of type [PdfPageAnnotationType::FreeText].

use crate::bindgen::{FPDF_ANNOTATION, FPDF_DOCUMENT, FPDF_PAGE};
use crate::bindings::PdfiumLibraryBindings;
use crate::error::PdfiumError;
use crate::pdf::document::page::annotation::attachment_points::PdfPageAnnotationAttachmentPoints;
use crate::pdf::document::page::annotation::free_text_appearance::FreeTextAppearanceBuilder;
use crate::pdf::document::page::annotation::objects::PdfPageAnnotationObjects;
use crate::pdf::document::page::annotation::private::internal::PdfPageAnnotationPrivate;
use crate::pdf::document::page::annotation::PdfPageAnnotationCommon;
use crate::pdf::document::page::object::ownership::PdfPageObjectOwnership;
use crate::pdf::document::page::objects::private::internal::PdfPageObjectsPrivate;

// Re-export appearance types for convenience
pub use crate::pdf::document::page::annotation::free_text_appearance::*;

#[cfg(doc)]
use crate::pdf::document::page::annotation::{PdfPageAnnotation, PdfPageAnnotationType};

/// A single [PdfPageAnnotation] of type [PdfPageAnnotationType::FreeText].
pub struct PdfPageFreeTextAnnotation<'a> {
    handle: FPDF_ANNOTATION,
    objects: PdfPageAnnotationObjects<'a>,
    attachment_points: PdfPageAnnotationAttachmentPoints<'a>,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfPageFreeTextAnnotation<'a> {
    pub(crate) fn from_pdfium(
        document_handle: FPDF_DOCUMENT,
        page_handle: FPDF_PAGE,
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        let mut annotation = PdfPageFreeTextAnnotation {
            handle: annotation_handle,
            objects: PdfPageAnnotationObjects::from_pdfium(
                document_handle,
                page_handle,
                annotation_handle,
                bindings,
            ),
            attachment_points: PdfPageAnnotationAttachmentPoints::from_pdfium(
                annotation_handle,
                bindings,
            ),
            bindings,
        };

        // Automatically generate appearance stream for existing annotations
        // This is done in a way that doesn't fail the constructor if appearance generation fails
        let _ = annotation.auto_generate_appearance_stream();

        annotation
    }

    /// Sets the Default Appearance (DA) string for this free text annotation.
    ///
    /// The DA string specifies the font, size, and color for rendering the text.
    /// This is required for free text annotations to display correctly when flattened.
    ///
    /// # Format
    /// The DA string follows the PDF content stream format:
    /// `"/FontName fontSize Tf r g b rg"`
    ///
    /// Where:
    /// - `FontName` = PDF font name (e.g., "Helv" for Helvetica)
    /// - `fontSize` = Font size in points (e.g., "12")
    /// - `r g b` = RGB color values in range 0.0-1.0 (e.g., "0 0 0" for black)
    /// - `Tf` = Set text font operator
    /// - `rg` = Set fill color operator
    ///
    /// # Example
    /// ```
    /// // Helvetica, 12pt, black text
    /// annot.set_default_appearance("/Helv 12 Tf 0 0 0 rg")?;
    ///
    /// // Helvetica, 14pt, red text
    /// annot.set_default_appearance("/Helv 14 Tf 1 0 0 rg")?;
    /// ```
    ///
    /// # Errors
    /// Returns an error if PDFium fails to set the DA string.
    pub fn set_default_appearance(&mut self, da_string: &str) -> Result<(), crate::error::PdfiumError> {
        use crate::pdf::document::page::annotation::private::internal::PdfPageAnnotationPrivate;
        self.set_string_value("DA", da_string)
    }

    /// Returns the Default Appearance (DA) string for this free text annotation, if set.
    ///
    /// The DA string specifies how the text should be rendered (font, size, color).
    /// Returns `None` if the DA string has not been set.
    pub fn default_appearance(&self) -> Option<String> {
        use crate::pdf::document::page::annotation::private::internal::PdfPageAnnotationPrivate;
        self.get_string_value("DA")
    }

    /// Automatically generates the appearance stream for this free text annotation.
    ///
    /// This method uses the annotation's current text content, bounds, and DA string
    /// to generate a PDF content stream that renders the text with proper formatting,
    /// borders, and backgrounds. The appearance stream is required for the annotation
    /// to display correctly when the PDF is viewed or flattened.
    ///
    /// # Errors
    ///
    /// Returns an error if PDFium fails to generate or set the appearance stream.
    pub fn auto_generate_appearance_stream(&mut self) -> Result<(), PdfiumError> {
        // CRITICAL: Ensure annotation rect is set before building appearance stream.
        // PDFium requires a valid rect with minimum size before FPDFAnnot_SetAP will work.
        use crate::bindgen::FS_RECTF;
        let mut rect = FS_RECTF {
            left: 0.0,
            bottom: 0.0,
            right: 0.0,
            top: 0.0,
        };
        let get_rect_result = self.bindings.FPDFAnnot_GetRect(self.handle, &mut rect);

        if !self.bindings.is_true(get_rect_result) {
            // If rect is not set, set a default rect
            let default_rect = FS_RECTF {
                left: 0.0,
                bottom: 0.0,
                right: 200.0,
                top: 100.0,
            };
            let set_rect_result = self.bindings.FPDFAnnot_SetRect(self.handle, &default_rect);
            if !self.bindings.is_true(set_rect_result) {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&"❌ ERROR: Failed to set default rect for free text annotation".into());
                }
                return Err(PdfiumError::PdfiumLibraryInternalError(
                    crate::error::PdfiumInternalError::Unknown,
                ));
            }
        } else {
            // Ensure rect has valid dimensions
            let width = rect.right - rect.left;
            let height = rect.top - rect.bottom;
            if width < 1.0 || height < 1.0 {
                // Expand rect to minimum size
                rect.right = rect.left + 200.0;
                rect.top = rect.bottom + 100.0;
                let set_rect_result = self.bindings.FPDFAnnot_SetRect(self.handle, &rect);
                if !self.bindings.is_true(set_rect_result) {
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&"❌ ERROR: Failed to expand rect to minimum size for free text annotation".into());
                    }
                    return Err(PdfiumError::PdfiumLibraryInternalError(
                        crate::error::PdfiumInternalError::Unknown,
                    ));
                }
            }
        }

        let text = self.contents().unwrap_or_default();
        let da_string = self.default_appearance();

        let builder = FreeTextAppearanceBuilder::new(
            self.handle,
            self.bindings,
            text,
            da_string,
        );

        builder.apply()
    }

    /// Regenerates the appearance stream for this free text annotation.
    ///
    /// This is a public method that can be called explicitly to regenerate the
    /// appearance stream after making changes to the annotation's properties.
    /// Normally, appearance streams are regenerated automatically when properties
    /// change, but this method allows manual regeneration if needed.
    ///
    /// # Errors
    ///
    /// Returns an error if PDFium fails to generate or set the appearance stream.
    pub fn regenerate_appearance(&mut self) -> Result<(), PdfiumError> {
        self.auto_generate_appearance_stream()
    }

    /// Returns a builder for customizing the appearance of this free text annotation.
    ///
    /// The builder allows fine-grained control over text formatting, alignment,
    /// borders, backgrounds, and other visual properties. Call `.apply()` on the
    /// builder to set the appearance stream.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// annotation.set_appearance()
    ///     .with_font_size(14.0)
    ///     .with_text_color(PdfColor::RED)
    ///     .with_horizontal_alignment(TextAlignment::Center)
    ///     .with_border(1.0, PdfColor::BLACK)
    ///     .with_background(PdfColor::new(240, 240, 240))
    ///     .apply()?;
    /// ```
    pub fn set_appearance(&mut self) -> FreeTextAppearanceBuilder<'_> {
        let text = self.contents().unwrap_or_default();
        let da_string = self.default_appearance();

        FreeTextAppearanceBuilder::new(
            self.handle,
            self.bindings,
            text,
            da_string,
        )
    }
}

impl<'a> PdfPageAnnotationPrivate<'a> for PdfPageFreeTextAnnotation<'a> {
    #[inline]
    fn handle(&self) -> FPDF_ANNOTATION {
        self.handle
    }

    #[inline]
    fn ownership(&self) -> &PdfPageObjectOwnership {
        self.objects_impl().ownership()
    }

    #[inline]
    fn bindings(&self) -> &dyn PdfiumLibraryBindings {
        self.bindings
    }

    #[inline]
    fn objects_impl(&self) -> &PdfPageAnnotationObjects<'_> {
        &self.objects
    }

    #[inline]
    fn attachment_points_impl(&self) -> &PdfPageAnnotationAttachmentPoints<'_> {
        &self.attachment_points
    }

    /// Override set_contents_impl to automatically regenerate appearance stream
    fn set_contents_impl(&mut self, contents: &str) -> Result<(), PdfiumError> {
        // Call the parent implementation first
        self.set_string_value("Contents", contents)?;

        // Automatically regenerate appearance stream after content change
        let _ = self.auto_generate_appearance_stream();

        Ok(())
    }

    /// Override set_bounds_impl to automatically regenerate appearance stream
    fn set_bounds_impl(&mut self, bounds: crate::pdf::rect::PdfRect) -> Result<(), PdfiumError> {
        // Call the parent implementation first
        if self.bindings().is_true(
            self.bindings()
                .FPDFAnnot_SetRect(self.handle(), &bounds.as_pdfium()),
        ) {
            self.set_string_value("M", &crate::utils::dates::date_time_to_pdf_string(chrono::Utc::now()))?;

            // Automatically regenerate appearance stream after bounds change
            let _ = self.auto_generate_appearance_stream();

            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                crate::error::PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Override set_string_value to automatically regenerate appearance stream for DA changes
    fn set_string_value(&mut self, key: &str, value: &str) -> Result<(), PdfiumError> {
        // Call the parent implementation first
        if self
            .bindings()
            .is_true(
                self.bindings()
                    .FPDFAnnot_SetStringValue_str(self.handle(), key, value),
            )
        {
            // If this is a DA (Default Appearance) string change, regenerate appearance stream
            if key == "DA" {
                let _ = self.auto_generate_appearance_stream();
            }

            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                crate::error::PdfiumInternalError::Unknown,
            ))
        }
    }
}
