//! Defines types and builders for setting the visual appearance of free text annotations
//! using PDF content streams.
//!
//! # Overview
//!
//! PDF free text annotations need appearance streams (`/AP` dictionary) to render properly
//! when the PDF is displayed or flattened. Unlike form fields, free text annotations don't
//! get automatic appearance generation from PDFium, so this module provides a builder
//! API for generating these appearance streams manually.
//!
//! # Example
//!
//! ```rust,ignore
//! // Create a free text annotation and set its appearance
//! let mut annotation = page.annotations_mut().create_free_text_annotation("Hello World")?;
//!
//! // The appearance stream is automatically generated, but you can customize it
//! annotation.set_appearance()
//!     .with_font_size(14.0)
//!     .with_text_color(PdfColor::RED)
//!     .with_horizontal_alignment(TextAlignment::Center)
//!     .with_vertical_alignment(VerticalAlignment::Middle)
//!     .with_border(1.0, PdfColor::BLACK)
//!     .with_background(PdfColor::new(240, 240, 240))
//!     .apply()?;
//! ```

use crate::bindgen::{FPDF_ANNOTATION, FS_RECTF};
use crate::bindings::PdfiumLibraryBindings;
use crate::error::{PdfiumError, PdfiumInternalError};
use crate::pdf::appearance_mode::PdfAppearanceMode;
use crate::pdf::color::PdfColor;

/// Text alignment options for positioning text within the annotation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlignment {
    /// Align text to the left edge of the annotation.
    Left,
    /// Center text horizontally within the annotation.
    Center,
    /// Align text to the right edge of the annotation.
    Right,
}

/// Vertical alignment options for positioning text within the annotation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerticalAlignment {
    /// Align text to the top of the annotation.
    Top,
    /// Center text vertically within the annotation.
    Middle,
    /// Align text to the bottom of the annotation.
    Bottom,
}

/// Border style options for the annotation border.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderStyle {
    /// Solid line border.
    Solid,
    /// Dashed line border.
    Dashed,
    /// Dotted line border.
    Dotted,
}

/// Configuration for free text annotation appearance rendering.
#[derive(Debug, Clone)]
pub struct FreeTextAppearanceConfig {
    /// Font name (e.g., "/Helv", "/Times-Roman").
    /// If None, will be extracted from the annotation's DA string or default to "/Helv".
    pub font_name: Option<String>,
    /// Font size in points.
    /// If None, will be extracted from the annotation's DA string or default to 12.0.
    pub font_size: Option<f32>,
    /// Text color.
    /// If None, will be extracted from the annotation's DA string or default to black.
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
    /// Border width in points (0.0 = no border).
    pub border_width: f32,
    /// Border color.
    /// If None and border_width > 0, defaults to black.
    pub border_color: Option<PdfColor>,
    /// Border style.
    pub border_style: BorderStyle,
    /// Background color.
    /// If None, no background is drawn.
    pub background_color: Option<PdfColor>,
    /// Whether to enable word wrapping.
    pub word_wrap: bool,
    /// Line spacing multiplier (1.0 = normal spacing).
    pub line_spacing: f32,
}

