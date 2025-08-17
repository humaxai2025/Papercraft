use printpdf::{IndirectFontRef, Mm, PdfLayerReference, Color, Rgb, Line, Point};
use crate::TextFormat;

#[derive(Debug, Clone)]
struct TextSegment {
    text: String,
    formats: Vec<TextFormat>,
}

#[derive(Clone)]
pub struct FontSystem {
    pub regular: IndirectFontRef,
    pub bold: IndirectFontRef,
    pub italic: IndirectFontRef,
    pub bold_italic: IndirectFontRef,
    pub code: IndirectFontRef,
}

pub struct TextRenderer {
    fonts: FontSystem,
    font_sizes: FontSizes,
    colors: ColorScheme,
}

pub struct FontSizes {
    pub h1: f32,
    pub h2: f32,
    pub h3: f32,
    pub h4: f32,
    pub h5: f32,
    pub h6: f32,
    pub body: f32,
    pub code: f32,
    pub small: f32,
    pub caption: f32,
}

pub struct ColorScheme {
    pub text: Color,
    pub heading: Color,
    pub code: Color,
    pub code_bg: Color,
    pub blockquote: Color,
    pub blockquote_border: Color,
    pub table_border: Color,
    pub table_header: Color,
    pub link: Color,
}

impl Default for FontSizes {
    fn default() -> Self {
        Self {
            h1: 32.0,  // Larger, more prominent headings
            h2: 26.0,
            h3: 22.0,
            h4: 18.0,
            h5: 16.0,
            h6: 14.0,
            body: 12.0,   // Slightly larger body text for readability
            code: 10.0,   // Larger code font
            small: 10.0,
            caption: 11.0,
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            text: Color::Rgb(Rgb::new(0.15, 0.15, 0.15, None)),        // Darker text for better readability
            heading: Color::Rgb(Rgb::new(0.05, 0.15, 0.3, None)),      // Professional dark blue headings
            code: Color::Rgb(Rgb::new(0.65, 0.15, 0.35, None)),        // Refined purple-red for code
            code_bg: Color::Rgb(Rgb::new(0.97, 0.97, 0.98, None)),     // Very light blue-gray background
            blockquote: Color::Rgb(Rgb::new(0.35, 0.35, 0.4, None)),   // Slightly darker gray
            blockquote_border: Color::Rgb(Rgb::new(0.3, 0.5, 0.8, None)), // Professional blue border
            table_border: Color::Rgb(Rgb::new(0.6, 0.6, 0.65, None)),  // Refined gray
            table_header: Color::Rgb(Rgb::new(0.05, 0.05, 0.1, None)), // Very dark for contrast
            link: Color::Rgb(Rgb::new(0.1, 0.35, 0.7, None)),          // Professional blue
        }
    }
}

impl TextRenderer {
    pub fn new(fonts: FontSystem) -> Self {
        Self {
            fonts,
            font_sizes: FontSizes::default(),
            colors: ColorScheme::default(),
        }
    }

    pub fn with_custom_sizes(fonts: FontSystem, font_sizes: FontSizes) -> Self {
        Self {
            fonts,
            font_sizes,
            colors: ColorScheme::default(),
        }
    }

    pub fn calculate_text_width(&self, text: &str, font_size: f32) -> Mm {
        // More accurate character width estimation based on font size
        let avg_char_width = font_size * 0.52; // Improved Helvetica character width ratio
        let width_pt = text.chars().map(|c| {
            match c {
                'i' | 'l' | 'j' | '!' | '|' | '.' | ',' | ':' | ';' => avg_char_width * 0.5,
                'm' | 'w' | 'M' | 'W' => avg_char_width * 1.3,
                't' | 'f' | 'r' => avg_char_width * 0.7,
                ' ' => avg_char_width * 0.6,
                _ => avg_char_width,
            }
        }).sum::<f32>();
        Mm(width_pt * 25.4 / 72.0) // Convert points to mm
    }

    pub fn calculate_line_height(&self, font_size: f32) -> Mm {
        // Professional line height - 1.5 for body text, 1.3 for headings
        let multiplier = if font_size > 16.0 { 1.3 } else { 1.5 };
        let line_height_pt = font_size * multiplier;
        Mm(line_height_pt * 25.4 / 72.0)
    }

    pub fn calculate_paragraph_spacing(&self, font_size: f32) -> Mm {
        // Enhanced paragraph spacing for better readability
        Mm((font_size * 1.0) * 25.4 / 72.0)
    }

