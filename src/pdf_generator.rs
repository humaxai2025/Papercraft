use crate::{DocumentElement, ElementType, ListType, TableData, TextFormat};
use crate::text_renderer::{TextRenderer, FontSystem};
use crate::image_handler::ImageHandler;
use anyhow::{Context, Result};
use printpdf::{BuiltinFont, Mm, PdfDocument, Color, Rgb, PdfLayerReference};
use std::fs::File;
use std::io::BufWriter;

pub struct PageLayout {
    pub width: Mm,
    pub height: Mm,
    pub margin_top: Mm,
    pub margin_bottom: Mm,
    pub margin_left: Mm,
    pub margin_right: Mm,
    pub header_height: Mm,
    pub footer_height: Mm,
}

impl Default for PageLayout {
    fn default() -> Self {
        Self {
            width: Mm(210.0),  // A4 width
            height: Mm(297.0), // A4 height
            margin_top: Mm(30.0),     // Larger top margin for professional look
            margin_bottom: Mm(30.0),  // Larger bottom margin
            margin_left: Mm(25.0),    // Comfortable left margin
            margin_right: Mm(25.0),   // Comfortable right margin
            header_height: Mm(18.0),  // Slightly larger header
            footer_height: Mm(18.0),  // Slightly larger footer
        }
    }
}

pub struct PageInfo {
    pub current_page: usize,
    pub total_pages: usize,
    pub title: String,
}

pub struct PdfGenerator {
    image_handler: ImageHandler,
    layout: PageLayout,
    page_info: PageInfo,
}

impl PdfGenerator {
    pub fn new() -> Result<Self> {
        Ok(Self {
            image_handler: ImageHandler::new(),
            layout: PageLayout::default(),
            page_info: PageInfo {
                current_page: 1,
                total_pages: 0,
                title: "Professional Document".to_string(),
            },
        })
    }

