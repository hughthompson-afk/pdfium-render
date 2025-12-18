//! Defines the [PdfPageAnnotations] struct, exposing functionality related to the
//! annotations that have been added to a single `PdfPage`.

use crate::bindgen::{FPDF_ANNOTATION, FPDF_DOCUMENT, FPDF_FORMHANDLE, FPDF_PAGE, FS_RECTF, FPDF_WCHAR};
use crate::bindings::PdfiumLibraryBindings;
use crate::error::{PdfiumError, PdfiumInternalError};
#[cfg(target_arch = "wasm32")]
use js_sys;
use crate::pdf::color::PdfColor;
use crate::pdf::document::page::annotation::caret::PdfPageCaretAnnotation;
use crate::pdf::document::page::annotation::circle::PdfPageCircleAnnotation;
use crate::pdf::document::page::annotation::file_attachment::PdfPageFileAttachmentAnnotation;
use crate::pdf::document::page::annotation::free_text::PdfPageFreeTextAnnotation;
use crate::pdf::document::page::annotation::highlight::PdfPageHighlightAnnotation;
use crate::pdf::document::page::annotation::ink::PdfPageInkAnnotation;
use crate::pdf::document::page::annotation::line::PdfPageLineAnnotation;
use crate::pdf::document::page::annotation::link::PdfPageLinkAnnotation;
use crate::pdf::document::page::annotation::popup::PdfPagePopupAnnotation;
use crate::pdf::document::page::annotation::polygon::PdfPagePolygonAnnotation;
use crate::pdf::document::page::annotation::polyline::PdfPagePolylineAnnotation;
use crate::pdf::document::page::annotation::private::internal::PdfPageAnnotationPrivate;
use crate::pdf::document::page::annotation::square::PdfPageSquareAnnotation;
use crate::pdf::document::page::annotation::squiggly::PdfPageSquigglyAnnotation;
use crate::pdf::document::page::annotation::stamp::PdfPageStampAnnotation;
use crate::pdf::document::page::annotation::strikeout::PdfPageStrikeoutAnnotation;
use crate::pdf::document::page::annotation::text::PdfPageTextAnnotation;
use crate::pdf::document::page::annotation::underline::PdfPageUnderlineAnnotation;
use crate::pdf::document::page::annotation::watermark::PdfPageWatermarkAnnotation;
#[cfg(feature = "pdfium_future")]
use crate::pdf::document::page::annotation::widget::PdfPageWidgetAnnotation;
use crate::pdf::document::page::annotation::{
    PdfPageAnnotation, PdfPageAnnotationCommon, PdfPageAnnotationType,
};
#[cfg(feature = "pdfium_future")]
use crate::pdf::document::page::field::PdfFormFieldType;
#[cfg(feature = "pdfium_future")]
use crate::pdf::document::page::field::private::internal::PdfFormFieldFlags;
#[cfg(feature = "pdfium_future")]
use crate::pdf::rect::PdfRect;
#[cfg(feature = "pdfium_future")]
use std::ffi::CString;
use crate::pdf::document::page::object::{PdfPageObject, PdfPageObjectCommon};
use crate::pdf::document::page::{PdfPage, PdfPageContentRegenerationStrategy, PdfPageIndexCache};
use crate::pdf::quad_points::PdfQuadPoints;
use crate::pdf::appearance_mode::PdfAppearanceMode;
use chrono::prelude::*;
use std::ops::Range;
use std::os::raw::c_int;

/// The zero-based index of a single [PdfPageAnnotation] inside its containing
/// [PdfPageAnnotations] collection.
pub type PdfPageAnnotationIndex = usize;

