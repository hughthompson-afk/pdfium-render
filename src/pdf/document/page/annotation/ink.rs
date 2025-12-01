//! Defines the [PdfPageInkAnnotation] struct, exposing functionality related to a single
//! user annotation of type [PdfPageAnnotationType::Ink].

use crate::bindgen::{FPDF_ANNOTATION, FPDF_DOCUMENT, FPDF_PAGE, FS_POINTF};
use crate::bindings::PdfiumLibraryBindings;
use crate::error::{PdfiumError, PdfiumInternalError};
use crate::pdf::document::page::annotation::attachment_points::PdfPageAnnotationAttachmentPoints;
use crate::pdf::document::page::annotation::objects::PdfPageAnnotationObjects;
use crate::pdf::document::page::annotation::private::internal::PdfPageAnnotationPrivate;
use crate::pdf::document::page::object::ownership::PdfPageObjectOwnership;
use crate::pdf::document::page::objects::private::internal::PdfPageObjectsPrivate;
use crate::pdf::points::PdfPoints;
use std::os::raw::c_ulong;

#[cfg(doc)]
use crate::pdf::document::page::annotation::{PdfPageAnnotation, PdfPageAnnotationType};

/// A single point in an ink stroke, representing x and y coordinates in page space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PdfInkStrokePoint {
    pub x: PdfPoints,
    pub y: PdfPoints,
}

impl PdfInkStrokePoint {
    /// Creates a new [PdfInkStrokePoint] with the given coordinates.
    #[inline]
    pub fn new(x: PdfPoints, y: PdfPoints) -> Self {
        Self { x, y }
    }

    /// Creates a new [PdfInkStrokePoint] from raw f32 values.
    #[inline]
    pub fn from_values(x: f32, y: f32) -> Self {
        Self {
            x: PdfPoints::new(x),
            y: PdfPoints::new(y),
        }
    }
}

/// A single [PdfPageAnnotation] of type [PdfPageAnnotationType::Ink].
pub struct PdfPageInkAnnotation<'a> {
    handle: FPDF_ANNOTATION,
    objects: PdfPageAnnotationObjects<'a>,
    attachment_points: PdfPageAnnotationAttachmentPoints<'a>,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfPageInkAnnotation<'a> {
    pub(crate) fn from_pdfium(
        document_handle: FPDF_DOCUMENT,
        page_handle: FPDF_PAGE,
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfPageInkAnnotation {
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

    /// Returns a mutable collection of all the page objects in this [PdfPageInkAnnotation].
    #[inline]
    pub fn objects_mut(&mut self) -> &mut PdfPageAnnotationObjects<'a> {
        &mut self.objects
    }

    /// Adds a new ink stroke to this [PdfPageInkAnnotation]'s InkList.
    ///
    /// The InkList is required for ink annotations to render properly in compliant PDF readers.
    /// This method creates an InkList if one doesn't already exist.
    ///
    /// Returns the 0-based index at which the new stroke was added in the InkList,
    /// or an error if the operation failed.
    ///
    /// # Example
    /// ```ignore
    /// // Create a simple stroke from (100, 100) to (200, 200) to (300, 100)
    /// let points = vec![
    ///     PdfInkStrokePoint::from_values(100.0, 100.0),
    ///     PdfInkStrokePoint::from_values(200.0, 200.0),
    ///     PdfInkStrokePoint::from_values(300.0, 100.0),
    /// ];
    ///
    /// let stroke_index = ink_annotation.add_ink_stroke(&points)?;
    /// ```
    pub fn add_ink_stroke(&mut self, points: &[PdfInkStrokePoint]) -> Result<usize, PdfiumError> {
        if points.is_empty() {
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        // Convert PdfInkStrokePoint to FS_POINTF array
        let fs_points: Vec<FS_POINTF> = points
            .iter()
            .map(|p| FS_POINTF {
                x: p.x.value,
                y: p.y.value,
            })
            .collect();

        let result = self.bindings.FPDFAnnot_AddInkStroke(
            self.handle,
            fs_points.as_ptr(),
            fs_points.len(),
        );

        if result == -1 {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        } else {
            Ok(result as usize)
        }
    }

    /// Removes all ink strokes from this [PdfPageInkAnnotation]'s InkList.
    ///
    /// Returns `Ok(())` on success, or an error if the operation failed.
    pub fn remove_ink_list(&mut self) -> Result<(), PdfiumError> {
        if self
            .bindings
            .is_true(self.bindings.FPDFAnnot_RemoveInkList(self.handle))
        {
            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Returns the number of ink strokes in this [PdfPageInkAnnotation]'s InkList.
    pub fn ink_stroke_count(&self) -> usize {
        self.bindings.FPDFAnnot_GetInkListCount(self.handle) as usize
    }

    /// Returns the points of the ink stroke at the given index.
    ///
    /// Returns `None` if the index is out of bounds or the annotation has no InkList.
    pub fn get_ink_stroke(&self, stroke_index: usize) -> Option<Vec<PdfInkStrokePoint>> {
        // First, get the number of points in this stroke by calling with length 0
        let point_count = self.bindings.FPDFAnnot_GetInkListPath(
            self.handle,
            stroke_index as c_ulong,
            std::ptr::null_mut(),
            0,
        );

        if point_count == 0 {
            return None;
        }

        // Allocate buffer and retrieve points
        let mut buffer: Vec<FS_POINTF> = vec![FS_POINTF { x: 0.0, y: 0.0 }; point_count as usize];

        let retrieved = self.bindings.FPDFAnnot_GetInkListPath(
            self.handle,
            stroke_index as c_ulong,
            buffer.as_mut_ptr(),
            point_count,
        );

        if retrieved == 0 {
            return None;
        }

        // Convert FS_POINTF to PdfInkStrokePoint
        Some(
            buffer
                .into_iter()
                .take(retrieved as usize)
                .map(|p| PdfInkStrokePoint {
                    x: PdfPoints::new(p.x),
                    y: PdfPoints::new(p.y),
                })
                .collect(),
        )
    }

    /// Returns an iterator over all ink strokes in this [PdfPageInkAnnotation].
    pub fn ink_strokes(&self) -> impl Iterator<Item = Vec<PdfInkStrokePoint>> + '_ {
        (0..self.ink_stroke_count()).filter_map(move |i| self.get_ink_stroke(i))
    }
}

impl<'a> PdfPageAnnotationPrivate<'a> for PdfPageInkAnnotation<'a> {
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
