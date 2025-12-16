//! Defines types and builders for setting the visual appearance of text fields
//! using PDF content streams.
//!
//! # Overview
//!
//! PDF text fields have appearance streams that define how their text content
//! renders when the PDF is displayed or flattened. This module provides a builder
//! API for generating these appearance streams manually.
//!
//! # Example
//!
//! ```rust,ignore
//! // Create a text field and set its appearance
//! let text_field = page.annotations_mut().create_widget_annotation(
//!     form_handle,
//!     "MyTextField",
//!     PdfFormFieldType::Text,
//!     PdfRect::new(100.0, 700.0, 300.0, 720.0),
//! )?;
//!
//! // Access the form field and set its appearance
//! if let Some(field) = text_field.form_field() {
//!     if let Some(text_field) = field.as_text_field() {
//!         text_field.set_appearance()
//!             .with_font_size(12.0)
//!             .with_text_color(PdfColor::BLACK)
//!             .with_alignment(TextAlignment::Left)
//!             .apply()?;
//!     }
//! }
//! ```

use crate::bindgen::FPDF_ANNOTATION;
use crate::bindings::PdfiumLibraryBindings;
use crate::error::{PdfiumError, PdfiumInternalError};
use crate::pdf::appearance_mode::PdfAppearanceMode;
use crate::pdf::color::PdfColor;

/// Text alignment options for positioning text within the field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlignment {
    /// Align text to the left edge of the field.
    Left,
    /// Center text horizontally within the field.
    Center,
    /// Align text to the right edge of the field.
    Right,
}

/// Vertical alignment options for positioning text within the field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerticalAlignment {
    /// Align text to the top of the field.
    Top,
    /// Center text vertically within the field.
    Middle,
    /// Align text to the bottom of the field.
    Bottom,
}

/// Configuration for text field appearance rendering.
#[derive(Debug, Clone)]
pub struct TextFieldAppearanceConfig {
    /// Font name (e.g., "/Helv", "/Times-Roman").
    /// If None, will be extracted from the field's DA string or default to "/Helv".
    pub font_name: Option<String>,
    /// Font size in points.
    /// If None, will be extracted from the field's DA string or default to 12.0.
    pub font_size: Option<f32>,
    /// Text color.
    /// If None, will be extracted from the field's DA string or default to black.
    pub text_color: Option<PdfColor>,
    /// Horizontal text alignment.
    pub horizontal_alignment: TextAlignment,
    /// Vertical text alignment.
    pub vertical_alignment: VerticalAlignment,
    /// Left padding in points.
    pub padding_left: f32,
    /// Right padding in points.
    pub padding_right: f32,
    /// Top padding in points.
    pub padding_top: f32,
    /// Bottom padding in points.
    pub padding_bottom: f32,
    /// Character to use for password masking.
    /// If None, defaults to "*".
    pub password_mask_char: Option<char>,
}

impl Default for TextFieldAppearanceConfig {
    fn default() -> Self {
        Self {
            font_name: None,
            font_size: None,
            text_color: None,
            horizontal_alignment: TextAlignment::Left,
            vertical_alignment: VerticalAlignment::Middle,
            padding_left: 2.0,
            padding_right: 2.0,
            padding_top: 2.0,
            padding_bottom: 2.0,
            password_mask_char: None,
        }
    }
}

/// Parsed components of a PDF Default Appearance (DA) string.
#[derive(Debug, Clone)]
struct DaComponents {
    font_name: String,
    font_size: f32,
    text_color: PdfColor,
}

/// Builder for constructing and applying the visual appearance of a text field.
///
/// This builder collects configuration and generates a PDF content stream
/// that renders the text field's value with correct font, size, color, and positioning.
/// The resulting appearance is used for both display and flattening operations.
pub struct TextFieldAppearanceBuilder<'a> {
    bindings: &'a dyn PdfiumLibraryBindings,
    annotation_handle: FPDF_ANNOTATION,
    field_value: String,
    is_password: bool,
    is_multiline: bool,
    da_string: Option<String>,
    config: TextFieldAppearanceConfig,
}

impl<'a> TextFieldAppearanceBuilder<'a> {
    /// Creates a new builder for the given annotation with field information.
    pub(crate) fn new(
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
        field_value: String,
        is_password: bool,
        is_multiline: bool,
        da_string: Option<String>,
    ) -> Self {
        Self {
            bindings,
            annotation_handle,
            field_value,
            is_password,
            is_multiline,
            da_string,
            config: TextFieldAppearanceConfig::default(),
        }
    }