/// The annotations that have been added to a single `PdfPage`.
pub struct PdfPageAnnotations<'a> {
    document_handle: FPDF_DOCUMENT,
    page_handle: FPDF_PAGE,
    form_handle: Option<FPDF_FORMHANDLE>,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfPageAnnotations<'a> {
    #[inline]
    pub(crate) fn from_pdfium(
        document_handle: FPDF_DOCUMENT,
        page_handle: FPDF_PAGE,
        form_handle: Option<FPDF_FORMHANDLE>,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfPageAnnotations {
            document_handle,
            page_handle,
            form_handle,
            bindings,
        }
    }

    /// Returns the internal `FPDF_DOCUMENT` handle of the [PdfDocument] containing this
    /// [PdfPageAnnotations] collection.
    #[inline]
    pub(crate) fn document_handle(&self) -> FPDF_DOCUMENT {
        self.document_handle
    }

    /// Returns the internal `FPDF_PAGE` handle of the [PdfPage] containing this
    /// [PdfPageAnnotations] collection.
    #[inline]
    pub(crate) fn page_handle(&self) -> FPDF_PAGE {
        self.page_handle
    }

    /// Returns the [PdfiumLibraryBindings] used by this [PdfPageAnnotations] collection.
    #[inline]
    pub fn bindings(&self) -> &'a dyn PdfiumLibraryBindings {
        self.bindings
    }


    /// Returns the total number of annotations that have been added to the containing `PdfPage`.
    #[inline]
    pub fn len(&self) -> PdfPageAnnotationIndex {
        self.bindings().FPDFPage_GetAnnotCount(self.page_handle) as PdfPageAnnotationIndex
    }

    /// Returns true if this [PdfPageAnnotations] collection is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a Range from 0..(number of annotations) for this [PdfPageAnnotations] collection.
    #[inline]
    pub fn as_range(&self) -> Range<PdfPageAnnotationIndex> {
        0..self.len()
    }

    /// Returns a single [PdfPageAnnotation] from this [PdfPageAnnotations] collection.
    pub fn get(&self, index: PdfPageAnnotationIndex) -> Result<PdfPageAnnotation<'a>, PdfiumError> {
        if index >= self.len() {
            return Err(PdfiumError::PageAnnotationIndexOutOfBounds);
        }

        let annotation_handle = self
            .bindings()
            .FPDFPage_GetAnnot(self.page_handle, index as c_int);

        if annotation_handle.is_null() {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        } else {
            Ok(PdfPageAnnotation::from_pdfium(
                self.document_handle,
                self.page_handle,
                annotation_handle,
                self.form_handle,
                self.bindings,
            ))
        }
    }

    /// Returns the first [PdfPageAnnotation] in this [PdfPageAnnotations] collection.
    #[inline]
    pub fn first(&self) -> Result<PdfPageAnnotation<'a>, PdfiumError> {
        if !self.is_empty() {
            self.get(0)
        } else {
            Err(PdfiumError::NoAnnotationsInCollection)
        }
    }

    /// Returns the last [PdfPageAnnotation] in this [PdfPageAnnotations] collection.
    #[inline]
    pub fn last(&self) -> Result<PdfPageAnnotation<'a>, PdfiumError> {
        if !self.is_empty() {
            self.get(self.len() - 1)
        } else {
            Err(PdfiumError::NoAnnotationsInCollection)
        }
    }

    /// Returns an iterator over all the annotations in this [PdfPageAnnotations] collection.
    #[inline]
    pub fn iter(&self) -> PdfPageAnnotationsIterator<'_> {
        PdfPageAnnotationsIterator::new(self)
    }

    /// Creates a new annotation of the given [PdfPageAnnotationType] by passing the result of calling
    /// `FPDFPage_CreateAnnot()` to an annotation constructor function.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    pub(crate) fn create_annotation<T: PdfPageAnnotationCommon>(
        &mut self,
        annotation_type: PdfPageAnnotationType,
        constructor: fn(
            FPDF_DOCUMENT,
            FPDF_PAGE,
            FPDF_ANNOTATION,
            &'a dyn PdfiumLibraryBindings,
        ) -> T,
    ) -> Result<T, PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&format!("🔧 create_annotation() - Creating {:?} annotation", annotation_type).into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&format!("   Page handle: {:?}", self.page_handle()).into());
            console::log_1(&format!("   Document handle: {:?}", self.document_handle()).into());
            console::log_1(&format!("   Annotation type PDFium value: {}", annotation_type.as_pdfium()).into());
        }

        // Check if this annotation type is supported for creation
        let is_supported = self
            .bindings()
            .FPDFAnnot_IsSupportedSubtype(annotation_type.as_pdfium());

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   FPDFAnnot_IsSupportedSubtype returned: {} (1=supported, 0=not supported)", is_supported).into());
            console::log_1(&format!("   is_true(is_supported): {}", self.bindings().is_true(is_supported)).into());
        }

        if !self.bindings().is_true(is_supported) {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"❌ ERROR: Annotation type is NOT supported for creation by PDFium".into());
                console::log_1(&format!("   {:?} annotations cannot be created via FPDFPage_CreateAnnot", annotation_type).into());
                console::log_1(&"   This annotation type may need to be created differently or is not supported in this PDFium version".into());
                console::log_1(&"═══════════════════════════════════════════════════════════".into());
            }
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        let handle = self
            .bindings()
            .FPDFPage_CreateAnnot(self.page_handle(), annotation_type.as_pdfium());

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   FPDFPage_CreateAnnot returned handle: {:?}", handle).into());
            console::log_1(&format!("   Handle is null: {}", handle.is_null()).into());
        }

        if handle.is_null() {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"❌ ERROR: FPDFPage_CreateAnnot returned NULL handle".into());
                console::log_1(&format!("   This means PDFium failed to create the {:?} annotation", annotation_type).into());
            }
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"✅ FPDFPage_CreateAnnot succeeded, calling constructor".into());
            }

            let mut annotation = constructor(
                self.document_handle(),
                self.page_handle(),
                handle,
                self.bindings(),
            );

            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"✅ Constructor succeeded, setting creation date".into());
            }

            let creation_date_result = annotation.set_creation_date(Utc::now());

            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                match &creation_date_result {
                    Ok(_) => console::log_1(&"✅ set_creation_date succeeded".into()),
                    Err(e) => console::log_1(&format!("❌ set_creation_date failed: {:?}", e).into()),
                }
            }

            let content_regeneration_result = creation_date_result.and_then(|()| {
                if let Some(content_regeneration_strategy) =
                    PdfPageIndexCache::get_content_regeneration_strategy_for_page(
                        self.document_handle(),
                        self.page_handle(),
                    )
                {
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&format!("   Content regeneration strategy: {:?}", content_regeneration_strategy).into());
                    }

                    if content_regeneration_strategy
                        == PdfPageContentRegenerationStrategy::AutomaticOnEveryChange
                    {
                        #[cfg(target_arch = "wasm32")]
                        {
                            use web_sys::console;
                            console::log_1(&"   Triggering content regeneration".into());
                        }
                        PdfPage::regenerate_content_immut_for_handle(
                            self.page_handle(),
                            self.bindings(),
                        )
                    } else {
                        Ok(())
                    }
                } else {
                    #[cfg(target_arch = "wasm32")]
                    {
                        use web_sys::console;
                        console::log_1(&"❌ ERROR: SourcePageIndexNotInCache".into());
                    }
                    Err(PdfiumError::SourcePageIndexNotInCache)
                }
            });

            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                match &content_regeneration_result {
                    Ok(_) => console::log_1(&"✅ Annotation creation completed successfully".into()),
                    Err(e) => console::log_1(&format!("❌ Content regeneration failed: {:?}", e).into()),
                }
                console::log_1(&"═══════════════════════════════════════════════════════════".into());
            }

            content_regeneration_result.map(|()| annotation)
        }
    }

    /// Creates a new [PdfPageFreeTextAnnotation] containing the given text in this
    /// [PdfPageAnnotations] collection, returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_free_text_annotation(
        &mut self,
        text: &str,
    ) -> Result<PdfPageFreeTextAnnotation<'a>, PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&"🔧 create_free_text_annotation()".into());
            console::log_1(&format!("   Text: '{}'", text).into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
        }

        let mut annotation = self.create_annotation(
            PdfPageAnnotationType::FreeText,
            PdfPageFreeTextAnnotation::from_pdfium,
        )?;

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"✅ Annotation created, setting contents".into());
        }

        let set_contents_result = annotation.set_contents(text);

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            match &set_contents_result {
                Ok(_) => console::log_1(&"✅ set_contents succeeded".into()),
                Err(e) => console::log_1(&format!("❌ set_contents failed: {:?}", e).into()),
            }
        }

        set_contents_result?;

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"   Generating appearance stream".into());
        }

        // Automatically generate appearance stream
        let appearance_result = annotation.auto_generate_appearance_stream();

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            match &appearance_result {
                Ok(_) => console::log_1(&"✅ auto_generate_appearance_stream succeeded".into()),
                Err(e) => console::log_1(&format!("❌ auto_generate_appearance_stream failed: {:?}", e).into()),
            }
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
        }

        appearance_result?;

        Ok(annotation)
    }

    /// Creates a new [PdfPageHighlightAnnotation] annotation in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_highlight_annotation(
        &mut self,
    ) -> Result<PdfPageHighlightAnnotation<'a>, PdfiumError> {
        let mut annotation = self.create_annotation(
            PdfPageAnnotationType::Highlight,
            PdfPageHighlightAnnotation::from_pdfium,
        )?;

        // Generate appearance stream for flattening support
        let _ = annotation.generate_appearance_stream();

        Ok(annotation)
    }

    /// Creates a new [PdfPageInkAnnotation] in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_ink_annotation(&mut self) -> Result<PdfPageInkAnnotation<'a>, PdfiumError> {
        self.create_annotation(
            PdfPageAnnotationType::Ink,
            PdfPageInkAnnotation::from_pdfium,
        )
    }

    /// Creates a new [PdfPageLinkAnnotation] with the given URI in this [PdfPageAnnotations]
    /// collection, returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    pub fn create_link_annotation(
        &mut self,
        uri: &str,
    ) -> Result<PdfPageLinkAnnotation<'a>, PdfiumError> {
        let mut annotation = self.create_annotation(
            PdfPageAnnotationType::Link,
            PdfPageLinkAnnotation::from_pdfium,
        )?;

        annotation.set_link(uri)?;

        Ok(annotation)
    }

    /// Creates a new [PdfPagePopupAnnotation] annotation in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_popup_annotation(&mut self) -> Result<PdfPagePopupAnnotation<'a>, PdfiumError> {
        self.create_annotation(
            PdfPageAnnotationType::Popup,
            PdfPagePopupAnnotation::from_pdfium,
        )
    }

    /// Creates a new [PdfPageSquareAnnotation] annotation in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_square_annotation(&mut self) -> Result<PdfPageSquareAnnotation<'a>, PdfiumError> {
        self.create_annotation(
            PdfPageAnnotationType::Square,
            PdfPageSquareAnnotation::from_pdfium,
        )
    }

    /// Creates a new [PdfPageCircleAnnotation] annotation in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_circle_annotation(&mut self) -> Result<PdfPageCircleAnnotation<'a>, PdfiumError> {
        self.create_annotation(
            PdfPageAnnotationType::Circle,
            PdfPageCircleAnnotation::from_pdfium,
        )
    }

    /// Creates a new [PdfPageSquigglyAnnotation] annotation in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_squiggly_annotation(
        &mut self,
    ) -> Result<PdfPageSquigglyAnnotation<'a>, PdfiumError> {
        let mut annotation = self.create_annotation(
            PdfPageAnnotationType::Squiggly,
            PdfPageSquigglyAnnotation::from_pdfium,
        )?;

        // Generate appearance stream for flattening support
        let _ = annotation.generate_appearance_stream();

        Ok(annotation)
    }

    /// Creates a new [PdfPageStampAnnotation] annotation in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_stamp_annotation(&mut self) -> Result<PdfPageStampAnnotation<'a>, PdfiumError> {
        let mut annotation = self.create_annotation(
            PdfPageAnnotationType::Stamp,
            PdfPageStampAnnotation::from_pdfium,
        )?;

        // Stamp annotations in PDFium often have a white background by default.
        // We set the background to transparent to ensure that any images or text
        // with transparency are rendered correctly.
        let _ = annotation.set_fill_color(PdfColor::new(0, 0, 0, 0));
        let _ = annotation.set_stroke_color(PdfColor::new(0, 0, 0, 0));

        Ok(annotation)
    }

    /// Creates a new [PdfPageStrikeoutAnnotation] annotation in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_strikeout_annotation(
        &mut self,
    ) -> Result<PdfPageStrikeoutAnnotation<'a>, PdfiumError> {
        let mut annotation = self.create_annotation(
            PdfPageAnnotationType::Strikeout,
            PdfPageStrikeoutAnnotation::from_pdfium,
        )?;

        // Generate appearance stream for flattening support
        let _ = annotation.generate_appearance_stream();

        Ok(annotation)
    }

    /// Creates a new [PdfPageTextAnnotation] containing the given text in this [PdfPageAnnotations]
    /// collection, returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_text_annotation(
        &mut self,
        text: &str,
    ) -> Result<PdfPageTextAnnotation<'a>, PdfiumError> {
        let mut annotation = self.create_annotation(
            PdfPageAnnotationType::Text,
            PdfPageTextAnnotation::from_pdfium,
        )?;

        annotation.set_contents(text)?;

        Ok(annotation)
    }

    /// Creates a new [PdfPageUnderlineAnnotation] annotation in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_underline_annotation(
        &mut self,
    ) -> Result<PdfPageUnderlineAnnotation<'a>, PdfiumError> {
        let mut annotation = self.create_annotation(
            PdfPageAnnotationType::Underline,
            PdfPageUnderlineAnnotation::from_pdfium,
        )?;

        // Generate appearance stream for flattening support
        let _ = annotation.generate_appearance_stream();

        Ok(annotation)
    }

    /// Creates a new [PdfPageCaretAnnotation] annotation in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_caret_annotation(&mut self) -> Result<PdfPageCaretAnnotation<'a>, PdfiumError> {
        self.create_annotation(
            PdfPageAnnotationType::Caret,
            PdfPageCaretAnnotation::from_pdfium,
        )
    }

    /// Creates a new [PdfPageFileAttachmentAnnotation] annotation in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_file_attachment_annotation(
        &mut self,
    ) -> Result<PdfPageFileAttachmentAnnotation<'a>, PdfiumError> {
        self.create_annotation(
            PdfPageAnnotationType::FileAttachment,
            PdfPageFileAttachmentAnnotation::from_pdfium,
        )
    }

    /// Creates a new [PdfPageLineAnnotation] annotation in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_line_annotation(&mut self) -> Result<PdfPageLineAnnotation<'a>, PdfiumError> {
        self.create_annotation(
            PdfPageAnnotationType::Line,
            PdfPageLineAnnotation::from_pdfium,
        )
    }

    /// Creates a new [PdfPagePolygonAnnotation] annotation in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_polygon_annotation(
        &mut self,
    ) -> Result<PdfPagePolygonAnnotation<'a>, PdfiumError> {
        self.create_annotation(
            PdfPageAnnotationType::Polygon,
            PdfPagePolygonAnnotation::from_pdfium,
        )
    }

    /// Creates a new [PdfPagePolylineAnnotation] annotation in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_polyline_annotation(
        &mut self,
    ) -> Result<PdfPagePolylineAnnotation<'a>, PdfiumError> {
        self.create_annotation(
            PdfPageAnnotationType::Polyline,
            PdfPagePolylineAnnotation::from_pdfium,
        )
    }

    /// Creates a new [PdfPageWatermarkAnnotation] annotation in this [PdfPageAnnotations] collection,
    /// returning the newly created annotation.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_watermark_annotation(
        &mut self,
    ) -> Result<PdfPageWatermarkAnnotation<'a>, PdfiumError> {
        self.create_annotation(
            PdfPageAnnotationType::Watermark,
            PdfPageWatermarkAnnotation::from_pdfium,
        )
    }

    /// Creates a new [PdfPageWidgetAnnotation] (form field annotation) in this [PdfPageAnnotations]
    /// collection, returning the newly created annotation.
    ///
    /// This creates both the form field dictionary and the widget annotation, properly linking
    /// them together. The widget annotation represents an interactive form field (text field,
    /// checkbox, radio button, etc.).
    ///
    /// # Arguments
    ///
    /// * `form_handle` - Handle to the form fill module (required).
    /// * `field_name` - The field name (e.g., "MyTextField").
    /// * `field_type` - The type of form field to create.
    /// * `rect` - Bounding rectangle for the widget annotation.
    /// * `options` - Optional list of options for ComboBox/ListBox fields.
    /// * `max_length` - Optional maximum length for text fields.
    /// * `quadding` - Optional text alignment (0=left, 1=center, 2=right).
    /// * `default_appearance` - Optional default appearance string.
    /// * `default_value` - Optional default value.
    /// * `additional_flags` - Optional additional form field flags to set. These are combined
    ///   with the required flags for the field type. Common flags include:
    ///   - Common: READONLY (1), REQUIRED (2), NOEXPORT (4)
    ///   - Text: MULTILINE (1<<12), PASSWORD (1<<13)
    ///   - Button: NOTOGGLETOOFF (1<<14), RADIO (1<<15), PUSHBUTTON (1<<16), RADIOSINUNISON (1<<25)
    ///   - Choice: COMBO (1<<17), EDIT (1<<18), MULTI_SELECT (1<<21)
    ///
    /// # Returns
    ///
    /// Returns `Ok(PdfPageWidgetAnnotation)` if successful, or an error if the operation fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use pdfium_render::prelude::*;
    ///
    /// let pdfium = Pdfium::new();
    /// let mut document = pdfium.create_new_pdf().unwrap();
    /// document.ensure_acro_form().unwrap();
    ///
    /// let mut page = document.pages_mut().create_page_at_start(
    ///     PdfPagePaperSize::a4().width,
    ///     PdfPagePaperSize::a4().height,
    /// ).unwrap();
    ///
    /// let form_handle = page.form_handle().unwrap();
    /// let rect = PdfRect::new(100.0, 700.0, 200.0, 720.0);
    ///
    /// // Create a required, multiline text field
    /// let flags = Some(2 | (1 << 12)); // REQUIRED | MULTILINE
    /// let widget = page.annotations_mut().create_widget_annotation(
    ///     form_handle,
    ///     "MyTextField",
    ///     PdfFormFieldType::Text,
    ///     rect,
    ///     None, None, None, None, None,
    ///     flags,
    /// ).unwrap();
    /// ```
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[cfg(feature = "pdfium_future")]
    pub fn create_widget_annotation(
        &mut self,
        form_handle: FPDF_FORMHANDLE,
        field_name: &str,
        field_type: PdfFormFieldType,
        rect: PdfRect,
        options: Option<&[&str]>,
        max_length: Option<i32>,
        quadding: Option<i32>,
        default_appearance: Option<&str>,
        default_value: Option<&str>,
        additional_flags: Option<u32>,
    ) -> Result<PdfPageWidgetAnnotation<'a>, PdfiumError> {
        use crate::utils::utf16le::get_pdfium_utf16le_bytes_from_str;
        use std::os::raw::c_ushort;

        // Validate that field_type is not Unknown
        if field_type == PdfFormFieldType::Unknown {
            return Err(PdfiumError::UnknownFormFieldType);
        }

        // Convert field_name to C string
        let field_name_cstr = CString::new(field_name)
            .map_err(|_| PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))?;

        // Get base type string and convert to C string
        let base_type_str = field_type.to_base_type_string()?;
        let field_type_cstr = CString::new(base_type_str)
            .map_err(|_| PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))?;

        // Convert rect to FS_RECTF
        let fs_rect = self.bindings().get_fs_rect_from_rect(&rect);

        // Handle options array if provided
        // For WASM, we need to pass the bytes directly, not pointers to Rust memory
        // Store the bytes in WASM state so bindings can copy them to PDFium memory
        let (options_ptr, option_count, _option_bytes, _option_ptrs) = if let Some(opts) = options {
            // Convert each option string to UTF-16LE bytes
            let option_bytes: Vec<Vec<u8>> = opts
                .iter()
                .map(|s| get_pdfium_utf16le_bytes_from_str(s))
                .collect();

            let option_count = option_bytes.len();
            
            // Store the bytes in WASM state for the bindings to access
            #[cfg(target_arch = "wasm32")]
            {
                use crate::bindings::wasm::PdfiumRenderWasmState;
                use js_sys::Array as JsArray;
                // Get write access to store the bytes
                let mut state = PdfiumRenderWasmState::lock_mut();
                // Convert Vec<Vec<u8>> to JavaScript Array of Uint8Arrays
                let js_array = JsArray::new();
                for bytes_vec in &option_bytes {
                    let uint8_array = js_sys::Uint8Array::new_with_length(bytes_vec.len() as u32);
                    uint8_array.copy_from(bytes_vec);
                    js_array.push(&uint8_array.into());
                }
                // Store the array in state using the public set method
                state.set("__widget_annotation_options", js_array.into());
            }
            
            // Create dummy pointers - the WASM bindings will use the stored bytes instead
            let option_ptrs: Vec<*const c_ushort> = vec![std::ptr::null(); option_count];
            let options_ptr = option_ptrs.as_ptr();

            (options_ptr, option_count, Some(option_bytes), Some(option_ptrs))
        } else {
            (std::ptr::null(), 0, None, None)
        };

        // Handle max_length
        let max_length_val = max_length.unwrap_or(-1);

        // Handle quadding
        let quadding_val = quadding.unwrap_or(-1);

        // Handle default_appearance
        let default_appearance_cstr = if let Some(appearance) = default_appearance {
            Some(CString::new(appearance)
                .map_err(|_| PdfiumError::PdfiumLibraryInternalError(
                    PdfiumInternalError::Unknown,
                ))?)
        } else {
            None
        };
        let default_appearance_ptr = default_appearance_cstr
            .as_ref()
            .map(|s| s.as_ptr())
            .unwrap_or(std::ptr::null());

        // Handle default_value
        let default_value_bytes = if let Some(value) = default_value {
            Some(get_pdfium_utf16le_bytes_from_str(value))
        } else {
            None
        };
        let default_value_ptr = default_value_bytes
            .as_ref()
            .map(|bytes| bytes.as_ptr() as *const c_ushort)
            .unwrap_or(std::ptr::null());

        // Compute field flags from the field type
        // This sets the /Ff (field flags) value at creation time to properly configure
        // button subtypes (checkbox/radio/push button), choice subtypes (combo/list), etc.
        let mut flags = field_type.required_flags();
        
        // Merge in any additional flags provided by the caller
        if let Some(extra) = additional_flags {
            flags |= PdfFormFieldFlags::from_bits_truncate(extra);
        }
        
        let field_flags_val = flags.bits() as c_int;

        // Create the widget annotation
        // The _option_bytes, _option_ptrs, default_appearance_cstr, and default_value_bytes
        // are kept alive for the duration of this call
        let annot_handle = self.bindings().FPDFPage_CreateWidgetAnnot(
            self.page_handle(),
            form_handle,
            field_name_cstr.as_ptr(),
            field_type_cstr.as_ptr(),
            &fs_rect,
            field_flags_val,
            options_ptr,
            option_count,
            max_length_val,
            quadding_val,
            default_appearance_ptr,
            default_value_ptr,
        );

        if annot_handle.is_null() {
            return Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ));
        }

        // Wrap in PdfPageWidgetAnnotation
        let annotation = PdfPageWidgetAnnotation::from_pdfium(
            self.document_handle(),
            self.page_handle(),
            annot_handle,
            Some(form_handle),
            self.bindings(),
        );

        // Note: We no longer automatically generate appearance streams for text fields.
        // PDFium should handle appearance stream generation natively when form values are set
        // via form handling APIs. Manual appearance stream generation via set_appearance()
        // can cause issues with font Resources dictionaries during flattening.

        // Handle content regeneration if needed
        if let Some(content_regeneration_strategy) =
            PdfPageIndexCache::get_content_regeneration_strategy_for_page(
                self.document_handle(),
                self.page_handle(),
            )
        {
            if content_regeneration_strategy
                == PdfPageContentRegenerationStrategy::AutomaticOnEveryChange
            {
                PdfPage::regenerate_content_immut_for_handle(
                    self.page_handle(),
                    self.bindings(),
                )?;
            }
        }

        Ok(annotation)
    }

    // Convenience functions for creating and positioning markup annotations
    // in a single function call.

    /// Creates a new [PdfPageSquigglyAnnotation] annotation and positions it underneath the given
    /// [PdfPageObject], coloring it with the given [PdfColor].
    ///
    /// If the given contents string is supplied, the annotation will be additionally configured
    /// so that when the given [PdfPageObject] is clicked in a conforming PDF viewer, the given
    /// contents string will be displayed in a popup window.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_squiggly_annotation_under_object(
        &mut self,
        object: &PdfPageObject,
        color: PdfColor,
        contents: Option<&str>,
    ) -> Result<PdfPageSquigglyAnnotation<'a>, PdfiumError> {
        let mut annotation = self.create_squiggly_annotation()?;

        // The annotation will not display if it is not positioned.

        let bounds = object.bounds()?;

        annotation.set_position(bounds.left(), bounds.bottom())?;
        annotation.set_stroke_color(color)?;

        const SQUIGGLY_HEIGHT: f32 = 12.0;

        let annotation_top = bounds.bottom().value - 5.0;
        let annotation_bottom = annotation_top - SQUIGGLY_HEIGHT;

        annotation
            .attachment_points_mut()
            .create_attachment_point_at_end(PdfQuadPoints::new_from_values(
                bounds.left().value,
                annotation_bottom,
                bounds.right().value,
                annotation_bottom,
                bounds.right().value,
                annotation_top,
                bounds.left().value,
                annotation_top,
            ))?;

        if let Some(contents) = contents {
            annotation.set_width(bounds.width())?;
            annotation.set_height(bounds.height())?;
            annotation.set_contents(contents)?;
        }

        // Generate appearance stream for flattening support
        let _ = annotation.generate_appearance_stream();

        Ok(annotation)
    }

    /// Creates a new [PdfPageUnderlineAnnotation] annotation and positions it underneath the given
    /// [PdfPageObject], coloring it with the given [PdfColor].
    ///
    /// If the given contents string is supplied, the annotation will be additionally configured
    /// so that when the given [PdfPageObject] is clicked in a conforming PDF viewer, the given
    /// contents string will be displayed in a popup window.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_underline_annotation_under_object(
        &mut self,
        object: &PdfPageObject,
        color: PdfColor,
        contents: Option<&str>,
    ) -> Result<PdfPageUnderlineAnnotation<'a>, PdfiumError> {
        let mut annotation = self.create_underline_annotation()?;

        // The annotation will not display if it is not positioned.

        let bounds = object.bounds()?;

        annotation.set_position(bounds.left(), bounds.bottom())?;
        annotation.set_stroke_color(color)?;
        annotation
            .attachment_points_mut()
            .create_attachment_point_at_end(bounds)?;

        if let Some(contents) = contents {
            annotation.set_width(bounds.width())?;
            annotation.set_height(bounds.height())?;
            annotation.set_contents(contents)?;
        }

        // Generate appearance stream for flattening support
        let _ = annotation.generate_appearance_stream();

        Ok(annotation)
    }

    /// Creates a new [PdfPageStrikeoutAnnotation] annotation and vertically positions it in the
    /// center the given [PdfPageObject], coloring it with the given [PdfColor].
    ///
    /// If the given contents string is supplied, the annotation will be additionally configured
    /// so that when the given [PdfPageObject] is clicked in a conforming PDF viewer, the given
    /// contents string will be displayed in a popup window.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_strikeout_annotation_through_object(
        &mut self,
        object: &PdfPageObject,
        color: PdfColor,
        contents: Option<&str>,
    ) -> Result<PdfPageStrikeoutAnnotation<'a>, PdfiumError> {
        let mut annotation = self.create_strikeout_annotation()?;

        // The annotation will not display if it is not positioned.

        let bounds = object.bounds()?;

        annotation.set_position(bounds.left(), bounds.bottom())?;
        annotation.set_stroke_color(color)?;
        annotation
            .attachment_points_mut()
            .create_attachment_point_at_end(bounds)?;

        if let Some(contents) = contents {
            annotation.set_width(bounds.width())?;
            annotation.set_height(bounds.height())?;
            annotation.set_contents(contents)?;
        }

        // Generate appearance stream for flattening support
        let _ = annotation.generate_appearance_stream();

        Ok(annotation)
    }

    /// Creates a new [PdfPageHighlightAnnotation] annotation and positions it so as to cover
    /// the given [PdfPageObject], coloring it with the given [PdfColor].
    ///
    /// If the given contents string is supplied, the annotation will be additionally configured
    /// so that when the given [PdfPageObject] is clicked in a conforming PDF viewer, the given
    /// contents string will be displayed in a popup window.
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    #[inline]
    pub fn create_highlight_annotation_over_object(
        &mut self,
        object: &PdfPageObject,
        color: PdfColor,
        contents: Option<&str>,
    ) -> Result<PdfPageHighlightAnnotation<'a>, PdfiumError> {
        let mut annotation = self.create_highlight_annotation()?;

        // The annotation will not display if it is not positioned.

        let bounds = object.bounds()?;

        annotation.set_position(bounds.left(), bounds.bottom())?;
        annotation.set_stroke_color(color)?;
        annotation
            .attachment_points_mut()
            .create_attachment_point_at_end(bounds)?;

        if let Some(contents) = contents {
            annotation.set_width(bounds.width())?;
            annotation.set_height(bounds.height())?;
            annotation.set_contents(contents)?;
        }

        // Generate appearance stream for flattening support
        let _ = annotation.generate_appearance_stream();

        Ok(annotation)
    }

    /// Removes the given [PdfPageAnnotation] from this [PdfPageAnnotations] collection,
    /// consuming the [PdfPageAnnotation].
    ///
    /// If the containing `PdfPage` has a content regeneration strategy of
    /// `PdfPageContentRegenerationStrategy::AutomaticOnEveryChange` then content regeneration
    /// will be triggered on the page.
    pub fn delete_annotation(
        &mut self,
        annotation: PdfPageAnnotation<'a>,
    ) -> Result<(), PdfiumError> {
        let index = self
            .bindings()
            .FPDFPage_GetAnnotIndex(self.page_handle(), annotation.handle());

        if index == -1 {
            return Err(PdfiumError::PageAnnotationIndexOutOfBounds);
        }

        if self.bindings().is_true(
            self.bindings()
                .FPDFPage_RemoveAnnot(self.page_handle(), index),
        ) {
            if let Some(content_regeneration_strategy) =
                PdfPageIndexCache::get_content_regeneration_strategy_for_page(
                    self.document_handle(),
                    self.page_handle(),
                )
            {
                if content_regeneration_strategy
                    == PdfPageContentRegenerationStrategy::AutomaticOnEveryChange
                {
                    PdfPage::regenerate_content_immut_for_handle(
                        self.page_handle(),
                        self.bindings(),
                    )
                } else {
                    Ok(())
                }
            } else {
                Err(PdfiumError::SourcePageIndexNotInCache)
            }
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Comprehensive debugging function to verify annotation appearance streams.
    /// Call this on an annotation index to diagnose flattening issues.
    ///
    /// This function prints detailed information about an annotation's appearance streams,
    /// including whether the /AP dictionary exists, stream lengths, content previews,
    /// and validation checks that are critical for successful flattening.
    ///
    /// # Parameters
    /// - `index` - Zero-based index of the annotation to debug
    /// - `annotation_type` - Descriptive name for the annotation (for logging)
    ///
    /// # Example
    /// ```rust,no_run
    /// use pdfium_render::prelude::*;
    ///
    /// let pdfium = Pdfium::default();
    /// let mut document = pdfium.create_new_pdf().unwrap();
    /// let mut page = document.pages_mut().create_page_at_start(PdfPagePaperSize::a4()).unwrap();
    ///
    /// page.annotations_mut().create_text_annotation("Test").unwrap();
    /// page.annotations().debug_appearance_streams(0, "Text Annotation");
    /// ```
    pub fn debug_appearance_streams(&self, index: usize, annotation_type: &str) {
        if index >= self.len() {
            println!("❌ Annotation index {} is out of bounds (max: {})", index, self.len().saturating_sub(1));
            return;
        }

        let handle = self.bindings.FPDFPage_GetAnnot(self.page_handle, index as c_int);
        if handle.is_null() {
            println!("❌ Failed to get annotation handle for index {}", index);
            return;
        }

        debug_annotation_appearance_streams(handle, self.bindings, annotation_type);
    }
}

/// An iterator over all the [PdfPageAnnotation] objects in a [PdfPageAnnotations] collection.
pub struct PdfPageAnnotationsIterator<'a> {
    annotations: &'a PdfPageAnnotations<'a>,
    next_index: PdfPageAnnotationIndex,
}

