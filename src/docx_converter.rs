use anyhow::{Context, Result};
use comrak::{nodes::{AstNode, NodeValue}, parse_document, Arena, ComrakOptions};
use docx_rs::*;
use std::fs;
use std::path::Path;
use crate::config::Config;
use crate::logger::Logger;

pub struct DocxConverter {
    config: Config,
}

impl DocxConverter {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn convert_file(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        Logger::verbose(format!("Converting {} to DOCX format", input_path.display()));

        // Read markdown content
        let markdown_content = fs::read_to_string(input_path)
            .with_context(|| format!("Failed to read input file: {}", input_path.display()))?;

        // Parse markdown to AST
        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.strikethrough = true;
        options.extension.tagfilter = false;
        options.extension.table = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options.extension.superscript = true;
        options.extension.header_ids = Some("".to_string());
        options.extension.footnotes = true;
        options.extension.description_lists = true;
        options.extension.front_matter_delimiter = Some("---".to_string());

        let root = parse_document(&arena, &markdown_content, &options);

        // Create DOCX document
        let mut docx = Docx::new();

        // Apply document settings based on config
        self.apply_document_settings(&mut docx)?;

        // Convert AST to DOCX by collecting paragraphs
        let paragraphs = self.collect_paragraphs(root)?;
        Logger::verbose(format!("Collected {} paragraphs for conversion", paragraphs.len()));
        
        // Add all paragraphs to the document
        for paragraph in paragraphs {
            docx = docx.add_paragraph(paragraph);
        }
        
        // Add a final empty paragraph for proper formatting
        docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_text("")));

        // Write to output file
        let mut file = std::fs::File::create(output_path)
            .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
        
        docx.build().pack(&mut file)
            .with_context(|| format!("Failed to write DOCX file: {}", output_path.display()))?;

        Logger::verbose(format!("Successfully converted to {}", output_path.display()));
        Ok(())
    }

    fn apply_document_settings(&self, docx: &mut Docx) -> Result<()> {
        // Apply page settings
        let page_size = match self.config.page.size.preset.as_deref() {
            Some("A4") => (8267, 11693), // A4 in twentieths of a point
            Some("Letter") => (7920, 10080), // Letter in twentieths of a point
            Some("Legal") => (7920, 12240), // Legal in twentieths of a point
            Some("A3") => (11693, 16535), // A3 in twentieths of a point
            Some("A5") => (4961, 7016), // A5 in twentieths of a point
            _ => (8267, 11693), // Default to A4
        };

        // Convert margins from CSS units to twentieths of a point
        let top_margin = self.parse_margin(&self.config.page.margins.top)?;
        let right_margin = self.parse_margin(&self.config.page.margins.right)?;
        let bottom_margin = self.parse_margin(&self.config.page.margins.bottom)?;
        let left_margin = self.parse_margin(&self.config.page.margins.left)?;

        // Apply page settings
        *docx = docx.clone()
            .page_size(page_size.0, page_size.1)
            .page_margin(PageMargin::new()
                .top(top_margin)
                .right(right_margin)
                .bottom(bottom_margin)
                .left(left_margin));

        // Apply font settings if specified
        if let Some(_font_family) = &self.config.fonts.family {
            // Note: Font settings will be applied to individual runs
            // as the current docx-rs version doesn't have DefaultFonts
        }

        Ok(())
    }

    fn parse_margin(&self, margin_str: &str) -> Result<i32> {
        // Parse margin string (e.g., "1in", "20mm", "2cm") to twentieths of a point
        let margin_str = margin_str.trim().to_lowercase();
        
        if margin_str.ends_with("in") {
            let value: f64 = margin_str[..margin_str.len()-2].parse()
                .context("Invalid margin value")?;
            Ok((value * 1440.0) as i32) // 1 inch = 1440 twentieths of a point
        } else if margin_str.ends_with("mm") {
            let value: f64 = margin_str[..margin_str.len()-2].parse()
                .context("Invalid margin value")?;
            Ok((value * 56.69) as i32) // 1 mm ≈ 56.69 twentieths of a point
        } else if margin_str.ends_with("cm") {
            let value: f64 = margin_str[..margin_str.len()-2].parse()
                .context("Invalid margin value")?;
            Ok((value * 566.93) as i32) // 1 cm ≈ 566.93 twentieths of a point
        } else if margin_str.ends_with("pt") {
            let value: f64 = margin_str[..margin_str.len()-2].parse()
                .context("Invalid margin value")?;
            Ok((value * 20.0) as i32) // 1 pt = 20 twentieths of a point
        } else {
            // Default: assume inches
            let value: f64 = margin_str.parse()
                .context("Invalid margin value")?;
            Ok((value * 1440.0) as i32)
        }
    }

    fn collect_paragraphs<'a>(&self, node: &'a AstNode<'a>) -> Result<Vec<Paragraph>> {
        let mut paragraphs = Vec::new();
        self.process_node_to_paragraphs(node, &mut paragraphs)?;
        Ok(paragraphs)
    }

    fn process_node_to_paragraphs<'a>(&self, node: &'a AstNode<'a>, paragraphs: &mut Vec<Paragraph>) -> Result<()> {
        match &node.data.borrow().value {
            NodeValue::Document => {
                // Process all children
                for child in node.children() {
                    self.process_node_to_paragraphs(child, paragraphs)?;
                }
            }
            NodeValue::Heading(heading) => {
                let level = heading.level;
                let text = self.extract_text_from_node(node)?;
                
                let paragraph = match level {
                    1 => Paragraph::new().add_run(Run::new().add_text(&text).size(32).bold()),
                    2 => Paragraph::new().add_run(Run::new().add_text(&text).size(28).bold()),
                    3 => Paragraph::new().add_run(Run::new().add_text(&text).size(24).bold()),
                    4 => Paragraph::new().add_run(Run::new().add_text(&text).size(22).bold()),
                    5 => Paragraph::new().add_run(Run::new().add_text(&text).size(20).bold()),
                    _ => Paragraph::new().add_run(Run::new().add_text(&text).size(18).bold()),
                };
                
                paragraphs.push(paragraph);
            }
            NodeValue::Paragraph => {
                let mut paragraph = Paragraph::new();
                for child in node.children() {
                    self.process_inline_node(&mut paragraph, child)?;
                }
                paragraphs.push(paragraph);
            }
            NodeValue::List(list_data) => {
                for (i, child) in node.children().enumerate() {
                    if let NodeValue::Item(_) = &child.data.borrow().value {
                        let text = self.extract_text_from_node(child)?;
                        let paragraph = if list_data.list_type == comrak::nodes::ListType::Ordered {
                            Paragraph::new()
                                .add_run(Run::new().add_text(&format!("{}. {}", i + 1, text)))
                                .indent(Some(720), None, None, None) // Indent list items
                        } else {
                            Paragraph::new()
                                .add_run(Run::new().add_text(&format!("• {}", text)))
                                .indent(Some(720), None, None, None) // Indent list items
                        };
                        paragraphs.push(paragraph);
                    }
                }
            }
            NodeValue::CodeBlock(code_block) => {
                let code_text = &code_block.literal;
                let _language = &code_block.info;
                
                let code_paragraph = Paragraph::new()
                    .add_run(
                        Run::new()
                            .add_text(code_text)
                            .fonts(RunFonts::new().ascii("Consolas").hi_ansi("Consolas"))
                            .size(20)
                    );
                    
                paragraphs.push(code_paragraph);
            }
            NodeValue::Table(_) => {
                // For now, convert table to simple text representation
                let table_text = self.extract_text_from_node(node)?;
                let table_paragraph = Paragraph::new()
                    .add_run(Run::new().add_text(&table_text));
                paragraphs.push(table_paragraph);
            }
            NodeValue::ThematicBreak => {
                // Add a horizontal rule as a simple line
                let hr_paragraph = Paragraph::new()
                    .add_run(Run::new().add_text("___________________________________"));
                paragraphs.push(hr_paragraph);
            }
            NodeValue::BlockQuote => {
                let quote_text = self.extract_text_from_node(node)?;
                let quote_paragraph = Paragraph::new()
                    .add_run(Run::new().add_text(&quote_text).italic())
                    .indent(Some(720), None, None, None); // Indent blockquotes
                paragraphs.push(quote_paragraph);
            }
            _ => {
                // Process children for other node types
                for child in node.children() {
                    self.process_node_to_paragraphs(child, paragraphs)?;
                }
            }
        }
        Ok(())
    }

    fn process_inline_node<'a>(&self, paragraph: &mut Paragraph, node: &'a AstNode<'a>) -> Result<()> {
        match &node.data.borrow().value {
            NodeValue::Text(text) => {
                *paragraph = paragraph.clone().add_run(Run::new().add_text(text));
            }
            NodeValue::Strong => {
                let text = self.extract_text_from_node(node)?;
                *paragraph = paragraph.clone().add_run(Run::new().add_text(&text).bold());
            }
            NodeValue::Emph => {
                let text = self.extract_text_from_node(node)?;
                *paragraph = paragraph.clone().add_run(Run::new().add_text(&text).italic());
            }
            NodeValue::Code(code) => {
                *paragraph = paragraph.clone().add_run(
                    Run::new()
                        .add_text(&code.literal)
                        .fonts(RunFonts::new().ascii("Consolas").hi_ansi("Consolas"))
                );
            }
            NodeValue::Link(link) => {
                let link_text = self.extract_text_from_node(node)?;
                // For now, just add as regular text with the URL in parentheses
                *paragraph = paragraph.clone().add_run(
                    Run::new().add_text(&format!("{} ({})", link_text, link.url))
                        .color("0000FF") // Blue color for links
                );
            }
            NodeValue::Strikethrough => {
                let text = self.extract_text_from_node(node)?;
                *paragraph = paragraph.clone().add_run(Run::new().add_text(&text).strike());
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                *paragraph = paragraph.clone().add_run(Run::new().add_break(BreakType::TextWrapping));
            }
            _ => {
                // Process children for other inline elements
                for child in node.children() {
                    self.process_inline_node(paragraph, child)?;
                }
            }
        }
        Ok(())
    }

    fn extract_text_from_node<'a>(&self, node: &'a AstNode<'a>) -> Result<String> {
        let mut text = String::new();
        self.collect_text_from_node(node, &mut text);
        Ok(text)
    }

    fn collect_text_from_node<'a>(&self, node: &'a AstNode<'a>, text: &mut String) {
        match &node.data.borrow().value {
            NodeValue::Text(node_text) => {
                text.push_str(node_text);
            }
            NodeValue::Code(code) => {
                text.push_str(&code.literal);
            }
            _ => {
                for child in node.children() {
                    self.collect_text_from_node(child, text);
                }
            }
        }
    }
}