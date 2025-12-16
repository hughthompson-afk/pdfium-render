//! Defines the [PdfFormTextField] struct, exposing functionality related to a single
//! form field of type [PdfFormFieldType::Text].

use crate::bindgen::{FPDF_ANNOTATION, FPDF_DOCUMENT, FPDF_FORMHANDLE};
use crate::bindings::PdfiumLibraryBindings;
use crate::error::PdfiumError;
use crate::pdf::document::page::field::private::internal::{
    PdfFormFieldFlags, PdfFormFieldPrivate,
};
use crate::pdf::document::page::field::text_appearance::TextFieldAppearanceBuilder;

#[cfg(doc)]
use {
    crate::pdf::document::form::PdfForm,
    crate::pdf::document::page::annotation::PdfPageAnnotationType,
    crate::pdf::document::page::field::{PdfFormField, PdfFormFieldType},
};

/// A single [PdfFormField] of type [PdfFormFieldType::Text]. The form field object defines
/// an interactive data entry widget that allows the user to enter data by typing.
///
/// Form fields in Pdfium are wrapped inside page annotations of type [PdfPageAnnotationType::Widget]
/// or [PdfPageAnnotationType::XfaWidget]. User-specified values can be retrieved directly from
/// each form field object by unwrapping the form field from the annotation, or in bulk from the
/// [PdfForm::field_values()] function.
pub struct PdfFormTextField<'a> {
    form_handle: FPDF_FORMHANDLE,
    annotation_handle: FPDF_ANNOTATION,
    document_handle: Option<FPDF_DOCUMENT>,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfFormTextField<'a> {
    #[inline]
    pub(crate) fn from_pdfium(
        form_handle: FPDF_FORMHANDLE,
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfFormTextField {
            form_handle,
            annotation_handle,
            document_handle: None,
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
        PdfFormTextField {
            form_handle,
            annotation_handle,
            document_handle: Some(document_handle),
            bindings,
        }
    }

    /// Returns the [PdfiumLibraryBindings] used by this [PdfFormTextField] object.
    #[inline]
    pub fn bindings(&self) -> &'a dyn PdfiumLibraryBindings {
        self.bindings
    }

    /// Returns the value assigned to this [PdfFormTextField] object, if any.
    #[inline]
    pub fn value(&self) -> Option<String> {
        if self.is_rich_text() {
            self.get_string_value("RV")
        } else {
            self.value_impl()
        }
    }

    /// Sets the value of this [PdfFormTextField] object.
    /// 
    /// This method attempts to use PDFium's form fill API to set the value, which ensures
    /// appearance streams are properly regenerated. If the form fill API cannot be used
    /// (e.g., page handle is not available), it falls back to direct annotation manipulation.
    /// 
    /// For best results (automatic appearance stream regeneration), use `set_value_with_page_handle()`
    /// instead, which allows you to provide the page handle directly.
    #[inline]
    pub fn set_value(&mut self, value: &str) -> Result<(), PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&"🔧 PdfFormTextField::set_value() - ATTEMPTING FORM FILL API".into());
            console::log_1(&format!("   Setting value: '{}'", value).into());
        }

        if self.is_rich_text() {
            self.set_string_value("RV", value)
        } else {
            // Try to use form fill API if we have a valid form handle
            let form_handle = self.form_handle();
            if form_handle.is_null() {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::warn_1(&"⚠️ Form handle is null, falling back to direct annotation manipulation".into());
                }
                return self.set_value_impl(value);
            }

            // For form fill API, we need the page handle. Since we don't have direct access,
            // we'll try to find it, but will fall back if not available.
            self.set_value_via_form_fill(value)
        }
    }

    /// Sets the value of this [PdfFormTextField] using PDFium's form fill API.
    /// 
    /// This method uses the form fill API (FORM_SetFocusedAnnot, FORM_SelectAllText,
    /// FORM_ReplaceSelection) which ensures appearance streams are properly regenerated,
    /// similar to how checkboxes work when their "AS" key is set.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The new value to set
    /// * `page_handle` - The FPDF_PAGE handle of the page containing this field
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// if let Some(field) = text_field.as_text_field_mut() {
    ///     field.set_value_with_page_handle("New value", page.page_handle())?;
    /// }
    /// ```
    pub fn set_value_with_page_handle(
        &mut self,
        value: &str,
        page_handle: crate::bindgen::FPDF_PAGE,
    ) -> Result<(), PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&"🔧 PdfFormTextField::set_value_with_page_handle() - USING FORM FILL API".into());
            console::log_1(&format!("   Setting value: '{}'", value).into());
            console::log_1(&format!("   Page handle: {:?}", page_handle).into());
        }

        if self.is_rich_text() {
            self.set_string_value("RV", value)
        } else {
            self.set_value_via_form_fill_with_page(value, page_handle)
        }
    }

    /// Sets the value using PDFium's form fill API. This ensures appearance streams
    /// are properly regenerated, similar to how checkboxes work.
    /// 
    /// Note: This method requires the form fill environment to be initialized and
    /// the page containing this field to be loaded.
    fn set_value_via_form_fill(&mut self, value: &str) -> Result<(), PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"   🔄 Attempting to use form fill API...".into());
        }

        // Try to find the page handle and annotation handle by searching through pages
        // This is a workaround since form fields don't directly store page handles
        // We need the annotation handle from the page, not the original one, for FORM_SetFocusedAnnot
        let result_opt = self.find_page_and_annotation_handle_for_annotation();

        if let Some((page_handle, page_annotation_handle)) = result_opt {
            let result = self.set_value_via_form_fill_with_page_and_annotation(
                value, 
                page_handle, 
                page_annotation_handle
            );
            // Close the annotation handle we retrieved from the page
            self.bindings().FPDFPage_CloseAnnot(page_annotation_handle);
            // Close the page handle we loaded
            self.bindings().FPDF_ClosePage(page_handle);
            result
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::warn_1(&"⚠️ Could not find page handle for annotation".into());
                console::warn_1(&"⚠️ Falling back to direct annotation manipulation (OLD METHOD)".into());
                console::warn_1(&"⚠️ This will NOT trigger appearance stream regeneration automatically".into());
                console::warn_1(&"💡 TIP: Use set_value_with_page_handle() to provide page handle for form fill API".into());
            }

            // Fall back to old method if form fill API couldn't be used
            self.set_value_impl(value)
        }
    }

    /// Internal implementation that uses form fill API with a known page handle.
    fn set_value_via_form_fill_with_page(
        &mut self,
        value: &str,
        page_handle: crate::bindgen::FPDF_PAGE,
    ) -> Result<(), PdfiumError> {
        // Use the original annotation handle - this is for when the page handle is provided externally
        self.set_value_via_form_fill_with_page_and_annotation(
            value,
            page_handle,
            self.annotation_handle(),
        )
    }

    /// Internal implementation that uses form fill API with a known page handle and annotation handle.
    /// The annotation handle should be the one retrieved from the page, not the original one.
    fn set_value_via_form_fill_with_page_and_annotation(
        &mut self,
        value: &str,
        page_handle: crate::bindgen::FPDF_PAGE,
        annotation_handle: crate::bindgen::FPDF_ANNOTATION,
    ) -> Result<(), PdfiumError> {
        let form_handle = self.form_handle();
        let bindings = self.bindings();

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"   ✅ Found page handle, using FORM FILL API".into());
        }

        // Ensure FORM_OnAfterLoadPage is called for this page
        bindings.FORM_OnAfterLoadPage(page_handle, form_handle);

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   🔧 Using annotation handle from page: {:?}", annotation_handle).into());
            console::log_1(&format!("   🔧 Original annotation handle: {:?}", self.annotation_handle()).into());
        }

        // Step 1: Focus the annotation
        // Use the annotation handle from the page, not the original one
        // This is important because FORM_SetFocusedAnnot needs the annotation handle
        // that's associated with the page handle we're using
        if bindings.is_true(bindings.FORM_SetFocusedAnnot(form_handle, annotation_handle)) {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"   ✅ FORM_SetFocusedAnnot succeeded".into());
            }

            // Step 2: Select all existing text
            bindings.FORM_SelectAllText(form_handle, page_handle);
            
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"   ✅ FORM_SelectAllText called".into());
            }

            // Step 3: Replace selection with new value
            // FORM_ReplaceSelection expects UTF-16LE encoded string
            let utf16le_bytes = bindings.get_pdfium_utf16le_bytes_from_str(value);
            // Cast the byte pointer to u16 pointer (FPDF_WIDESTRING is *const u16)
            let ws_text = utf16le_bytes.as_ptr() as *const std::os::raw::c_ushort;

            bindings.FORM_ReplaceSelection(form_handle, page_handle, ws_text);
            
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   ✅ FORM_ReplaceSelection called with value: '{}'", value).into());
            }

            // Step 4: Kill focus to save the value and trigger appearance stream regeneration
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
                console::warn_1(&"⚠️ FORM_SetFocusedAnnot failed, falling back to old method".into());
            }

            // Fall back to old method
            self.set_value_impl(value)
        }
    }

    /// Attempts to find the page handle and annotation handle that contains this annotation.
    /// This searches through all pages in the document to find which page contains this annotation.
    /// Returns (page_handle, annotation_handle_from_page) - both handles must be closed by the caller.
    fn find_page_and_annotation_handle_for_annotation(
        &self,
    ) -> Option<(crate::bindgen::FPDF_PAGE, crate::bindgen::FPDF_ANNOTATION)> {
        let document_handle = match self.document_handle {
            Some(handle) => handle,
            None => {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::warn_1(&"⚠️ Document handle not available in PdfFormTextField".into());
                }
                return None;
            }
        };
        let annotation_handle = self.annotation_handle();
        let bindings = self.bindings();

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   🔍 Searching for page containing annotation handle: {:?}", annotation_handle).into());
        }

        // Search through all pages to find which one contains this annotation
        let page_count = bindings.FPDF_GetPageCount(document_handle);
        
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   📄 Document has {} pages", page_count).into());
        }

        // Get the rectangle of the annotation we're looking for
        let mut target_rect = crate::bindgen::FS_RECTF {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        };
        if !bindings.is_true(bindings.FPDFAnnot_GetRect(annotation_handle, &mut target_rect)) {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::warn_1(&"   ⚠️ Could not get rectangle for target annotation".into());
            }
            return None;
        }

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   📐 Target annotation rect: left={:.2}, top={:.2}, right={:.2}, bottom={:.2}", 
                target_rect.left, target_rect.top, target_rect.right, target_rect.bottom).into());
        }

        for i in 0..page_count {
            let page_handle = bindings.FPDF_LoadPage(document_handle, i);
            if !page_handle.is_null() {
                let annot_count = bindings.FPDFPage_GetAnnotCount(page_handle);
                
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&format!("   📄 Page {} has {} annotations", i, annot_count).into());
                }

                for j in 0..annot_count {
                    let current_annot_handle = bindings.FPDFPage_GetAnnot(page_handle, j);
                    if !current_annot_handle.is_null() {
                        // Compare annotations by their rectangle instead of handle
                        // PDFium may return different handle values for the same annotation
                        let mut current_rect = crate::bindgen::FS_RECTF {
                            left: 0.0,
                            top: 0.0,
                            right: 0.0,
                            bottom: 0.0,
                        };
                        
                        if bindings.is_true(bindings.FPDFAnnot_GetRect(current_annot_handle, &mut current_rect)) {
                            // Compare rectangles with a small epsilon for floating point comparison
                            const EPSILON: f32 = 0.01;
                            let rects_match = 
                                (current_rect.left - target_rect.left).abs() < EPSILON &&
                                (current_rect.top - target_rect.top).abs() < EPSILON &&
                                (current_rect.right - target_rect.right).abs() < EPSILON &&
                                (current_rect.bottom - target_rect.bottom).abs() < EPSILON;

                            #[cfg(target_arch = "wasm32")]
                            {
                                use web_sys::console;
                                console::log_1(&format!("   🔍 Annotation {} rect: left={:.2}, top={:.2}, right={:.2}, bottom={:.2}", 
                                    j, current_rect.left, current_rect.top, current_rect.right, current_rect.bottom).into());
                                console::log_1(&format!("   🔍 Rectangles match: {}", rects_match).into());
                            }

                            if rects_match {
                                #[cfg(target_arch = "wasm32")]
                                {
                                    use web_sys::console;
                                    console::log_1(&format!("   ✅ Found annotation on page {} by rectangle match!", i).into());
                                }
                                // Return both the page handle and the annotation handle from the page
                                // The caller is responsible for closing both handles
                                // Note: We don't close current_annot_handle here - the caller needs it
                                return Some((page_handle, current_annot_handle));
                            }
                        }
                        // Only close the annotation handle if it didn't match
                        bindings.FPDFPage_CloseAnnot(current_annot_handle);
                    }
                }
                // Close the page handle if we didn't find the annotation on this page
                bindings.FPDF_ClosePage(page_handle);
            }
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::warn_1(&"   ⚠️ Annotation not found on any page".into());
        }
        
        None
    }

    /// Returns `true` if this [PdfFormTextField] is configured as a multi-line text field.
    #[inline]
    pub fn is_multiline(&self) -> bool {
        self.get_flags_impl()
            .contains(PdfFormFieldFlags::TextMultiline)
    }

    #[cfg(any(feature = "pdfium_future", feature = "pdfium_7350"))]
    /// Controls whether or not this [PdfFormTextField] is configured as a multi-line text field.
    #[inline]
    pub fn set_is_multiline(&self, is_multiline: bool) -> Result<(), PdfiumError> {
        self.update_one_flag_impl(PdfFormFieldFlags::TextMultiline, is_multiline)
    }

    /// Returns `true` if this [PdfFormTextField] is configured as a password field.
    #[inline]
    pub fn is_password(&self) -> bool {
        self.get_flags_impl()
            .contains(PdfFormFieldFlags::TextPassword)
    }

    #[cfg(any(feature = "pdfium_future", feature = "pdfium_7350"))]
    /// Controls whether or not this [PdfFormTextField] is configured as a password text field.
    #[inline]
    pub fn set_is_password(&self, is_password: bool) -> Result<(), PdfiumError> {
        self.update_one_flag_impl(PdfFormFieldFlags::TextPassword, is_password)
    }

    /// Returns `true` if this [PdfFormTextField] represents the path of a file
    /// whose contents are to be submitted as the value of the field.
    ///
    /// This flag was added in PDF version 1.4
    pub fn is_file_select(&self) -> bool {
        self.get_flags_impl()
            .contains(PdfFormFieldFlags::TextFileSelect)
    }

    #[cfg(any(feature = "pdfium_future", feature = "pdfium_7350"))]
    /// Controls whether or not this [PdfFormTextField] represents the path of a file
    /// whose contents are to be submitted as the value of the field.
    ///
    /// This flag was added in PDF version 1.4.
    pub fn set_is_file_select(&mut self, is_file_select: bool) -> Result<(), PdfiumError> {
        self.update_one_flag_impl(PdfFormFieldFlags::TextFileSelect, is_file_select)
    }

    /// Returns `true` if text entered into this [PdfFormTextField] should be spell checked.
    pub fn is_spell_checked(&self) -> bool {
        !self
            .get_flags_impl()
            .contains(PdfFormFieldFlags::TextDoNotSpellCheck)
    }

    #[cfg(any(feature = "pdfium_future", feature = "pdfium_7350"))]
    /// Controls whether or not text entered into this [PdfFormTextField] should be spell checked.
    pub fn set_is_spell_checked(&mut self, is_spell_checked: bool) -> Result<(), PdfiumError> {
        self.update_one_flag_impl(PdfFormFieldFlags::TextDoNotSpellCheck, !is_spell_checked)
    }

    /// Returns `true` if the internal area of this [PdfFormTextField] can scroll either
    /// horizontally or vertically to accommodate text entry longer than what can fit
    /// within the field's annotation bounds. If this value is `false`, then once the
    /// field is full, no further text entry will be accepted.
    ///
    /// This flag was added in PDF version 1.4.
    pub fn is_scrollable(&self) -> bool {
        !self
            .get_flags_impl()
            .contains(PdfFormFieldFlags::TextDoNotScroll)
    }

    #[cfg(any(feature = "pdfium_future", feature = "pdfium_7350"))]
    /// Controls whether or not the internal area of this [PdfFormTextField] can scroll
    /// either horizontally or vertically to accommodate text entry longer than what can fit
    /// within the field's annotation bounds. If set to `false`, no further text entry
    /// will be accepted once the field's annotation bounds are full.
    ///
    /// This flag was added in PDF version 1.4.
    pub fn set_is_scrollable(&mut self, is_scrollable: bool) -> Result<(), PdfiumError> {
        self.update_one_flag_impl(PdfFormFieldFlags::TextDoNotScroll, !is_scrollable)
    }

    /// Returns `true` if this [PdfFormTextField] is "combed", that is, automatically divided
    /// into equally-spaced positions ("combs"), with the text in the field laid out into
    /// those combs.
    ///
    /// For more information on this setting, refer to Table 8.77 of The PDF Reference
    /// (Sixth Edition, PDF Format 1.7), on page 691.
    ///
    /// This flag was added in PDF version 1.5.
    pub fn is_combed(&self) -> bool {
        // This flag only takes effect if the multi-line, password, and file select flags
        // are all unset.

        !self.is_multiline()
            && !self.is_password()
            && !self.is_file_select()
            && self.get_flags_impl().contains(PdfFormFieldFlags::TextComb)
    }

    // TODO: AJRC - 20/06/25 - there is little point providing the matching `set_is_combed()`
    // function, because it makes little sense without being also able to set the `MaxValue`
    // dictionary parameter that controls the number of combs. However, `MaxValue` must be
    // an integer, and Pdfium does not currently provide a `FPDFAnnot_SetNumberValue()`
    // function that could correctly set it.

    /// Returns `true` if the text in this [PdfFormTextField] is a rich text string.
    ///
    /// This flag was added in PDF version 1.5.
    pub fn is_rich_text(&self) -> bool {
        self.get_flags_impl()
            .contains(PdfFormFieldFlags::TextRichText)
    }

    #[cfg(any(feature = "pdfium_future", feature = "pdfium_7350"))]
    /// Controls whether or not the text in this [PdfFormTextField] is a rich text string.
    ///
    /// This flag was added in PDF version 1.5.
    pub fn set_is_rich_text(&mut self, is_rich_text: bool) -> Result<(), PdfiumError> {
        self.update_one_flag_impl(PdfFormFieldFlags::TextRichText, is_rich_text)
    }

    /// Returns a builder for setting the visual appearance of this text field.
    ///
    /// The appearance builder allows you to configure how the text field renders
    /// its content, including font, size, color, and alignment. This is essential
    /// for proper flattening of text fields, as they require explicit appearance
    /// streams to render correctly.
    ///
    /// Note: Text fields automatically generate appearance streams when created via
    /// `create_widget_annotation()`. This method allows manual customization of
    /// the appearance stream after creation.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// text_field.set_appearance()
    ///     .with_font_size(14.0)
    ///     .with_text_color(PdfColor::BLUE)
    ///     .with_alignment(TextAlignment::Center)
    ///     .apply()?;
    /// ```
    pub fn set_appearance(&self) -> TextFieldAppearanceBuilder<'_> {
        let field_value = self.value().unwrap_or_default();
        let is_password = self.is_password();
        let is_multiline = self.is_multiline();
        let da_string = self.get_string_value("DA");

        TextFieldAppearanceBuilder::new(
            self.annotation_handle,
            self.bindings,
            field_value,
            is_password,
            is_multiline,
            da_string,
        )
    }
}

impl<'a> PdfFormFieldPrivate<'a> for PdfFormTextField<'a> {
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