    pub fn calculate_heading_spacing_before(&self, level: u8) -> Mm {
        // More generous spacing before headings for visual hierarchy
        match level {
            1 => Mm(24.0),  // Large space before H1
            2 => Mm(20.0),  // Good space before H2
            3 => Mm(16.0),  // Medium space before H3
            4 => Mm(14.0),  // Smaller space before H4
            5 => Mm(12.0),  // Minimal space before H5
            _ => Mm(10.0),  // Default for H6+
        }
    }

    pub fn calculate_heading_spacing_after(&self, level: u8) -> Mm {
        // Balanced spacing after headings
        match level {
            1 => Mm(16.0),  // Good space after H1
            2 => Mm(14.0),  // Good space after H2
            3 => Mm(12.0),  // Medium space after H3
            4 => Mm(10.0),  // Smaller space after H4
            5 => Mm(8.0),   // Minimal space after H5
            _ => Mm(6.0),   // Default for H6+
        }
    }

    pub fn wrap_text(&self, text: &str, max_width: Mm, font_size: f32) -> Vec<String> {
        let avg_char_width = font_size * 0.55; // More accurate for Helvetica
        let max_width_pt = max_width.0 * 72.0 / 25.4; // Convert mm to points
        let max_chars_per_line = (max_width_pt / avg_char_width) as usize;
        
        if max_chars_per_line == 0 {
            return vec![text.to_string()];
        }

        let mut lines = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_line = String::new();
        
        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            // Check if the test line fits
            if self.calculate_text_width(&test_line, font_size).0 <= max_width.0 {
                current_line = test_line;
            } else {
                // Current line doesn't fit, save it and start a new line
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                
                // Handle words that are longer than the line width
                if self.calculate_text_width(word, font_size).0 > max_width.0 {
                    let chars: Vec<char> = word.chars().collect();
                    let mut remaining = String::new();
                    
                    for char in chars {
                        let test_word = format!("{}{}", remaining, char);
                        if self.calculate_text_width(&test_word, font_size).0 <= max_width.0 {
                            remaining = test_word;
                        } else {
                            if !remaining.is_empty() {
                                lines.push(remaining);
                            }
                            remaining = char.to_string();
                        }
                    }
                    current_line = remaining;
                } else {
                    current_line = word.to_string();
                }
            }
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        if lines.is_empty() {
            lines.push(String::new());
        }
        
        lines
    }

    pub fn render_heading(
        &self,
        layer: &PdfLayerReference,
        text: &str,
        level: u8,
        x: Mm,
        y: Mm,
        max_width: Mm,
    ) -> (Mm, usize) {
        let font_size = match level {
            1 => self.font_sizes.h1,
            2 => self.font_sizes.h2,
            3 => self.font_sizes.h3,
            4 => self.font_sizes.h4,
            5 => self.font_sizes.h5,
            _ => self.font_sizes.h6,
        };

        // Set heading color
        layer.set_fill_color(self.colors.heading.clone());
        
        let lines = self.wrap_text(text, max_width, font_size);
        let line_height = self.calculate_line_height(font_size);
        let mut current_y = y;
        
        for line in &lines {
            layer.use_text(line, font_size, x, current_y, &self.fonts.bold);
            current_y -= line_height;
        }

        // Add underline for H1 and H2
        if level <= 2 && !lines.is_empty() {
            let underline_y = current_y + Mm(2.0);
            let underline_width = self.calculate_text_width(&lines[0], font_size);
            self.draw_line(layer, x, underline_y, x + underline_width, underline_y, 0.5);
        }
        
        // Reset text color
        layer.set_fill_color(self.colors.text.clone());
        
        let total_height = line_height * lines.len() as f32;
        (total_height, lines.len())
    }

    pub fn render_paragraph(
        &self,
        layer: &PdfLayerReference,
        text: &str,
        x: Mm,
        y: Mm,
        max_width: Mm,
        justify: bool,
    ) -> (Mm, usize) {
        self.render_formatted_text(layer, text, x, y, max_width, justify, &[], self.font_sizes.body)
    }

