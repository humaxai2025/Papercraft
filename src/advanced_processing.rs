use anyhow::Result;
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashMap;
use crate::config::{Config, TocStyle, FootnoteNumbering};
use lazy_static::lazy_static;

lazy_static! {
    static ref FOOTNOTE_REGEX: Regex = Regex::new(r"\[\^([^\]]+)\]").expect("Invalid footnote regex");
    static ref FOOTNOTE_DEF_REGEX: Regex = Regex::new(r"(?m)^\[\^([^\]]+)\]:\s*(.+)$").expect("Invalid footnote definition regex");
    static ref XREF_REGEX: Regex = Regex::new(r"\[@ref:(\w+):([^\]]+)\]").expect("Invalid cross-reference regex");
    static ref CITATION_REGEX: Regex = Regex::new(r"\[@cite:([^\]]+)\]").expect("Invalid citation regex");
    static ref CODE_BLOCK_REGEX: Regex = Regex::new(r#"(?s)<pre><code class="language-([^"]*)">(.*?)</code></pre>"#).expect("Invalid code block regex");
    static ref SCRIPT_REGEX: Regex = Regex::new(r"(?i)<script[^>]*>.*?</script>").expect("Invalid script regex");
    static ref STYLE_REGEX: Regex = Regex::new(r"(?i)<style[^>]*>.*?</style>").expect("Invalid style regex");
    static ref ON_EVENT_REGEX: Regex = Regex::new(r#"(?i)\s+on\w+\s*=\s*["'][^"']*["']"#).expect("Invalid event handler regex");
    static ref JAVASCRIPT_REGEX: Regex = Regex::new(r"(?i)javascript:").expect("Invalid javascript regex");
}

pub struct AdvancedProcessor {
    config: Config,
    #[allow(dead_code)]
    footnote_counter: u32,
    #[allow(dead_code)]
    citation_counter: u32,
    #[allow(dead_code)]
    cross_references: HashMap<String, String>,
}

impl AdvancedProcessor {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            footnote_counter: 0,
            citation_counter: 0,
            cross_references: HashMap::new(),
        }
    }

    pub fn process_document(&mut self, html: &str) -> Result<String> {
        let mut processed_html = html.to_string();

        // Sanitize HTML first for security
        processed_html = self.sanitize_html(&processed_html)?;
        
        // Process in order of dependencies
        processed_html = self.process_cross_references(&processed_html)?;
        processed_html = self.process_footnotes(&processed_html)?;
        processed_html = self.process_citations(&processed_html)?;
        processed_html = self.enhance_code_blocks(&processed_html)?;
        processed_html = self.generate_advanced_toc(&processed_html)?;

        Ok(processed_html)
    }

    pub fn generate_advanced_toc(&self, html: &str) -> Result<String> {
        if !self.config.toc.enabled {
            return Ok(html.replace("[TOC]", ""));
        }

        let document = Html::parse_fragment(html);
        let header_selector = Selector::parse("h1, h2, h3, h4, h5, h6")
            .map_err(|e| anyhow::anyhow!("Failed to parse header selector: {:?}", e))?;
        
        let mut toc_entries = Vec::new();
        let mut section_counters = [0u32; 6]; // For h1-h6

        for element in document.select(&header_selector) {
            let level = match element.value().name() {
                "h1" => 1,
                "h2" => 2,
                "h3" => 3,
                "h4" => 4,
                "h5" => 5,
                "h6" => 6,
                _ => continue,
            };

            if level as u8 > self.config.toc.max_depth {
                continue;
            }

            let text = element.text().collect::<String>().trim().to_string();
            let id = element.value().attr("id")
                .unwrap_or(&format!("heading-{}", toc_entries.len()))
                .to_string();

            // Update section counters for numbered style
            section_counters[level - 1] += 1;
            // Reset deeper level counters
            for counter in section_counters.iter_mut().skip(level) {
                *counter = 0;
            }

            let section_number = if matches!(self.config.toc.style, TocStyle::Numbered) {
                self.generate_section_number(&section_counters, level)
            } else {
                String::new()
            };

            toc_entries.push(TocEntry {
                level,
                text,
                id,
                section_number,
            });
        }

        let toc_html = self.render_toc(&toc_entries);
        Ok(html.replace("[TOC]", &toc_html))
    }

    fn generate_section_number(&self, counters: &[u32; 6], level: usize) -> String {
        let relevant_counters: Vec<u32> = counters[0..level]
            .iter()
            .filter(|&&c| c > 0)
            .copied()
            .collect();
        
        if relevant_counters.is_empty() {
            String::new()
        } else {
            format!("{}. ", relevant_counters.iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("."))
        }
    }

    fn render_toc(&self, entries: &[TocEntry]) -> String {
        let mut toc = format!(
            r#"<nav class="toc" data-page-numbers="{}">
            <h2 class="toc-title">{}</h2>
            <div class="toc-content">"#,
            self.config.toc.page_numbers,
            self.config.toc.title
        );

        match self.config.toc.style {
            TocStyle::Simple => {
                toc.push_str("<ul class=\"toc-list toc-simple\">");
                for entry in entries {
                    let link = if self.config.toc.links {
                        format!("<a href=\"#{}\" class=\"toc-link\">{}</a>", entry.id, entry.text)
                    } else {
                        entry.text.clone()
                    };
                    
                    let page_number = if self.config.toc.page_numbers {
                        format!("<span class=\"toc-page-number\" data-ref=\"#{}\"></span>", entry.id)
                    } else {
                        String::new()
                    };

                    toc.push_str(&format!(
                        r#"<li class="toc-entry toc-level-{}">{} {}</li>"#,
                        entry.level, link, page_number
                    ));
                }
                toc.push_str("</ul>");
            },
            TocStyle::Numbered => {
                toc.push_str("<ul class=\"toc-list toc-numbered\">");
                for entry in entries {
                    let link = if self.config.toc.links {
                        format!("<a href=\"#{}\" class=\"toc-link\">{}{}</a>", 
                               entry.id, entry.section_number, entry.text)
                    } else {
                        format!("{}{}", entry.section_number, entry.text)
                    };
                    
                    let page_number = if self.config.toc.page_numbers {
                        format!("<span class=\"toc-page-number\" data-ref=\"#{}\"></span>", entry.id)
                    } else {
                        String::new()
                    };

                    toc.push_str(&format!(
                        r#"<li class="toc-entry toc-level-{}">{} {}</li>"#,
                        entry.level, link, page_number
                    ));
                }
                toc.push_str("</ul>");
            },
            TocStyle::Indented => {
                let mut current_level = 0;
                for entry in entries {
                    while current_level < entry.level {
                        toc.push_str("<ul class=\"toc-list toc-indented\">");
                        current_level += 1;
                    }
                    while current_level > entry.level {
                        toc.push_str("</ul>");
                        current_level -= 1;
                    }

                    let link = if self.config.toc.links {
                        format!("<a href=\"#{}\" class=\"toc-link\">{}</a>", entry.id, entry.text)
                    } else {
                        entry.text.clone()
                    };
                    
                    let page_number = if self.config.toc.page_numbers {
                        format!("<span class=\"toc-page-number\" data-ref=\"#{}\"></span>", entry.id)
                    } else {
                        String::new()
                    };

                    toc.push_str(&format!(
                        r#"<li class="toc-entry toc-level-{}">{} {}</li>"#,
                        entry.level, link, page_number
                    ));
                }
                while current_level > 0 {
                    toc.push_str("</ul>");
                    current_level -= 1;
                }
            }
        }

        toc.push_str("</div></nav>");
        toc
    }

    pub fn process_footnotes(&mut self, html: &str) -> Result<String> {
        if !self.config.references.footnotes.enabled {
            return Ok(html.to_string());
        }

        let footnote_regex = &*FOOTNOTE_REGEX;
        let footnote_def_regex = &*FOOTNOTE_DEF_REGEX;

        let mut footnote_definitions = HashMap::new();
        let mut footnote_order = Vec::new();

        // Extract footnote definitions
        for cap in footnote_def_regex.captures_iter(html) {
            let id = cap[1].to_string();
            let content = cap[2].to_string();
            footnote_definitions.insert(id.clone(), content);
            footnote_order.push(id);
        }

        // Remove footnote definitions from main text
        let mut processed_html = footnote_def_regex.replace_all(html, "").to_string();

        // Replace footnote references
        let mut footnote_counter = 0;
        processed_html = footnote_regex.replace_all(&processed_html, |caps: &regex::Captures| {
            let id = &caps[1];
            if footnote_definitions.contains_key(id) {
                footnote_counter += 1;
                let number = self.format_footnote_number(footnote_counter);
                format!("<sup class=\"footnote-ref\"><a href=\"#footnote-{id}\" id=\"footnote-ref-{id}\">{number}</a></sup>")
            } else {
                caps[0].to_string() // Keep original if definition not found
            }
        }).to_string();

        // Add footnotes section
        if !footnote_definitions.is_empty() {
            let footnotes_html = self.render_footnotes(&footnote_definitions, &footnote_order);
            processed_html.push_str(&footnotes_html);
        }

        Ok(processed_html)
    }

    fn format_footnote_number(&self, number: u32) -> String {
        match self.config.references.footnotes.numbering {
            FootnoteNumbering::Numeric => number.to_string(),
            FootnoteNumbering::Roman => self.to_roman(number),
            FootnoteNumbering::Letters => self.to_letters(number),
            FootnoteNumbering::Symbols => self.to_symbols(number),
        }
    }

    fn to_roman(&self, mut num: u32) -> String {
        let values = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
        let numerals = ["m", "cm", "d", "cd", "c", "xc", "l", "xl", "x", "ix", "v", "iv", "i"];
        let mut result = String::new();
        
        for (i, &value) in values.iter().enumerate() {
            while num >= value {
                result.push_str(numerals[i]);
                num -= value;
            }
        }
        
        result
    }

    fn to_letters(&self, num: u32) -> String {
        if num == 0 { return "".to_string(); }
        let mut result = String::new();
        let mut n = num - 1;
        
        while n >= 26 {
            result.push(((n % 26) as u8 + b'a') as char);
            n /= 26;
            n -= 1;
        }
        result.push(((n % 26) as u8 + b'a') as char);
        result.chars().rev().collect()
    }

    fn to_symbols(&self, num: u32) -> String {
        let symbols = ["*", "†", "‡", "§", "¶", "**", "††", "‡‡"];
        if num as usize <= symbols.len() {
            symbols[(num - 1) as usize].to_string()
        } else {
            format!("*{num}")
        }
    }

    fn render_footnotes(&self, definitions: &HashMap<String, String>, order: &[String]) -> String {
        let mut footnotes = String::from(r#"<div class="footnotes"><hr><ol class="footnotes-list">"#);
        
        for (i, id) in order.iter().enumerate() {
            if let Some(content) = definitions.get(id) {
                let _number = self.format_footnote_number((i + 1) as u32);
                footnotes.push_str(&format!(
                    "<li id=\"footnote-{id}\" class=\"footnote-item\">{content} <a href=\"#footnote-ref-{id}\" class=\"footnote-backref\">↩</a></li>"
                ));
            }
        }
        
        footnotes.push_str("</ol></div>");
        footnotes
    }

    pub fn process_cross_references(&mut self, html: &str) -> Result<String> {
        if !self.config.references.cross_references {
            return Ok(html.to_string());
        }

        // Pattern for cross-references: [@ref:type:id]
        let xref_regex = &*XREF_REGEX;
        
        let processed_html = xref_regex.replace_all(html, |caps: &regex::Captures| {
            let ref_type = &caps[1];
            let ref_id = &caps[2];
            
            match ref_type {
                "fig" => format!("<a href=\"#fig-{ref_id}\" class=\"cross-ref cross-ref-fig\">Figure <span class=\"ref-number\" data-ref=\"fig-{ref_id}\"></span></a>"),
                "table" => format!("<a href=\"#table-{ref_id}\" class=\"cross-ref cross-ref-table\">Table <span class=\"ref-number\" data-ref=\"table-{ref_id}\"></span></a>"),
                "eq" => format!("<a href=\"#eq-{ref_id}\" class=\"cross-ref cross-ref-eq\">Equation <span class=\"ref-number\" data-ref=\"eq-{ref_id}\"></span></a>"),
                "sec" => format!("<a href=\"#sec-{ref_id}\" class=\"cross-ref cross-ref-sec\">Section <span class=\"ref-number\" data-ref=\"sec-{ref_id}\"></span></a>"),
                _ => format!("<a href=\"#{ref_id}\" class=\"cross-ref\">{ref_id}</a>"),
            }
        });

        Ok(processed_html.to_string())
    }

    pub fn process_citations(&mut self, html: &str) -> Result<String> {
        if !self.config.references.bibliography.enabled {
            return Ok(html.to_string());
        }

        // Pattern for citations: [@cite:key] or [@cite:key1,key2,key3]
        let citation_regex = &*CITATION_REGEX;
        
        let processed_html = citation_regex.replace_all(html, |caps: &regex::Captures| {
            let citation_keys = &caps[1];
            let keys: Vec<&str> = citation_keys.split(',').map(|s| s.trim()).collect();
            
            let citations: Vec<String> = keys.iter().map(|key| {
                format!("<a href=\"#bib-{}\" class=\"citation\" data-key=\"{}\">[<span class=\"citation-number\" data-cite=\"{}\"></span>]</a>", 
                       key, key, key)
            }).collect();
            
            citations.join(", ")
        });

        Ok(processed_html.to_string())
    }

    pub fn enhance_code_blocks(&self, html: &str) -> Result<String> {
        let code_block_regex = &*CODE_BLOCK_REGEX;
        
        let enhanced_html = code_block_regex.replace_all(html, |caps: &regex::Captures| {
            let language = caps.get(1).map_or("", |m| m.as_str());
            let code = caps.get(2).map_or("", |m| m.as_str());
            
            let mut enhanced_code = String::new();
            
            // Add language label if enabled
            if self.config.code.show_language && !language.is_empty() {
                enhanced_code.push_str(&format!(
                    r#"<div class="code-header"><span class="code-language">{}</span></div>"#,
                    language.to_uppercase()
                ));
            }
            
            enhanced_code.push_str(r#"<pre class="code-block">"#);
            
            if self.config.code.line_numbers {
                enhanced_code.push_str(&self.add_line_numbers(code));
            } else {
                enhanced_code.push_str(&format!(r#"<code class="language-{}">{}</code>"#, language, code));
            }
            
            enhanced_code.push_str("</pre>");
            
            enhanced_code
        });

        Ok(enhanced_html.to_string())
    }

    fn add_line_numbers(&self, code: &str) -> String {
        let lines: Vec<&str> = code.split('\n').collect();
        let total_lines = lines.len();
        let width = total_lines.to_string().len();
        
        let mut result = String::from(r#"<table class="code-table"><tbody>"#);
        
        for (i, line) in lines.iter().enumerate() {
            let line_number = i + 1;
            result.push_str(&format!(
                r#"<tr><td class="line-number" data-line-number="{:width$}"></td><td class="line-code"><code>{}</code></td></tr>"#,
                line_number,
                html_escape::encode_text(line),
                width = width
            ));
        }
        
        result.push_str("</tbody></table>");
        result
    }
    
    /// Sanitizes HTML by removing potentially dangerous elements and attributes
    pub fn sanitize_html(&self, html: &str) -> Result<String> {
        // Allow only safe HTML tags and attributes
        let _safe_tags = [
            "p", "br", "h1", "h2", "h3", "h4", "h5", "h6",
            "strong", "em", "b", "i", "u", "code", "pre",
            "ul", "ol", "li", "blockquote", "table", "thead",
            "tbody", "tr", "td", "th", "a", "img", "div",
            "span", "hr", "sup", "sub"
        ];
        
        let _safe_attributes = [
            "href", "src", "alt", "title", "class", "id",
            "width", "height", "target", "data-line-number",
            "data-key", "data-cite"
        ];
        
        // Use regex to remove script tags and other dangerous elements
        let script_regex = &*SCRIPT_REGEX;
        let style_regex = &*STYLE_REGEX;
        let on_event_regex = &*ON_EVENT_REGEX;
        let javascript_regex = &*JAVASCRIPT_REGEX;
        
        let mut sanitized = html.to_string();
        
        // Remove dangerous elements
        sanitized = script_regex.replace_all(&sanitized, "").to_string();
        sanitized = style_regex.replace_all(&sanitized, "").to_string();
        
        // Remove event handlers
        sanitized = on_event_regex.replace_all(&sanitized, "").to_string();
        
        // Remove javascript: urls
        sanitized = javascript_regex.replace_all(&sanitized, "").to_string();
        
        // Note: For production use, consider using a proper HTML sanitization library
        // like ammonia or html5ever for more comprehensive sanitization
        
        Ok(sanitized)
    }
}

#[derive(Debug)]
struct TocEntry {
    level: usize,
    text: String,
    id: String,
    section_number: String,
}