impl<'a> PdfPageAnnotationsIterator<'a> {
    #[inline]
    pub(crate) fn new(annotations: &'a PdfPageAnnotations<'a>) -> Self {
        PdfPageAnnotationsIterator {
            annotations,
            next_index: 0,
        }
    }
}

/// Comprehensive debugging function to verify annotation appearance streams.
/// Call this after creating annotations to diagnose flattening issues.
///
/// # Example
/// ```rust,no_run
/// use pdfium_render::prelude::*;
///
/// let pdfium = Pdfium::default();
/// let mut document = pdfium.create_new_pdf().unwrap();
/// let mut page = document.pages_mut().create_page_at_start(PdfPagePaperSize::a4()).unwrap();
///
/// let annotation = page.annotations_mut().create_text_annotation("Test").unwrap();
/// debug_annotation_appearance_streams(annotation.handle(), page.bindings(), "Text Annotation");
/// ```
pub fn debug_annotation_appearance_streams(
    annotation_handle: FPDF_ANNOTATION,
    bindings: &dyn PdfiumLibraryBindings,
    annotation_type: &str,
) {
    #[cfg(target_arch = "wasm32")]
    use web_sys::console;

    #[cfg(not(target_arch = "wasm32"))]
    fn log_info(message: &str) { println!("{}", message); }
    #[cfg(target_arch = "wasm32")]
    fn log_info(message: &str) { console::log_1(&message.into()); }

    log_info(&format!("🔍 DEBUGGING ANNOTATION APPEARANCE STREAMS: {}", annotation_type));
    log_info("═══════════════════════════════════════════════════════════");

    // Check if annotation has /AP key (appearance dictionary)
    let has_ap = bindings.FPDFAnnot_HasKey(annotation_handle, "AP");
    log_info(&format!("   Has /AP key: {}", bindings.is_true(has_ap)));

    if bindings.is_true(has_ap) {
        // Check each appearance mode
        let modes = [
            (PdfAppearanceMode::Normal, "Normal (/N)"),
            (PdfAppearanceMode::RollOver, "RollOver (/R)"),
            (PdfAppearanceMode::Down, "Down (/D)")
        ];

        for (mode, mode_name) in modes.iter() {
            // Get appearance stream length
            let ap_len = bindings.FPDFAnnot_GetAP(
                annotation_handle,
                mode.as_pdfium(),
                std::ptr::null_mut(),
                0,
            );
            log_info(&format!("   {} appearance stream length: {} bytes", mode_name, ap_len));

            // If there's content, try to read it
            if ap_len > 2 { // More than just empty UTF-16LE string
                // Allocate buffer for content
                let mut buffer = vec![0u16; (ap_len / 2) as usize];
                let result = bindings.FPDFAnnot_GetAP(
                    annotation_handle,
                    mode.as_pdfium(),
                    buffer.as_mut_ptr() as *mut FPDF_WCHAR,
                    ap_len,
                );

                if result == ap_len {
                    // Convert UTF-16LE to string and show preview
                    if let Ok(content) = String::from_utf16(&buffer[..buffer.len().saturating_sub(1)]) {
                        let preview = if content.len() > 1000 {
                            format!("{}...", &content[..1000])
                        } else {
                            content.clone()
                        };
                        log_info(&format!("   {} content preview: {}", mode_name, preview));

                        // Check for common PDF operators
                        let has_operators = content.contains(" BT") || content.contains(" ET") ||
                                          content.contains(" q") || content.contains(" Q") ||
                                          content.contains(" cm") || content.contains(" Do");
                        log_info(&format!("   {} has PDF operators: {}", mode_name, has_operators));
                        
                        // Check for ExtGState reference (critical for opacity/flattening)
                        // PDFium may use /GS gs, /GS1 gs, or generated names like /FXE1 gs
                        let has_gs_ref = content.contains(" gs");
                        log_info(&format!("   {} contains ExtGState reference (' gs'): {}", mode_name, has_gs_ref));
                    } else {
                        log_info(&format!("   {} content: (could not decode as UTF-16)", mode_name));
                    }
                } else {
                    log_info(&format!("   {} content: (failed to read)", mode_name));
                }
            }
        }
    } else {
        log_info("   ❌ No /AP dictionary found!");
    }

    // Check annotation rectangle (important for appearance streams)
    let mut rect = FS_RECTF { left: 0.0, bottom: 0.0, right: 0.0, top: 0.0 };
    let rect_result = bindings.FPDFAnnot_GetRect(annotation_handle, &mut rect);
    log_info(&format!("   Has valid rect: {} (left={}, bottom={}, right={}, top={})",
             bindings.is_true(rect_result), rect.left, rect.bottom, rect.right, rect.top));

    // Check if annotation is marked as invisible or other flags that might affect flattening
    let flags = bindings.FPDFAnnot_GetFlags(annotation_handle);
    log_info(&format!("   Annotation flags: {} (0x{:X})", flags, flags));

    // Check /ca (fill opacity) value - critical for flattening with opacity
    let has_ca = bindings.FPDFAnnot_HasKey(annotation_handle, "ca");
    if bindings.is_true(has_ca) {
        let mut ca_value: f32 = 0.0;
        let get_ca_result = bindings.FPDFAnnot_GetNumberValue(annotation_handle, "ca", &mut ca_value);
        if bindings.is_true(get_ca_result) {
            log_info(&format!("   /ca (fill opacity): {:.4} {}", ca_value, 
                if ca_value < 1.0 { "(< 1.0, requires ExtGState Resources)" } else { "(= 1.0, no opacity)" }));
            if ca_value < 1.0 {
                log_info("   ⚠️  For flattening: Resources/ExtGState/GS/ca must be set to match /ca value");
            }
        } else {
            log_info("   /ca key exists but could not read value");
        }
    } else {
        log_info("   /ca (fill opacity): not set (defaults to 1.0)");
    }

    log_info("═══════════════════════════════════════════════════════════");
}

impl<'a> Iterator for PdfPageAnnotationsIterator<'a> {
    type Item = PdfPageAnnotation<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.annotations.get(self.next_index);

        self.next_index += 1;

        next.ok()
    }
}