    pub fn render_formatted_text(
        &self,
        layer: &PdfLayerReference,
        text: &str,
        x: Mm,
        y: Mm,
        max_width: Mm,
        justify: bool,
        formats: &[TextFormat],
        font_size: f32,
    ) -> (Mm, usize) {
        let line_height = self.calculate_line_height(font_size);
        let lines = self.wrap_text(text, max_width, font_size);
        let mut current_y = y;
        
        // Select appropriate font based on formatting
        let font = self.get_font_for_formats(formats);
        
        // Set text color based on format
        if formats.contains(&TextFormat::Code) {
            layer.set_fill_color(self.colors.code.clone());
        } else {
            layer.set_fill_color(self.colors.text.clone());
        }
        
        for (i, line) in lines.iter().enumerate() {
            let is_last_line = i == lines.len() - 1;
            
            if justify && !is_last_line && lines.len() > 1 {
                self.render_justified_text_with_font(layer, line, x, current_y, max_width, font_size, font);
            } else {
                layer.use_text(line, font_size, x, current_y, font);
            }
            
            // Add strikethrough if needed
            if formats.contains(&TextFormat::Strikethrough) {
                let text_width = self.calculate_text_width(line, font_size);
                let strike_y = current_y + Mm(font_size * 0.3 * 25.4 / 72.0);
                self.draw_line(layer, x, strike_y, x + text_width, strike_y, 0.5);
            }
            
            current_y -= line_height;
        }
        
        // Reset text color
        layer.set_fill_color(self.colors.text.clone());
        
        let total_height = line_height * lines.len() as f32;
        (total_height, lines.len())
    }


    fn parse_formatting(&self, text: &str, base_formats: &[TextFormat]) -> Vec<TextSegment> {
        let mut segments = Vec::new();
        let mut current_text = String::new();
        let mut format_stack = base_formats.to_vec();
        let mut i = 0;
        let chars: Vec<char> = text.chars().collect();
        
        while i < chars.len() {
            if chars[i] == '[' {
                // Check for formatting markers
                let remaining: String = chars[i..].iter().collect();
                
                if remaining.starts_with("[BOLD_START]") {
                    if !current_text.is_empty() {
                        segments.push(TextSegment {
                            text: current_text.clone(),
                            formats: format_stack.clone(),
                        });
                        current_text.clear();
                    }
                    format_stack.push(TextFormat::Bold);
                    i += "[BOLD_START]".len();
                    continue;
                } else if remaining.starts_with("[BOLD_END]") {
                    if !current_text.is_empty() {
                        segments.push(TextSegment {
                            text: current_text.clone(),
                            formats: format_stack.clone(),
                        });
                        current_text.clear();
                    }
                    format_stack.retain(|f| !matches!(f, TextFormat::Bold));
                    i += "[BOLD_END]".len();
                    continue;
                } else if remaining.starts_with("[ITALIC_START]") {
                    if !current_text.is_empty() {
                        segments.push(TextSegment {
                            text: current_text.clone(),
                            formats: format_stack.clone(),
                        });
                        current_text.clear();
                    }
                    format_stack.push(TextFormat::Italic);
                    i += "[ITALIC_START]".len();
                    continue;
                } else if remaining.starts_with("[ITALIC_END]") {
                    if !current_text.is_empty() {
                        segments.push(TextSegment {
                            text: current_text.clone(),
                            formats: format_stack.clone(),
                        });
                        current_text.clear();
                    }
                    format_stack.retain(|f| !matches!(f, TextFormat::Italic));
                    i += "[ITALIC_END]".len();
                    continue;
                } else if remaining.starts_with("[STRIKE_START]") {
                    if !current_text.is_empty() {
                        segments.push(TextSegment {
                            text: current_text.clone(),
                            formats: format_stack.clone(),
                        });
                        current_text.clear();
                    }
                    format_stack.push(TextFormat::Strikethrough);
                    i += "[STRIKE_START]".len();
                    continue;
                } else if remaining.starts_with("[STRIKE_END]") {
                    if !current_text.is_empty() {
                        segments.push(TextSegment {
                            text: current_text.clone(),
                            formats: format_stack.clone(),
                        });
                        current_text.clear();
                    }
                    format_stack.retain(|f| !matches!(f, TextFormat::Strikethrough));
                    i += "[STRIKE_END]".len();
                    continue;
                } else if remaining.starts_with("[CODE_START]") {
                    if !current_text.is_empty() {
                        segments.push(TextSegment {
                            text: current_text.clone(),
                            formats: format_stack.clone(),
                        });
                        current_text.clear();
                    }
                    format_stack.push(TextFormat::Code);
                    i += "[CODE_START]".len();
                    continue;
                } else if remaining.starts_with("[CODE_END]") {
                    if !current_text.is_empty() {
                        segments.push(TextSegment {
                            text: current_text.clone(),
                            formats: format_stack.clone(),
                        });
                        current_text.clear();
                    }
                    format_stack.retain(|f| !matches!(f, TextFormat::Code));
                    i += "[CODE_END]".len();
                    continue;
                }
            }
            
            current_text.push(chars[i]);
            i += 1;
        }
        
        if !current_text.is_empty() {
            segments.push(TextSegment {
                text: current_text,
                formats: format_stack,
            });
        }
        
        if segments.is_empty() {
            segments.push(TextSegment {
                text: String::new(),
                formats: base_formats.to_vec(),
            });
        }
        
        segments
    }

