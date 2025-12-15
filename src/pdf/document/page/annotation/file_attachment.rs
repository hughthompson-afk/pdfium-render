//! Defines the [PdfPageFileAttachmentAnnotation] struct, exposing functionality related to a single
//! user annotation of type [PdfPageAnnotationType::FileAttachment].

use crate::bindgen::{FPDF_ANNOTATION, FPDF_DOCUMENT, FPDF_PAGE};
use crate::bindings::PdfiumLibraryBindings;
use crate::error::{PdfiumError, PdfiumInternalError};
use crate::pdf::document::attachment::PdfAttachment;
use crate::pdf::document::page::annotation::attachment_points::PdfPageAnnotationAttachmentPoints;
use crate::pdf::document::page::annotation::objects::PdfPageAnnotationObjects;
use crate::pdf::document::page::annotation::private::internal::PdfPageAnnotationPrivate;
use crate::pdf::document::page::object::ownership::PdfPageObjectOwnership;
use crate::pdf::document::page::objects::private::internal::PdfPageObjectsPrivate;

#[cfg(doc)]
use crate::pdf::document::page::annotation::{PdfPageAnnotation, PdfPageAnnotationType};

/// A single [PdfPageAnnotation] of type [PdfPageAnnotationType::FileAttachment].
pub struct PdfPageFileAttachmentAnnotation<'a> {
    handle: FPDF_ANNOTATION,
    objects: PdfPageAnnotationObjects<'a>,
    attachment_points: PdfPageAnnotationAttachmentPoints<'a>,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfPageFileAttachmentAnnotation<'a> {
    pub(crate) fn from_pdfium(
        document_handle: FPDF_DOCUMENT,
        page_handle: FPDF_PAGE,
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfPageFileAttachmentAnnotation {
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
        }
    }

    /// Returns the [PdfAttachment] handle associated with this file attachment annotation,
    /// if any.
    ///
    /// Returns `None` if the annotation has no file attachment associated with it.
    pub fn get_file_attachment(&self) -> Option<PdfAttachment<'a>> {
        let attachment_handle = self.bindings.FPDFAnnot_GetFileAttachment(self.handle);

        if attachment_handle.is_null() {
            None
        } else {
            Some(PdfAttachment::from_pdfium(attachment_handle, self.bindings))
        }
    }

    /// Adds a new file attachment to this annotation with the given name.
    ///
    /// Returns the newly created [PdfAttachment] handle, or an error if the operation failed.
    pub fn add_file_attachment(&mut self, name: &str) -> Result<PdfAttachment<'a>, PdfiumError> {
        let attachment_handle = self.bindings.FPDFAnnot_AddFileAttachment_str(self.handle, name);

        if attachment_handle.is_null() {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        } else {
            Ok(PdfAttachment::from_pdfium(attachment_handle, self.bindings))
        }
    }

    /// Returns the name of the file attachment associated with this annotation, if any.
    pub fn attachment_name(&self) -> Option<String> {
        self.get_file_attachment().map(|attachment| attachment.name())
    }
}

impl<'a> PdfPageAnnotationPrivate<'a> for PdfPageFileAttachmentAnnotation<'a> {
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
}