    pub fn generate_pdf(&mut self, elements: &[DocumentElement], output_path: &str) -> Result<()> {
        // First pass: count pages (simplified estimation)
        self.page_info.total_pages = self.estimate_total_pages(elements);
        
        let (doc, page1, layer1) = PdfDocument::new("Professional Markdown to PDF", self.layout.width, self.layout.height, "Layer 1");
        let mut current_layer_ref = doc.get_page(page1).get_layer(layer1);

        // Load professional fonts
        let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
        let font_italic = doc.add_builtin_font(BuiltinFont::HelveticaOblique)?;
        let font_bold_italic = doc.add_builtin_font(BuiltinFont::HelveticaBoldOblique)?;
        let font_code = doc.add_builtin_font(BuiltinFont::Courier)?;

        let font_system = FontSystem {
            regular: font_regular,
            bold: font_bold,
            italic: font_italic,
            bold_italic: font_bold_italic,
            code: font_code,
        };

        let text_renderer = TextRenderer::new(font_system.clone());
        
        // Professional page layout
        let content_width = self.get_content_width();
        let mut y_position = self.get_content_start_y();
        
        // Add header and footer to first page
        self.add_header_footer(&current_layer_ref, self.page_info.current_page, &font_system, &text_renderer);

        let mut list_counters: Vec<u64> = Vec::new();
        let mut footnotes = Vec::new();
        let mut in_list = false;
        let mut _current_list_type: Option<ListType> = None;

        for element in elements {
            // Check if we need a new page
            let min_space_needed = self.calculate_element_height(element, &text_renderer, content_width);
            if y_position - min_space_needed < self.layout.margin_bottom + self.layout.footer_height + Mm(10.0) {
                self.page_info.current_page += 1;
                let (new_page, new_layer) = doc.add_page(self.layout.width, self.layout.height, "Layer 1");
                current_layer_ref = doc.get_page(new_page).get_layer(new_layer);
                y_position = self.get_content_start_y();
                
                // Add header and footer to new page
                self.add_header_footer(&current_layer_ref, self.page_info.current_page, &font_system, &text_renderer);
            }

            // Handle list state changes
            match &element.element_type {
                ElementType::ListItem | ElementType::TaskListItem => {
                    if !in_list {
                        in_list = true;
                        _current_list_type = element.list_type.clone();
                        if matches!(element.list_type, Some(ListType::Ordered(_))) {
                            if let Some(ListType::Ordered(start)) = &element.list_type {
                                list_counters.push(*start);
                            }
                        }
                        y_position -= Mm(4.0); // Space before list
                    }
                }
                _ => {
                    if in_list {
                        in_list = false;
                        _current_list_type = None;
                        list_counters.clear();
                        y_position -= Mm(6.0); // Space after list
                    }
                }
            }
            
            match element.element_type {
                ElementType::Heading => {
                    let level = element.level.unwrap_or(1);
                    y_position -= text_renderer.calculate_heading_spacing_before(level);
                    
                    let (height_used, _) = text_renderer.render_heading(
                        &current_layer_ref,
                        &self.process_formatting(&element.content, &element.formatting),
                        level,
                        self.layout.margin_left,
                        y_position,
                        content_width,
                    );
                    
                    y_position -= height_used + text_renderer.calculate_heading_spacing_after(level);
                }

                ElementType::Paragraph => {
                    if !in_list {
                        y_position -= text_renderer.calculate_paragraph_spacing(text_renderer.get_font_size("body"));
                    }
                    
                    let processed_text = self.process_formatting(&element.content, &element.formatting);
                    let (height_used, _) = text_renderer.render_formatted_text(
                        &current_layer_ref,
                        &processed_text,
                        self.layout.margin_left,
                        y_position,
                        content_width,
                        true,
                        &element.formatting,
                        text_renderer.get_font_size("body"),
                    );
                    
                    y_position -= height_used;
                    if !in_list {
                        y_position -= text_renderer.calculate_paragraph_spacing(text_renderer.get_font_size("body"));
                    }
                }

                ElementType::ListItem => {
                    let list_type = element.list_type.as_ref().unwrap_or(&ListType::Bullet);
                    let bullet = match list_type {
                        ListType::Bullet => "•".to_string(),
                        ListType::Ordered(_) => {
                            let counter = if let Some(last) = list_counters.last_mut() {
                                let current = *last;
                                *last += 1;
                                current
                            } else {
                                1
                            };
                            format!("{}.", counter)
                        }
                        ListType::Task => "☐".to_string(),
                    };
                    
                    let (height_used, _) = text_renderer.render_list_item(
                        &current_layer_ref,
                        &self.process_formatting(&element.content, &element.formatting),
                        &bullet,
                        self.layout.margin_left,
                        y_position,
                        content_width,
                        0,
                    );
                    
                    y_position -= height_used + Mm(3.0);
                }

                ElementType::TaskListItem => {
                    let checkbox = if element.is_checked.unwrap_or(false) { "☑" } else { "☐" };
                    
                    let (height_used, _) = text_renderer.render_list_item(
                        &current_layer_ref,
                        &self.process_formatting(&element.content, &element.formatting),
                        checkbox,
                        self.layout.margin_left,
                        y_position,
                        content_width,
                        0,
                    );
                    
                    y_position -= height_used + Mm(3.0);
                }

                ElementType::BlockQuote => {
                    y_position -= Mm(8.0);
                    
                    let (height_used, _) = text_renderer.render_blockquote(
                        &current_layer_ref,
                        &self.process_formatting(&element.content, &element.formatting),
                        self.layout.margin_left,
                        y_position,
                        content_width,
                    );
                    
                    y_position -= height_used + Mm(8.0);
                }

                ElementType::Table => {
                    if let Some(table_data) = &element.table_data {
                        y_position -= Mm(10.0);
                        let table_height = self.render_professional_table(
                            &current_layer_ref,
                            table_data,
                            self.layout.margin_left,
                            y_position,
                            content_width,
                            &font_system,
                            &text_renderer,
                        )?;
                        y_position -= table_height + Mm(10.0);
                    }
                }

                ElementType::Code => {
                    let font_size = text_renderer.get_font_size("code");
                    current_layer_ref.set_fill_color(text_renderer.get_colors().code.clone());
                    current_layer_ref.use_text(
                        &format!("`{}`", element.content),
                        font_size,
                        self.layout.margin_left,
                        y_position,
                        &font_system.code,
                    );
                    current_layer_ref.set_fill_color(text_renderer.get_colors().text.clone());
                    y_position -= text_renderer.calculate_line_height(font_size) + Mm(2.0);
                }

                ElementType::CodeBlock => {
                    y_position -= Mm(10.0);
                    
                    let (height_used, _) = text_renderer.render_code_block(
                        &current_layer_ref,
                        &element.content,
                        self.layout.margin_left,
                        y_position,
                        content_width,
                        true,
                    );
                    
                    y_position -= height_used + Mm(10.0);
                }

                ElementType::Link => {
                    let link_text = if let Some(url) = &element.url {
                        format!("{} ({})", element.content, url)
                    } else {
                        element.content.clone()
                    };
                    
                    let font_size = text_renderer.get_font_size("body");
                    current_layer_ref.set_fill_color(text_renderer.get_colors().link.clone());
                    current_layer_ref.use_text(
                        &link_text,
                        font_size,
                        self.layout.margin_left,
                        y_position,
                        &font_system.regular,
                    );
                    
                    // Add underline to links
                    let text_width = text_renderer.calculate_text_width(&link_text, font_size);
                    text_renderer.draw_line(
                        &current_layer_ref,
                        self.layout.margin_left,
                        y_position - Mm(2.0),
                        self.layout.margin_left + text_width,
                        y_position - Mm(2.0),
                        0.5,
                    );
                    
                    current_layer_ref.set_fill_color(text_renderer.get_colors().text.clone());
                    y_position -= text_renderer.calculate_line_height(font_size) + Mm(2.0);
                }

                ElementType::Image => {
                    if let Some(url) = &element.url {
                        y_position -= Mm(16.0); // More space before image
                        
                        // Professional image sizing with better proportions
                        let max_image_width = content_width * 0.85; // Slightly smaller for better margins
                        let max_image_height = Mm(180.0); // Allow larger images
                        
                        match self.image_handler.add_image_to_pdf_sync(
                            &current_layer_ref,
                            url,
                            self.layout.margin_left + (content_width - max_image_width) / 2.0,
                            y_position,
                            max_image_width,
                            max_image_height,
                        ) {
                            Ok(image_height) => {
                                // Image positioning without unnecessary boxes
                                
                                // Simple image caption without background box
                                if !element.content.is_empty() {
                                    let caption_y = y_position - image_height - Mm(8.0);
                                    let caption_font_size = text_renderer.get_font_size("caption");
                                    let caption_width = text_renderer.calculate_text_width(&element.content, caption_font_size);
                                    let caption_x = self.layout.margin_left + (content_width - caption_width) / 2.0;
                                    
                                    // Simple caption text
                                    current_layer_ref.set_fill_color(text_renderer.get_colors().blockquote.clone());
                                    current_layer_ref.use_text(
                                        &format!("Figure: {}", element.content),
                                        caption_font_size,
                                        caption_x,
                                        caption_y,
                                        &font_system.italic,
                                    );
                                    current_layer_ref.set_fill_color(text_renderer.get_colors().text.clone());
                                    y_position = caption_y - text_renderer.calculate_line_height(caption_font_size) - Mm(12.0);
                                } else {
                                    y_position -= image_height + Mm(16.0);
                                }
                            }
                            Err(e) => {
                                // Professional placeholder for missing images
                                let placeholder_height = Mm(60.0);
                                let placeholder_width = content_width * 0.7;
                                let placeholder_x = self.layout.margin_left + (content_width - placeholder_width) / 2.0;
                                
                                // Draw placeholder background
                                text_renderer.draw_rectangle(
                                    &current_layer_ref,
                                    placeholder_x,
                                    y_position,
                                    placeholder_width,
                                    placeholder_height,
                                    Some(Color::Rgb(Rgb::new(0.95, 0.95, 0.95, None))), // Light gray
                                    Some(Color::Rgb(Rgb::new(0.8, 0.8, 0.8, None))), // Dashed border effect
                                    1.0,
                                );
                                
                                // Placeholder text
                                let placeholder_text = "Image not available";
                                let error_text = format!("Error: {}", e.to_string().chars().take(50).collect::<String>());
                                
                                current_layer_ref.set_fill_color(text_renderer.get_colors().blockquote.clone());
                                current_layer_ref.use_text(
                                    placeholder_text,
                                    text_renderer.get_font_size("body"),
                                    placeholder_x + Mm(8.0),
                                    y_position - Mm(20.0),
                                    &font_system.bold,
                                );
                                
                                current_layer_ref.use_text(
                                    &error_text,
                                    text_renderer.get_font_size("small"),
                                    placeholder_x + Mm(8.0),
                                    y_position - Mm(35.0),
                                    &font_system.italic,
                                );
                                
                                eprintln!("Image loading failed: {}", e);
                                current_layer_ref.set_fill_color(text_renderer.get_colors().text.clone());
                                y_position -= placeholder_height + Mm(16.0);
                            }
                        }
                    }
                }

                ElementType::HorizontalRule => {
                    y_position -= Mm(16.0);
                    
                    // Draw professional decorative horizontal rule with gradient effect
                    let rule_width = content_width * 0.8;
                    let rule_x = self.layout.margin_left + (content_width - rule_width) / 2.0;
                    
                    // Main rule line
                    current_layer_ref.set_outline_color(Color::Rgb(Rgb::new(0.4, 0.5, 0.7, None)));
                    text_renderer.draw_line(
                        &current_layer_ref,
                        rule_x,
                        y_position,
                        rule_x + rule_width,
                        y_position,
                        1.5,
                    );
                    
                    // Add decorative dots at the ends
                    current_layer_ref.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.5, 0.7, None)));
                    current_layer_ref.use_text(
                        "●",
                        text_renderer.get_font_size("small"),
                        rule_x - Mm(4.0),
                        y_position + Mm(1.0),
                        &text_renderer.get_fonts().regular,
                    );
                    current_layer_ref.use_text(
                        "●",
                        text_renderer.get_font_size("small"),
                        rule_x + rule_width + Mm(1.0),
                        y_position + Mm(1.0),
                        &text_renderer.get_fonts().regular,
                    );
                    
                    // Reset color
                    current_layer_ref.set_fill_color(text_renderer.get_colors().text.clone());
                    y_position -= Mm(16.0);
                }