    /// Sets the font size in points.
    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.config.font_size = Some(font_size);
        self
    }

    /// Sets the text color.
    pub fn with_text_color(mut self, color: PdfColor) -> Self {
        self.config.text_color = Some(color);
        self
    }

    /// Sets the horizontal text alignment.
    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.config.horizontal_alignment = alignment;
        self
    }

    /// Sets the vertical text alignment.
    pub fn with_vertical_alignment(mut self, alignment: VerticalAlignment) -> Self {
        self.config.vertical_alignment = alignment;
        self
    }

    /// Sets the padding on all sides.
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.config.padding_left = padding;
        self.config.padding_right = padding;
        self.config.padding_top = padding;
        self.config.padding_bottom = padding;
        self
    }

    /// Sets individual padding values.
    pub fn with_individual_padding(
        mut self,
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
    ) -> Self {
        self.config.padding_left = left;
        self.config.padding_right = right;
        self.config.padding_top = top;
        self.config.padding_bottom = bottom;
        self
    }

    /// Sets the password masking character.
    pub fn with_password_mask(mut self, mask_char: char) -> Self {
        self.config.password_mask_char = Some(mask_char);
        self
    }

    /// Applies the text field appearance to the field.
    ///
    /// This sets the normal appearance stream (`/AP /N`) of the text field.
    /// The appearance is used for both display and flattening operations.
    ///
    /// # Errors
    ///
    /// Returns an error if PDFium fails to set the appearance stream.
    pub fn apply(self) -> Result<(), PdfiumError> {
        self.apply_with_mode(PdfAppearanceMode::Normal)
    }

    /// Applies the text field appearance with a specific appearance mode.
    ///
    /// Most text fields only need the Normal appearance. RollOver and Down
    /// appearances are used for interactive hover/click states.
    ///
    /// NOTE: Instead of manually setting appearance streams (which can cause font
    /// Resources dictionary issues during flattening), this method now ensures
    /// the DA (Default Appearance) string is set correctly. PDFium will then
    /// generate appearance streams natively, which properly handle font Resources
    /// during flattening operations.
    pub fn apply_with_mode(self, mode: PdfAppearanceMode) -> Result<(), PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&"🔧 TextFieldAppearanceBuilder::apply_with_mode() - DA DEBUG".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
        }

        // Get existing DA string before modification
        let existing_da = self.da_string.clone();
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   Existing DA string (from field): {:?}", existing_da).into());
        }

        // Build DA string from configuration
        let da_components = self.parse_da_string()?;
        let font_name = self.config.font_name.clone()
            .unwrap_or_else(|| da_components.font_name.clone());
        let font_size = self.config.font_size
            .unwrap_or(da_components.font_size);
        let text_color = self.config.text_color
            .unwrap_or(da_components.text_color);

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   Configuration: font={}, size={}, color=({},{},{})", 
                font_name, font_size, text_color.red(), text_color.green(), text_color.blue()).into());
        }

        // Build DA string: "/FontName fontSize Tf r g b rg"
        let r = text_color.red() as f32 / 255.0;
        let g = text_color.green() as f32 / 255.0;
        let b = text_color.blue() as f32 / 255.0;
        let da_string = format!("/{} {} Tf {:.4} {:.4} {:.4} rg", font_name, font_size, r, g, b);

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   📝 Setting DA string: '{}'", da_string).into());
        }

        // Set the DA string - this allows PDFium to generate appearance streams natively
        // with proper font Resources dictionary handling
        let _ = mode; // Only Normal mode is typically used for text fields
        
        let set_result = self.bindings.FPDFAnnot_SetStringValue_str(
            self.annotation_handle,
            "DA",
            &da_string,
        );

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   FPDFAnnot_SetStringValue_str('DA', '{}') returned: {}", 
                da_string, if self.bindings.is_true(set_result) { "✅ SUCCESS" } else { "❌ FAILED" }).into());
        }

        if self.bindings.is_true(set_result) {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                use crate::utils::mem::create_byte_buffer;
                use crate::utils::utf16le::get_string_from_pdfium_utf16le_bytes;

                // Verify the DA string was actually written by reading it back
                // First, get the buffer length
                let buffer_length = self.bindings.FPDFAnnot_GetStringValue(
                    self.annotation_handle,
                    "DA",
                    std::ptr::null_mut(),
                    0,
                );

                console::log_1(&format!("   Buffer length needed: {} bytes", buffer_length).into());

                if buffer_length > 2 {
                    let mut buffer = create_byte_buffer(buffer_length as usize);
                    let read_length = self.bindings.FPDFAnnot_GetStringValue(
                        self.annotation_handle,
                        "DA",
                        buffer.as_mut_ptr() as *mut crate::bindgen::FPDF_WCHAR,
                        buffer_length,
                    );

                    if read_length == buffer_length {
                        let read_da = get_string_from_pdfium_utf16le_bytes(buffer).unwrap_or_default();
                        console::log_1(&format!("   ✅ Verified: Read back DA string from annotation: '{}'", read_da).into());
                        
                        if read_da == da_string {
                            console::log_1(&"   ✅ DA string matches what we set!".into());
                        } else {
                            console::warn_1(&format!("   ⚠️  DA string mismatch! Expected: '{}', Got: '{}'", da_string, read_da).into());
                        }
                    } else {
                        console::warn_1(&format!("   ⚠️  Failed to read back DA string (expected {} bytes, got {})", buffer_length, read_length).into());
                    }
                } else {
                    console::warn_1(&format!("   ⚠️  DA string appears to be empty or invalid (length: {})", buffer_length).into());
                }

                // Also check if /DA key exists in annotation dictionary
                let has_da = self.bindings.FPDFAnnot_HasKey(self.annotation_handle, "DA");
                console::log_1(&format!("   Annotation has /DA key: {}", if self.bindings.is_true(has_da) { "✅ YES" } else { "❌ NO" }).into());

                // Check the value type
                let value_type = self.bindings.FPDFAnnot_GetValueType(self.annotation_handle, "DA");
                let type_name = match value_type {
                    0 => "UNKNOWN",
                    1 => "BOOLEAN",
                    2 => "NUMBER",
                    3 => "STRING",
                    4 => "NAME",
                    5 => "ARRAY",
                    6 => "DICTIONARY",
                    7 => "STREAM",
                    _ => "UNKNOWN",
                };
                console::log_1(&format!("   /DA value type: {} ({})", value_type, type_name).into());

                console::log_1(&"═══════════════════════════════════════════════════════════".into());
            }

            // DA string set successfully - PDFium will generate appearance streams natively
            Ok(())
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::warn_1(&"   ❌ Failed to set DA string!".into());
                console::log_1(&"═══════════════════════════════════════════════════════════".into());
            }
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Builds the PDF content stream for the text field appearance.
    fn build_content_stream(&self) -> Result<String, PdfiumError> {
        // Get field information
        let field_bounds = self.get_field_bounds()?;
        let da_components = self.parse_da_string()?;

        // Apply configuration overrides or use DA defaults
        let font_name = self.config.font_name.clone()
            .unwrap_or_else(|| da_components.font_name.clone());
        let font_size = self.config.font_size
            .unwrap_or(da_components.font_size);
        let text_color = self.config.text_color
            .unwrap_or(da_components.text_color);

        // Process text value
        let display_text = if self.is_password {
            self.mask_password_text(&self.field_value)
        } else {
            self.field_value.clone()
        };

        // Handle empty text
        if display_text.is_empty() {
            return Ok(String::new());
        }

        // Build content stream
        // CRITICAL: Use PDFium's marked content structure (/Tx BMC ... EMC) for form fields.
        // This ensures PDFium properly handles font Resources during flattening.
        // Without this, manually set appearance streams may not render correctly after flattening.
        let mut stream = String::with_capacity(1024);

        // Save graphics state
        stream.push_str("q\n");
        stream.push_str("Q\n");

        // Begin marked content for form field text (/Tx = Text field)
        stream.push_str("/Tx BMC\n");

        // Save graphics state for text content
        stream.push_str("q\n");

        // Begin text object
        stream.push_str("BT\n");

        // Set text color (RGB) - must come before font setting for proper rendering
        let r = text_color.red() as f32 / 255.0;
        let g = text_color.green() as f32 / 255.0;
        let b = text_color.blue() as f32 / 255.0;
        stream.push_str(&format!("{:.4} {:.4} {:.4} rg\n", r, g, b));

        // Set font and size
        stream.push_str(&format!("/{} {} Tf\n", font_name, font_size));

        if self.is_multiline {
            self.render_multiline_text(&display_text, font_size, &field_bounds, &mut stream);
        } else {
            self.render_singleline_text(&display_text, font_size, &field_bounds, &mut stream);
        }

        // End text object
        stream.push_str("ET\n");

        // Restore graphics state
        stream.push_str("Q\n");

        // End marked content
        stream.push_str("EMC\n");

        Ok(stream)
    }


    /// Gets the field bounds from the annotation.
    fn get_field_bounds(&self) -> Result<(f32, f32, f32, f32), PdfiumError> {
        // Get annotation rectangle (left, bottom, right, top)
        let mut rect = crate::bindgen::FS_RECTF {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        };

        let success = self.bindings.FPDFAnnot_GetRect(self.annotation_handle, &mut rect);

        if self.bindings.is_true(success) {
            Ok((rect.left, rect.bottom, rect.right, rect.top))
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Parses the Default Appearance (DA) string from the field.
    fn parse_da_string(&self) -> Result<DaComponents, PdfiumError> {
        if let Some(da) = &self.da_string {
            self.parse_da_string_content(da)
        } else {
            // Return sensible defaults if no DA string is found
            Ok(DaComponents {
                font_name: "Helv".to_string(),
                font_size: 12.0,
                text_color: PdfColor::BLACK,
            })
        }
    }

    /// Parses the content of a DA string.
    fn parse_da_string_content(&self, da_string: &str) -> Result<DaComponents, PdfiumError> {
        // DA string format: /FontName FontSize Tf r g b rg
        // Example: "/Helv 12 Tf 0 0 0 rg"
        // Note: DA strings may have quotes around them, which we need to strip

        // Strip surrounding quotes if present
        let cleaned_da = da_string.trim_matches('"').trim();

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   🔍 Parsing DA string (original): '{}'", da_string).into());
            console::log_1(&format!("   🔍 Parsing DA string (cleaned): '{}'", cleaned_da).into());
        }

        let mut font_name = "Helv".to_string();
        let mut font_size = 12.0;
        let mut text_color = PdfColor::BLACK;

        // Simple parsing - split by whitespace and look for patterns
        let tokens: Vec<&str> = cleaned_da.split_whitespace().collect();

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   🔍 Tokens: {:?}", tokens).into());
        }

        let mut i = 0;
        while i < tokens.len() {
            let token = tokens[i];

            // Look for font name (starts with /)
            if token.starts_with('/') {
                font_name = token[1..].to_string(); // Remove leading /
                
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&format!("   ✅ Parsed font name: '{}'", font_name).into());
                }
                
                i += 1;

                // Next token should be font size
                if i < tokens.len() {
                    if let Ok(size) = tokens[i].parse::<f32>() {
                        font_size = size;
                        
                        #[cfg(target_arch = "wasm32")]
                        {
                            use web_sys::console;
                            console::log_1(&format!("   ✅ Parsed font size: {}", font_size).into());
                        }
                    }
                    i += 1;
                }

                // Skip "Tf" if present
                if i < tokens.len() && tokens[i] == "Tf" {
                    i += 1;
                }
            }
            // Look for color values (r g b rg pattern)
            else if let Ok(r) = token.parse::<f32>() {
                if i + 3 < tokens.len() {
                    if let (Ok(g), Ok(b)) = (tokens[i + 1].parse::<f32>(), tokens[i + 2].parse::<f32>()) {
                        if tokens[i + 3] == "rg" {
                            // Convert from 0.0-1.0 range to 0-255
                            let r_u8 = (r * 255.0).round() as u8;
                            let g_u8 = (g * 255.0).round() as u8;
                            let b_u8 = (b * 255.0).round() as u8;
                            text_color = PdfColor::new(r_u8, g_u8, b_u8, 255);
                            
                            #[cfg(target_arch = "wasm32")]
                            {
                                use web_sys::console;
                                console::log_1(&format!("   ✅ Parsed color: r={} g={} b={} -> RGB({},{},{})", r, g, b, r_u8, g_u8, b_u8).into());
                            }
                            
                            i += 4;
                            continue;
                        }
                    }
                }
                i += 1;
            } else {
                i += 1;
            }
        }

        Ok(DaComponents {
            font_name,
            font_size,
            text_color,
        })
    }


    /// Masks password text with the specified character.
    fn mask_password_text(&self, text: &str) -> String {
        let mask_char = self.config.password_mask_char.unwrap_or('*');
        mask_char.to_string().repeat(text.chars().count())
    }


    /// Renders single-line text within the field bounds.
    fn render_singleline_text(
        &self,
        text: &str,
        font_size: f32,
        bounds: &(f32, f32, f32, f32),
        stream: &mut String,
    ) {
        let text_position = self.calculate_singleline_position(text, font_size, bounds);
        stream.push_str(&format!("{:.4} {:.4} Td\n", text_position.0, text_position.1));

        let escaped_text = self.escape_pdf_string(text);
        stream.push_str(&format!("({}) Tj\n", escaped_text));
    }

    /// Renders multi-line text within the field bounds.
    fn render_multiline_text(
        &self,
        text: &str,
        font_size: f32,
        bounds: &(f32, f32, f32, f32),
        stream: &mut String,
    ) {
        // Simple line splitting by \n or \r\n
        let lines: Vec<&str> = text.split('\n').collect();
        let line_height = font_size * 1.2; // Standard line spacing

        let start_y = self.calculate_multiline_start_y(lines.len(), font_size, line_height, bounds);

        for (i, line) in lines.iter().enumerate() {
            if line.is_empty() {
                continue;
            }

            let y = start_y - (i as f32 * line_height);
            let x = self.config.padding_left;

            stream.push_str(&format!("{:.4} {:.4} Td\n", x, y));

            let escaped_line = self.escape_pdf_string(line);
            stream.push_str(&format!("({}) Tj\n", escaped_line));

            if i < lines.len() - 1 {
                stream.push_str("T*\n"); // Move to next line
            }
        }
    }

    /// Calculates the position for single-line text.
    fn calculate_singleline_position(
        &self,
        text: &str,
        font_size: f32,
        bounds: &(f32, f32, f32, f32),
    ) -> (f32, f32) {
        let (_left, bottom, _right, top) = *bounds;
        let field_height = top - bottom;

        // Calculate approximate text width (rough approximation for common fonts)
        // TODO: Implement proper font metrics for accurate text width calculation
        let text_width = self.calculate_text_width(text, font_size);

        // Calculate X position based on horizontal alignment
        let field_width = bounds.2 - bounds.0;
        let x = match self.config.horizontal_alignment {
            TextAlignment::Left => self.config.padding_left,
            TextAlignment::Center => {
                let center_x = field_width / 2.0;
                center_x - text_width / 2.0
            }
            TextAlignment::Right => {
                field_width - text_width - self.config.padding_right
            }
        };

        // Calculate Y position based on vertical alignment
        let y = match self.config.vertical_alignment {
            VerticalAlignment::Top => {
                field_height - font_size - self.config.padding_top
            }
            VerticalAlignment::Middle => {
                (field_height - font_size) / 2.0
            }
            VerticalAlignment::Bottom => {
                self.config.padding_bottom
            }
        };

        (x.max(self.config.padding_left), y.max(self.config.padding_bottom))
    }

    /// Calculates approximate text width based on character count and font size.
    /// This is a rough approximation - proper implementation would require font metrics.
    fn calculate_text_width(&self, text: &str, font_size: f32) -> f32 {
        // Rough approximation: average character width is about 0.5 * font_size
        // This works reasonably well for most Western fonts
        let avg_char_width = font_size * 0.5;
        text.chars().count() as f32 * avg_char_width
    }

    /// Calculates the starting Y position for multi-line text.
    fn calculate_multiline_start_y(
        &self,
        line_count: usize,
        font_size: f32,
        line_height: f32,
        bounds: &(f32, f32, f32, f32),
    ) -> f32 {
        let (_left, bottom, _right, top) = *bounds;
        let field_height = top - bottom;
        let total_text_height = line_count as f32 * line_height;

        match self.config.vertical_alignment {
            VerticalAlignment::Top => field_height - font_size - self.config.padding_top,
            VerticalAlignment::Middle => {
                let center_y = field_height / 2.0;
                center_y + (total_text_height / 2.0) - font_size
            }
            VerticalAlignment::Bottom => {
                total_text_height - font_size + self.config.padding_bottom
            }
        }
    }

    /// Escapes special characters in a PDF string.
    fn escape_pdf_string(&self, text: &str) -> String {
        // PDF string escaping for content streams
        text.replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)")
            .replace('\r', "\\r")
            .replace('\n', "\\n")
            .replace('\t', "\\t")
    }
}