impl Default for FreeTextAppearanceConfig {
    fn default() -> Self {
        Self {
            font_name: None,
            font_size: None,
            text_color: None,
            horizontal_alignment: TextAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
            padding_left: 4.0,
            padding_right: 4.0,
            padding_top: 4.0,
            padding_bottom: 4.0,
            border_width: 1.0,
            border_color: Some(PdfColor::BLACK),
            border_style: BorderStyle::Solid,
            background_color: Some(PdfColor::new(255, 255, 200, 255)), // Light yellow background
            word_wrap: true,
            line_spacing: 1.2,
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

/// Builder for constructing and applying the visual appearance of a free text annotation.
///
/// This builder collects configuration and generates a PDF content stream
/// that renders the free text annotation with correct font, size, color, positioning,
/// borders, and backgrounds. The resulting appearance is used for both display and flattening.
pub struct FreeTextAppearanceBuilder<'a> {
    bindings: &'a dyn PdfiumLibraryBindings,
    annotation_handle: FPDF_ANNOTATION,
    text_content: String,
    da_string: Option<String>,
    config: FreeTextAppearanceConfig,
}

impl<'a> FreeTextAppearanceBuilder<'a> {
    /// Creates a new builder for the given annotation with text content and DA string.
    pub(crate) fn new(
        annotation_handle: FPDF_ANNOTATION,
        bindings: &'a dyn PdfiumLibraryBindings,
        text_content: String,
        da_string: Option<String>,
    ) -> Self {
        Self {
            bindings,
            annotation_handle,
            text_content,
            da_string,
            config: FreeTextAppearanceConfig::default(),
        }
    }

    /// Sets the font name.
    ///
    /// # Arguments
    /// * `font_name` - The PDF font name (e.g., "/Helv" for Helvetica)
    pub fn with_font_name(mut self, font_name: &str) -> Self {
        self.config.font_name = Some(font_name.to_string());
        self
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
    pub fn with_horizontal_alignment(mut self, alignment: TextAlignment) -> Self {
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
    pub fn with_padding_individual(mut self, left: f32, top: f32, right: f32, bottom: f32) -> Self {
        self.config.padding_left = left;
        self.config.padding_top = top;
        self.config.padding_right = right;
        self.config.padding_bottom = bottom;
        self
    }

    /// Sets the border width and color.
    pub fn with_border(mut self, width: f32, color: PdfColor) -> Self {
        self.config.border_width = width;
        self.config.border_color = Some(color);
        self
    }

    /// Removes the border.
    pub fn without_border(mut self) -> Self {
        self.config.border_width = 0.0;
        self.config.border_color = None;
        self
    }

    /// Sets the border style.
    pub fn with_border_style(mut self, style: BorderStyle) -> Self {
        self.config.border_style = style;
        self
    }

    /// Sets the background color.
    pub fn with_background(mut self, color: PdfColor) -> Self {
        self.config.background_color = Some(color);
        self
    }

    /// Removes the background.
    pub fn without_background(mut self) -> Self {
        self.config.background_color = None;
        self
    }

    /// Enables or disables word wrapping.
    pub fn with_word_wrap(mut self, enabled: bool) -> Self {
        self.config.word_wrap = enabled;
        self
    }

    /// Sets the line spacing multiplier.
    pub fn with_line_spacing(mut self, spacing: f32) -> Self {
        self.config.line_spacing = spacing;
        self
    }

    /// Applies the free text annotation appearance.
    ///
    /// This sets the normal appearance stream (`/AP /N`) of the free text annotation.
    /// The appearance is used for both display and flattening operations.
    ///
    /// # Errors
    ///
    /// Returns an error if PDFium fails to set the appearance stream.
    pub fn apply(self) -> Result<(), PdfiumError> {
        self.apply_with_mode(PdfAppearanceMode::Normal)
    }

    /// Applies the free text annotation appearance with a specific appearance mode.
    ///
    /// Most free text annotations only need the Normal appearance. RollOver and Down
    /// appearances are used for interactive hover/click states.
    pub fn apply_with_mode(self, mode: PdfAppearanceMode) -> Result<(), PdfiumError> {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
            console::log_1(&"🔧 FreeTextAppearanceBuilder::apply_with_mode()".into());
            console::log_1(&"═══════════════════════════════════════════════════════════".into());
        }

        let content_stream = self.build_content_stream()?;

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   Content stream length: {} bytes", content_stream.len()).into());
            if content_stream.len() > 200 {
                let preview: String = content_stream.chars().take(200).collect();
                console::log_1(&format!("   Content stream preview: {}...", preview).into());
            } else {
                console::log_1(&format!("   Content stream: {}", content_stream).into());
            }
        }

        let result = self.bindings.FPDFAnnot_SetAP_str(
            self.annotation_handle,
            mode.as_pdfium(),
            &content_stream,
        );

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   FPDFAnnot_SetAP_str returned: {} (1=success, 0=failure)", result).into());
            console::log_1(&format!("   is_true(result): {}", self.bindings.is_true(result)).into());
        }

        // Set the Appearance State (/AS) to match the mode
        let mode_str = match mode {
            PdfAppearanceMode::Normal => "/N",
            PdfAppearanceMode::RollOver => "/R",
            PdfAppearanceMode::Down => "/D",
        };
        let as_result = self.bindings.FPDFAnnot_SetStringValue_str(
            self.annotation_handle,
            "AS",
            mode_str,
        );

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   Setting /AS to '{}': {}", mode_str,
                if self.bindings.is_true(as_result) { "✅ success" } else { "❌ failed" }).into());
        }

        if self.bindings.is_true(result) {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"✅ FreeTextAppearanceBuilder::apply_with_mode() succeeded".into());
                console::log_1(&"═══════════════════════════════════════════════════════════".into());
            }
            Ok(())
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"❌ ERROR: FPDFAnnot_SetAP_str failed".into());
                console::log_1(&"   This usually means the annotation rect is invalid or PDFium rejected the appearance stream".into());
                console::log_1(&"═══════════════════════════════════════════════════════════".into());
            }
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Builds the PDF content stream for the free text annotation appearance.
    fn build_content_stream(&self) -> Result<String, PdfiumError> {
        // Get annotation bounds
        let (left, bottom, right, top) = self.get_annotation_bounds()?;

        // Parse DA string for font/size/color
        let da_components = self.parse_da_string()?;

        // Apply configuration overrides or use DA defaults
        let font_name = self.config.font_name.clone()
            .unwrap_or_else(|| da_components.font_name.clone());
        let font_size = self.config.font_size
            .unwrap_or(da_components.font_size);
        let text_color = self.config.text_color
            .unwrap_or(da_components.text_color);

        // Handle empty text
        if self.text_content.is_empty() {
            return Ok(String::new());
        }

        // Build content stream
        let mut stream = String::with_capacity(2048);

        // Save graphics state
        stream.push_str("q\n");

        // Draw background if specified
        if let Some(bg_color) = self.config.background_color {
            self.draw_background(&mut stream, left, bottom, right, top, bg_color);
        }

        // Draw border if specified
        if self.config.border_width > 0.0 {
            if let Some(border_color) = self.config.border_color {
                self.draw_border(&mut stream, left, bottom, right, top, border_color);
            }
        }

        // Begin text object
        stream.push_str("BT\n");

        // Set font and size
        stream.push_str(&format!("/{} {} Tf\n", font_name, font_size));

        // Set text color (RGB)
        let r = text_color.red() as f32 / 255.0;
        let g = text_color.green() as f32 / 255.0;
        let b = text_color.blue() as f32 / 255.0;
        stream.push_str(&format!("{:.4} {:.4} {:.4} rg\n", r, g, b));

        // Render text with layout
        self.render_text_layout(&self.text_content, font_size, left, bottom, right, top, &mut stream);

        // End text object
        stream.push_str("ET\n");

        // Restore graphics state
        stream.push_str("Q\n");

        Ok(stream)
    }

    /// Gets the annotation bounds from the annotation.
    fn get_annotation_bounds(&self) -> Result<(f32, f32, f32, f32), PdfiumError> {
        // Get annotation rectangle (left, bottom, right, top)
        let mut rect = FS_RECTF {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        };

        let success = self.bindings.FPDFAnnot_GetRect(self.annotation_handle, &mut rect);

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::console;
            console::log_1(&format!("   FPDFAnnot_GetRect returned: {} (1=success, 0=failure)", success).into());
            console::log_1(&format!("   Rect: left={:.2}, bottom={:.2}, right={:.2}, top={:.2}", 
                rect.left, rect.bottom, rect.right, rect.top).into());
        }

        if self.bindings.is_true(success) {
            let width = rect.right - rect.left;
            let height = rect.top - rect.bottom;
            
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&format!("   Rect dimensions: width={:.2}, height={:.2}", width, height).into());
            }

            // Validate that the rect has valid dimensions
            if width < 1.0 || height < 1.0 {
                #[cfg(target_arch = "wasm32")]
                {
                    use web_sys::console;
                    console::log_1(&"❌ ERROR: Annotation rect has invalid dimensions (width or height < 1.0)".into());
                }
                return Err(PdfiumError::PdfiumLibraryInternalError(
                    PdfiumInternalError::Unknown,
                ));
            }

            Ok((rect.left, rect.bottom, rect.right, rect.top))
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                use web_sys::console;
                console::log_1(&"❌ ERROR: FPDFAnnot_GetRect failed - annotation rect is not set".into());
            }
            Err(PdfiumError::PdfiumLibraryInternalError(
                PdfiumInternalError::Unknown,
            ))
        }
    }

    /// Parses the Default Appearance (DA) string from the annotation.
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

        let parts: Vec<&str> = da_string.split_whitespace().collect();

        if parts.len() < 7 {
            return Ok(DaComponents {
                font_name: "Helv".to_string(),
                font_size: 12.0,
                text_color: PdfColor::BLACK,
            });
        }

        // Extract font name (remove leading slash)
        let font_name = parts[0].trim_start_matches('/').to_string();

        // Extract font size
        let font_size = parts[1].parse::<f32>().unwrap_or(12.0);

        // Extract RGB color values (parts 3, 4, 5)
        let r = (parts[3].parse::<f32>().unwrap_or(0.0) * 255.0) as u8;
        let g = (parts[4].parse::<f32>().unwrap_or(0.0) * 255.0) as u8;
        let b = (parts[5].parse::<f32>().unwrap_or(0.0) * 255.0) as u8;

        Ok(DaComponents {
            font_name,
            font_size,
            text_color: PdfColor::new(r, g, b, 255),
        })
    }

    /// Draws the background rectangle.
    fn draw_background(&self, stream: &mut String, left: f32, bottom: f32, right: f32, top: f32, color: PdfColor) {
        let r = color.red() as f32 / 255.0;
        let g = color.green() as f32 / 255.0;
        let b = color.blue() as f32 / 255.0;

        stream.push_str(&format!("{:.4} {:.4} {:.4} rg\n", r, g, b));
        stream.push_str(&format!("{:.4} {:.4} {:.4} {:.4} re\n", left, bottom, right - left, top - bottom));
        stream.push_str("f\n");
    }

    /// Draws the border rectangle.
    fn draw_border(&self, stream: &mut String, left: f32, bottom: f32, right: f32, top: f32, color: PdfColor) {
        let r = color.red() as f32 / 255.0;
        let g = color.green() as f32 / 255.0;
        let b = color.blue() as f32 / 255.0;

        stream.push_str(&format!("{:.4} {:.4} {:.4} RG\n", r, g, b));

        // Set line width
        stream.push_str(&format!("{} w\n", self.config.border_width));

        // Set line style
        match self.config.border_style {
            BorderStyle::Solid => {
                stream.push_str("[] 0 d\n"); // Solid line
            }
            BorderStyle::Dashed => {
                stream.push_str("[3 2] 0 d\n"); // 3 on, 2 off
            }
            BorderStyle::Dotted => {
                stream.push_str("[1 2] 0 d\n"); // 1 on, 2 off
            }
        }

        // Draw rectangle border
        stream.push_str(&format!("{:.4} {:.4} {:.4} {:.4} re\n", left, bottom, right - left, top - bottom));
        stream.push_str("S\n");
    }

    /// Renders text with proper layout, wrapping, and alignment.
    fn render_text_layout(&self, text: &str, font_size: f32, left: f32, bottom: f32, right: f32, top: f32, stream: &mut String) {
        // Calculate text area with padding
        let text_left = left + self.config.padding_left;
        let text_right = right - self.config.padding_right;
        let text_bottom = bottom + self.config.padding_bottom;
        let text_top = top - self.config.padding_top;
        let text_width = text_right - text_left;
        let text_height = text_top - text_bottom;

        // Handle empty text area
        if text_width <= 0.0 || text_height <= 0.0 {
            return;
        }

        // Split text into lines
        let lines = if self.config.word_wrap {
            self.wrap_text(text, font_size, text_width)
        } else {
            vec![text.to_string()]
        };

        // Calculate line height
        let line_height = font_size * self.config.line_spacing;
        let total_text_height = lines.len() as f32 * line_height;

        // Calculate vertical starting position based on alignment
        let start_y = match self.config.vertical_alignment {
            VerticalAlignment::Top => text_top - font_size,
            VerticalAlignment::Middle => text_top - (text_height - total_text_height) / 2.0 - font_size,
            VerticalAlignment::Bottom => text_bottom + total_text_height - line_height,
        };

        // Render each line
        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let y = start_y - i as f32 * line_height;

            // Calculate horizontal position based on alignment
            let x = match self.config.horizontal_alignment {
                TextAlignment::Left => text_left,
                TextAlignment::Center => text_left + (text_width - self.estimate_text_width(&line, font_size)) / 2.0,
                TextAlignment::Right => text_right - self.estimate_text_width(&line, font_size),
            };

            // Position and draw text
            stream.push_str(&format!("{:.4} {:.4} Td\n", x, y));
            stream.push_str(&format!("({}) Tj\n", self.escape_pdf_string(line)));
        }
    }

    /// Wraps text into lines that fit within the given width.
    fn wrap_text(&self, text: &str, font_size: f32, max_width: f32) -> Vec<String> {
        let mut lines = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();

        let mut current_line = String::new();

        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            if self.estimate_text_width(&test_line, font_size) <= max_width {
                current_line = test_line;
            } else {
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        // Handle case where single word is too long
        if lines.is_empty() && !text.is_empty() {
            lines.push(text.to_string());
        }

        lines
    }

    /// Estimates the width of text using a simple character-based approximation.
    fn estimate_text_width(&self, text: &str, font_size: f32) -> f32 {
        // Simple estimation: average character width is about 0.5 * font_size
        // This is a rough approximation - real fonts vary
        text.chars().count() as f32 * font_size * 0.5
    }

    /// Escapes special characters in text for PDF content streams.
    fn escape_pdf_string(&self, text: &str) -> String {
        text.replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)")
    }
}