    fn get_font_for_formats(&self, formats: &[TextFormat]) -> &IndirectFontRef {
        let has_bold = formats.contains(&TextFormat::Bold);
        let has_italic = formats.contains(&TextFormat::Italic);
        let has_code = formats.contains(&TextFormat::Code);
        
        if has_code {
            &self.fonts.code
        } else if has_bold && has_italic {
            &self.fonts.bold_italic
        } else if has_bold {
            &self.fonts.bold
        } else if has_italic {
            &self.fonts.italic
        } else {
            &self.fonts.regular
        }
    }

    pub fn render_code_block(
        &self,
        layer: &PdfLayerReference,
        text: &str,
        x: Mm,
        y: Mm,
        max_width: Mm,
        background: bool,
    ) -> (Mm, usize) {
        let font_size = self.font_sizes.code;
        let line_height = self.calculate_line_height(font_size);
        let lines: Vec<&str> = text.lines().collect();
        let padding = Mm(6.0); // More generous padding
        let line_number_width = Mm(15.0);
        let mut current_y = y;
        
        // Draw professional code block background
        if background {
            let bg_height = line_height * lines.len() as f32 + (padding * 2.0);
            
            // Main background with subtle gradient effect
            self.draw_rectangle(
                layer,
                x - padding,
                current_y + padding,
                max_width + (padding * 2.0),
                bg_height,
                Some(Color::Rgb(Rgb::new(0.96, 0.97, 0.98, None))), // Very light blue-gray
                Some(Color::Rgb(Rgb::new(0.8, 0.82, 0.85, None))), // Professional border
                1.0,
            );
            
            // Left accent border for code blocks
            self.draw_rectangle(
                layer,
                x - padding,
                current_y + padding,
                Mm(3.0),
                bg_height,
                Some(Color::Rgb(Rgb::new(0.3, 0.4, 0.6, None))), // Blue accent
                None,
                0.0,
            );
            
            // Line numbers background
            if lines.len() > 5 { // Only show line numbers for longer code blocks
                self.draw_rectangle(
                    layer,
                    x - padding + Mm(3.0),
                    current_y + padding,
                    line_number_width,
                    bg_height,
                    Some(Color::Rgb(Rgb::new(0.92, 0.93, 0.95, None))), // Darker background for line numbers
                    None,
                    0.0,
                );
            }
        }
        
        // Render code lines with syntax highlighting simulation
        for (line_num, line) in lines.iter().enumerate() {
            let x_offset = if background { 
                x + padding + if lines.len() > 5 { line_number_width + Mm(3.0) } else { Mm(0.0) }
            } else { 
                x 
            };
            
            // Render line numbers for longer code blocks
            if background && lines.len() > 5 {
                layer.set_fill_color(Color::Rgb(Rgb::new(0.6, 0.6, 0.6, None))); // Gray line numbers
                layer.use_text(
                    &format!("{:3}", line_num + 1),
                    font_size * 0.85,
                    x + Mm(1.0),
                    current_y,
                    &self.fonts.regular,
                );
            }
            
            // Simple syntax highlighting - keywords in bold, strings in different color
            if line.contains("fn ") || line.contains("function ") || line.contains("def ") || 
               line.contains("class ") || line.contains("struct ") || line.contains("impl ") {
                layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.3, 0.8, None))); // Blue for keywords
                layer.use_text(*line, font_size, x_offset, current_y, &self.fonts.bold);
            } else if line.trim_start().starts_with("//") || line.trim_start().starts_with("#") {
                layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.6, 0.5, None))); // Green for comments
                layer.use_text(*line, font_size, x_offset, current_y, &self.fonts.italic);
            } else {
                layer.set_fill_color(self.colors.code.clone());
                layer.use_text(*line, font_size, x_offset, current_y, &self.fonts.code);
            }
            
            current_y -= line_height;
        }
        
        // Reset text color
        layer.set_fill_color(self.colors.text.clone());
        
        let total_height = line_height * lines.len() as f32 + if background { padding * 2.0 } else { Mm(0.0) };
        (total_height, lines.len())
    }

    pub fn render_list_item(
        &self,
        layer: &PdfLayerReference,
        text: &str,
        bullet: &str,
        x: Mm,
        y: Mm,
        max_width: Mm,
        indent_level: usize,
    ) -> (Mm, usize) {
        let font_size = self.font_sizes.body;
        let line_height = self.calculate_line_height(font_size);
        let indent = Mm(16.0 * indent_level as f32); // Increased indent for better hierarchy
        let bullet_width = Mm(12.0); // Wider space for bullets
        
        // Enhanced bullet rendering with better typography
        let bullet_x = x + indent;
        let bullet_y_offset = line_height * 0.15; // Slight vertical adjustment for better alignment
        
        // Use different bullet styles based on type and nesting
        let enhanced_bullet = if bullet.starts_with(char::is_numeric) {
            // For numbered lists, keep as is but with better formatting
            bullet.to_string()
        } else {
            // Enhanced bullet characters for better visual hierarchy
            match indent_level {
                0 => "●".to_string(),      // Solid circle for first level
                1 => "◦".to_string(),      // Open circle for second level  
                2 => "▪".to_string(),      // Small square for third level
                _ => "▫".to_string(),      // Open square for deeper levels
            }
        };
        
        // Render bullet with proper color and positioning
        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.4, 0.6, None))); // Professional blue for bullets
        layer.use_text(
            &enhanced_bullet, 
            font_size,
            bullet_x, 
            y + bullet_y_offset, 
            &self.fonts.bold
        );
        
        // Reset text color for list content
        layer.set_fill_color(self.colors.text.clone());
        
        // Render text with appropriate wrapping and formatting
        let text_x = x + indent + bullet_width;
        let text_max_width = max_width - indent - bullet_width - Mm(4.0); // Extra margin
        
        let (height, lines) = self.render_formatted_text(
            layer,
            text,
            text_x,
            y,
            text_max_width,
            false,
            &[],
            font_size,
        );
        
        (height, lines)
    }

    pub fn render_blockquote(
        &self,
        layer: &PdfLayerReference,
        text: &str,
        x: Mm,
        y: Mm,
        max_width: Mm,
    ) -> (Mm, usize) {
        let font_size = self.font_sizes.body * 0.95; // Slightly smaller for quotes
        let border_width = Mm(4.0);
        let padding_left = Mm(12.0);
        let padding_vertical = Mm(6.0);
        let background_padding = Mm(8.0);
        
        // Calculate text area
        let text_x = x + border_width + padding_left;
        let text_width = max_width - border_width - padding_left - background_padding;
        
        // Render text with blockquote formatting (italic style for quotes)
        layer.set_fill_color(self.colors.blockquote.clone());
        let (text_height, line_count) = self.render_formatted_text(
            layer,
            text,
            text_x,
            y - padding_vertical,
            text_width,
            false,
            &[TextFormat::Italic], // Make blockquotes italic
            font_size,
        );
        
        let total_height = text_height + padding_vertical * 2.0;
        
        // Draw subtle background rectangle
        self.draw_rectangle(
            layer,
            x,
            y,
            max_width,
            total_height,
            Some(Color::Rgb(Rgb::new(0.98, 0.98, 0.99, None))), // Very light background
            None,
            0.0,
        );
        
        // Draw thick left border with gradient effect (multiple lines)
        layer.set_outline_color(self.colors.blockquote_border.clone());
        self.draw_line(
            layer,
            x + border_width / 2.0,
            y + Mm(1.0),
            x + border_width / 2.0,
            y - total_height + Mm(1.0),
            border_width.0,
        );
        
        // Add a thinner highlight line next to the main border
        layer.set_outline_color(Color::Rgb(Rgb::new(0.6, 0.75, 0.9, None))); // Light blue accent
        self.draw_line(
            layer,
            x + border_width + Mm(1.0),
            y + Mm(1.0),
            x + border_width + Mm(1.0),
            y - total_height + Mm(1.0),
            1.0,
        );
        
        // Add quotation mark decoration at the top
        layer.set_fill_color(Color::Rgb(Rgb::new(0.7, 0.7, 0.75, None)));
        layer.use_text(
            "\"", // Opening quote
            font_size * 1.8,
            x + border_width + Mm(2.0),
            y - Mm(2.0),
            &self.fonts.italic,
        );
        
        // Reset colors
        layer.set_fill_color(self.colors.text.clone());
        
        (total_height + Mm(4.0), line_count)
    }

    fn render_justified_text(
        &self,
        layer: &PdfLayerReference,
        text: &str,
        x: Mm,
        y: Mm,
        max_width: Mm,
        font_size: f32,
    ) {
        self.render_justified_text_with_font(layer, text, x, y, max_width, font_size, &self.fonts.regular);
    }

    fn render_justified_text_with_font(
        &self,
        layer: &PdfLayerReference,
        text: &str,
        x: Mm,
        y: Mm,
        max_width: Mm,
        font_size: f32,
        font: &IndirectFontRef,
    ) {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() <= 1 {
            layer.use_text(text, font_size, x, y, font);
            return;
        }

        // Calculate total text width without spaces
        let text_without_spaces = words.join("");
        let text_width = self.calculate_text_width(&text_without_spaces, font_size);
        let available_space = max_width - text_width;
        let space_count = words.len() - 1;
        
        if space_count == 0 || available_space.0 <= 0.0 {
            layer.use_text(text, font_size, x, y, font);
            return;
        }
        
        let space_width = available_space.0 / space_count as f32;

        // Render words with calculated spacing
        let mut current_x = x;
        for word in words.iter() {
            layer.use_text(*word, font_size, current_x, y, font);
            let word_width = self.calculate_text_width(word, font_size);
            current_x += word_width + Mm(space_width);
        }
    }

    pub fn draw_line(&self, layer: &PdfLayerReference, x1: Mm, y1: Mm, x2: Mm, y2: Mm, thickness: f32) {
        layer.set_outline_thickness(thickness);
        let line = Line {
            points: vec![
                (Point::new(x1, y1), false),
                (Point::new(x2, y2), false),
            ],
            is_closed: false,
        };
        layer.add_line(line);
    }

    pub fn draw_rectangle(&self, layer: &PdfLayerReference, x: Mm, y: Mm, width: Mm, height: Mm, fill_color: Option<Color>, border_color: Option<Color>, border_thickness: f32) {
        if let Some(fill) = fill_color {
            layer.set_fill_color(fill);
        }
        if let Some(border) = border_color {
            layer.set_outline_color(border);
            layer.set_outline_thickness(border_thickness);
        }
        
        // Draw rectangle using lines for better control
        let top_left = Point::new(x, y);
        let top_right = Point::new(x + width, y);
        let bottom_right = Point::new(x + width, y - height);
        let bottom_left = Point::new(x, y - height);
        
        let rect = Line {
            points: vec![
                (top_left, false),
                (top_right, false),
                (bottom_right, false),
                (bottom_left, false),
            ],
            is_closed: true,
        };
        
        layer.add_line(rect);
    }

    pub fn get_font_size(&self, element_type: &str) -> f32 {
        match element_type {
            "h1" => self.font_sizes.h1,
            "h2" => self.font_sizes.h2,
            "h3" => self.font_sizes.h3,
            "h4" => self.font_sizes.h4,
            "h5" => self.font_sizes.h5,
            "h6" => self.font_sizes.h6,
            "code" => self.font_sizes.code,
            "small" => self.font_sizes.small,
            "caption" => self.font_sizes.caption,
            _ => self.font_sizes.body,
        }
    }

    pub fn get_colors(&self) -> &ColorScheme {
        &self.colors
    }

    pub fn get_fonts(&self) -> &FontSystem {
        &self.fonts
    }

    pub fn get_font_sizes(&self) -> &FontSizes {
        &self.font_sizes
    }

    // Helper method for rendering with proper spacing
    pub fn render_with_spacing(
        &self,
        layer: &PdfLayerReference,
        text: &str,
        x: Mm,
        y: Mm,
        max_width: Mm,
        element_type: &str,
    ) -> (Mm, usize) {
        match element_type {
            "blockquote" => self.render_blockquote(layer, text, x, y, max_width),
            "code_block" => self.render_code_block(layer, text, x, y, max_width, true),
            _ => self.render_paragraph(layer, text, x, y, max_width, true),
        }
    }
}