                ElementType::Footnote => {
                    footnotes.push(element.clone());
                }

                ElementType::FootnoteReference => {
                    let reference_text = format!("[{}]", element.content);
                    let font_size = text_renderer.get_font_size("small");
                    current_layer_ref.set_fill_color(text_renderer.get_colors().link.clone());
                    current_layer_ref.use_text(
                        &reference_text,
                        font_size,
                        self.layout.margin_left,
                        y_position + Mm(2.0), // Superscript effect
                        &font_system.regular,
                    );
                    current_layer_ref.set_fill_color(text_renderer.get_colors().text.clone());
                }

                _ => {
                    y_position -= Mm(4.0);
                }
            }
        }

        // Add professional footnotes section
        if !footnotes.is_empty() {
            y_position -= Mm(24.0);
            
            // Footnotes separator line
            text_renderer.draw_line(
                &current_layer_ref,
                self.layout.margin_left,
                y_position,
                self.layout.margin_left + content_width * 0.3,
                y_position,
                1.0,
            );
            y_position -= Mm(8.0);
            
            // Footnotes title
            current_layer_ref.set_fill_color(text_renderer.get_colors().heading.clone());
            current_layer_ref.use_text(
                "References",
                text_renderer.get_font_size("h6"),
                self.layout.margin_left,
                y_position,
                &font_system.bold,
            );
            current_layer_ref.set_fill_color(text_renderer.get_colors().text.clone());
            y_position -= Mm(12.0);
            
            for footnote in footnotes {
                if let Some(label) = &footnote.url {
                    let footnote_text = format!("[{}]: {}", label, footnote.content);
                    let (height_used, _) = text_renderer.render_paragraph(
                        &current_layer_ref,
                        &footnote_text,
                        self.layout.margin_left,
                        y_position,
                        content_width,
                        false,
                    );
                    y_position -= height_used + Mm(4.0);
                }
            }
        }

        // Save the professional PDF
        doc.save(&mut BufWriter::new(
            File::create(output_path).context("Failed to create output file")?,
        ))?;

        Ok(())
    }

    fn get_content_width(&self) -> Mm {
        self.layout.width - self.layout.margin_left - self.layout.margin_right
    }

    fn get_content_start_y(&self) -> Mm {
        self.layout.height - self.layout.margin_top - self.layout.header_height
    }

    fn add_header_footer(&self, layer: &PdfLayerReference, page_num: usize, font_system: &FontSystem, text_renderer: &TextRenderer) {
        let colors = text_renderer.get_colors();
        
        // Clean Header without background box
        let header_y = self.layout.height - self.layout.margin_top + Mm(8.0);
        
        // Header title
        layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.3, 0.5, None))); // Professional blue
        layer.use_text(
            &self.page_info.title,
            text_renderer.get_font_size("small"),
            self.layout.margin_left,
            header_y,
            &font_system.bold,
        );
        
        // Remove date/time from header
        
        // Professional header line with gradient effect
        layer.set_outline_color(Color::Rgb(Rgb::new(0.3, 0.4, 0.6, None)));
        text_renderer.draw_line(
            layer,
            self.layout.margin_left,
            header_y - Mm(4.0),
            self.layout.width - self.layout.margin_right,
            header_y - Mm(4.0),
            1.0,
        );
        
        // Clean Footer without background box
        let footer_y = self.layout.margin_bottom - Mm(3.0);
        
        // Footer line above text
        layer.set_outline_color(Color::Rgb(Rgb::new(0.3, 0.4, 0.6, None)));
        text_renderer.draw_line(
            layer,
            self.layout.margin_left,
            footer_y + Mm(5.0),
            self.layout.width - self.layout.margin_right,
            footer_y + Mm(5.0),
            1.0,
        );
        
        // Professional page numbering
        let page_text = format!("Page {} of {}", page_num, self.page_info.total_pages);
        let page_width = text_renderer.calculate_text_width(&page_text, text_renderer.get_font_size("small"));
        layer.set_fill_color(colors.text.clone());
        layer.use_text(
            &page_text,
            text_renderer.get_font_size("small"),
            self.layout.width - self.layout.margin_right - page_width,
            footer_y,
            &font_system.regular,
        );
        
        // Footer branding
        let branding = "Generated by Professional MD-to-PDF Converter";
        layer.set_fill_color(colors.blockquote.clone());
        layer.use_text(
            branding,
            text_renderer.get_font_size("small") * 0.85,
            self.layout.margin_left,
            footer_y,
            &font_system.italic,
        );
    }

    fn estimate_total_pages(&self, elements: &[DocumentElement]) -> usize {
        // Simple estimation - count elements and estimate space usage
        let estimated_height_per_page = self.get_content_start_y() - self.layout.margin_bottom - self.layout.footer_height;
        let mut total_estimated_height = Mm(0.0);
        
        for element in elements {
            let estimated_height = match element.element_type {
                ElementType::Heading => Mm(20.0),
                ElementType::Paragraph => Mm(15.0),
                ElementType::ListItem | ElementType::TaskListItem => Mm(8.0),
                ElementType::BlockQuote => Mm(20.0),
                ElementType::Table => Mm(50.0),
                ElementType::CodeBlock => Mm(30.0),
                ElementType::Image => Mm(100.0),
                ElementType::HorizontalRule => Mm(24.0),
                _ => Mm(10.0),
            };
            total_estimated_height += estimated_height;
        }
        
        ((total_estimated_height.0 / estimated_height_per_page.0).ceil() as usize).max(1)
    }
    
    fn calculate_element_height(&self, element: &DocumentElement, text_renderer: &TextRenderer, content_width: Mm) -> Mm {
        match element.element_type {
            ElementType::Heading => {
                let level = element.level.unwrap_or(1);
                text_renderer.calculate_heading_spacing_before(level) + 
                text_renderer.calculate_line_height(text_renderer.get_font_size(&format!("h{}", level))) +
                text_renderer.calculate_heading_spacing_after(level)
            }
            ElementType::Paragraph => {
                let font_size = text_renderer.get_font_size("body");
                let lines = text_renderer.wrap_text(&element.content, content_width, font_size);
                text_renderer.calculate_line_height(font_size) * lines.len() as f32 + Mm(8.0)
            }
            ElementType::Table => Mm(60.0), // Conservative estimate
            ElementType::Image => Mm(120.0), // Conservative estimate
            ElementType::CodeBlock => {
                let lines = element.content.lines().count();
                text_renderer.calculate_line_height(text_renderer.get_font_size("code")) * lines as f32 + Mm(20.0)
            }
            _ => Mm(15.0),
        }
    }

    fn render_professional_table(
        &self,
        layer: &printpdf::PdfLayerReference,
        table_data: &TableData,
        x: Mm,
        y: Mm,
        max_width: Mm,
        font_system: &FontSystem,
        text_renderer: &TextRenderer,
    ) -> Result<Mm> {
        let cell_padding = Mm(4.0);
        let min_cell_height = Mm(15.0);
        let border_thickness = 0.8;
        
        let col_count = table_data.headers.len().max(
            table_data.rows.iter().map(|row| row.len()).max().unwrap_or(0)
        );
        
        if col_count == 0 {
            return Ok(Mm(0.0));
        }
        
        let cell_width = max_width / col_count as f32;
        let row_count = if table_data.headers.is_empty() { 0 } else { 1 } + table_data.rows.len();
        
        // Calculate dynamic row heights based on content
        let mut row_heights = Vec::new();
        
        // Header height
        if !table_data.headers.is_empty() {
            let mut max_lines = 1;
            for header in &table_data.headers {
                let lines = text_renderer.wrap_text(header, cell_width - cell_padding * 2.0, text_renderer.get_font_size("body"));
                max_lines = max_lines.max(lines.len());
            }
            let header_height = text_renderer.calculate_line_height(text_renderer.get_font_size("body")) * max_lines as f32 + cell_padding * 2.0;
            row_heights.push(header_height.max(min_cell_height));
        }
        
        // Row heights
        for row in &table_data.rows {
            let mut max_lines = 1;
            for cell in row {
                let lines = text_renderer.wrap_text(cell, cell_width - cell_padding * 2.0, text_renderer.get_font_size("body"));
                max_lines = max_lines.max(lines.len());
            }
            let row_height = text_renderer.calculate_line_height(text_renderer.get_font_size("body")) * max_lines as f32 + cell_padding * 2.0;
            row_heights.push(row_height.max(min_cell_height));
        }
        
        let total_height: Mm = row_heights.iter().fold(Mm(0.0), |acc, &h| acc + h);
        
        let mut current_y = y;
        let mut row_index = 0;
        
        // Set table colors
        let colors = text_renderer.get_colors();
        layer.set_outline_color(colors.table_border.clone());
        layer.set_outline_thickness(border_thickness);
        
        // Draw table outer border with rounded corners effect
        text_renderer.draw_rectangle(
            layer,
            x,
            current_y,
            max_width,
            total_height,
            None,
            Some(colors.table_border.clone()),
            border_thickness * 1.5,
        );
        
        // Draw headers with professional background
        if !table_data.headers.is_empty() {
            let header_height = row_heights[row_index];
            
            // Header background with gradient-like effect (dark header)
            text_renderer.draw_rectangle(
                layer,
                x,
                current_y,
                max_width,
                header_height,
                Some(Color::Rgb(Rgb::new(0.25, 0.35, 0.55, None))), // Professional blue-gray
                None,
                0.0,
            );
            
            for (i, header) in table_data.headers.iter().enumerate() {
                let cell_x = x + cell_width * i as f32;
                
                // Vertical line between headers
                if i > 0 {
                    text_renderer.draw_line(
                        layer,
                        cell_x,
                        current_y,
                        cell_x,
                        current_y - header_height,
                        border_thickness,
                    );
                }
                
                // Header text in white for contrast
                layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None))); // White text
                let wrapped_text = text_renderer.wrap_text(header, cell_width - cell_padding * 2.0, text_renderer.get_font_size("body"));
                let line_height = text_renderer.calculate_line_height(text_renderer.get_font_size("body"));
                let mut text_y = current_y - cell_padding - line_height * 0.8;
                
                for line in wrapped_text {
                    layer.use_text(
                        &line,
                        text_renderer.get_font_size("body"),
                        cell_x + cell_padding,
                        text_y,
                        &font_system.bold,
                    );
                    text_y -= line_height;
                }
            }
            current_y -= header_height;
            row_index += 1;
            
            // Thick horizontal line after headers
            text_renderer.draw_line(
                layer,
                x,
                current_y,
                x + max_width,
                current_y,
                border_thickness * 2.0,
            );
        }
        
        // Draw rows with professional styling
        layer.set_fill_color(colors.text.clone());
        for (row_idx, row) in table_data.rows.iter().enumerate() {
            let row_height = row_heights[row_index];
            
            // Professional alternating row colors
            if row_idx % 2 == 1 {
                text_renderer.draw_rectangle(
                    layer,
                    x,
                    current_y,
                    max_width,
                    row_height,
                    Some(Color::Rgb(Rgb::new(0.97, 0.98, 0.99, None))), // Very light blue
                    None,
                    0.0,
                );
            }
            
            for (i, cell) in row.iter().enumerate() {
                if i >= col_count { break; }
                let cell_x = x + cell_width * i as f32;
                
                // Vertical line with subtle color
                if i > 0 {
                    layer.set_outline_color(Color::Rgb(Rgb::new(0.85, 0.85, 0.85, None)));
                    text_renderer.draw_line(
                        layer,
                        cell_x,
                        current_y,
                        cell_x,
                        current_y - row_height,
                        border_thickness * 0.7,
                    );
                    layer.set_outline_color(colors.table_border.clone());
                }
                
                // Cell text with proper vertical centering
                let wrapped_text = text_renderer.wrap_text(cell, cell_width - cell_padding * 2.0, text_renderer.get_font_size("body"));
                let line_height = text_renderer.calculate_line_height(text_renderer.get_font_size("body"));
                let total_text_height = line_height * wrapped_text.len() as f32;
                let vertical_offset = (row_height - total_text_height) / 2.0;
                let mut text_y = current_y - cell_padding - vertical_offset - line_height * 0.8;
                
                layer.set_fill_color(colors.text.clone());
                for line in wrapped_text {
                    layer.use_text(
                        &line,
                        text_renderer.get_font_size("body"),
                        cell_x + cell_padding,
                        text_y,
                        &font_system.regular,
                    );
                    text_y -= line_height;
                }
            }
            current_y -= row_height;
            row_index += 1;
            
            // Subtle horizontal line after each row
            layer.set_outline_color(Color::Rgb(Rgb::new(0.90, 0.90, 0.90, None)));
            text_renderer.draw_line(
                layer,
                x,
                current_y,
                x + max_width,
                current_y,
                border_thickness * 0.5,
            );
            layer.set_outline_color(colors.table_border.clone());
        }
        
        Ok(total_height)
    }

    fn process_formatting(&self, text: &str, _formatting: &[TextFormat]) -> String {
        // Process embedded formatting markers
        let mut result = text.to_string();
        
        // Handle links
        while let Some(start) = result.find("[LINK_START:") {
            if let Some(url_end) = result[start..].find(']') {
                let url_start = start + 12; // "[LINK_START:".len()
                let url = &result[url_start..start + url_end];
                if let Some(link_end) = result[start + url_end..].find("[LINK_END]") {
                    let text_start = start + url_end + 1;
                    let text_end = start + url_end + link_end;
                    let link_text = &result[text_start..text_end];
                    let replacement = format!("{} ({})", link_text, url);
                    result.replace_range(start..text_end + 10, &replacement); // 10 = "[LINK_END]".len()
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        result
    }
}