use crate::config::{Config, TocStyle, FootnoteStyle};

pub struct AdvancedStyles;

impl AdvancedStyles {
    pub fn generate_toc_styles(config: &Config) -> String {
        let base_styles = r#"
/* Table of Contents Styles */
.toc {
    background-color: #f8f9fa;
    border: 1px solid #e9ecef;
    border-radius: 8px;
    padding: 1.5rem;
    margin: 2rem 0;
    page-break-inside: avoid;
}

.toc-title {
    margin-top: 0;
    margin-bottom: 1rem;
    color: #2c3e50;
    font-size: 1.25rem;
    font-weight: 600;
    text-align: center;
    border-bottom: 2px solid #3498db;
    padding-bottom: 0.5rem;
}

.toc-content {
    font-size: 0.95rem;
}

.toc-list {
    list-style: none;
    padding-left: 0;
    margin: 0;
}

.toc-entry {
    margin: 0.25rem 0;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.toc-link {
    color: #2c3e50;
    text-decoration: none;
    flex-grow: 1;
}

.toc-link:hover {
    color: #3498db;
    text-decoration: underline;
}

.toc-page-number {
    font-weight: 500;
    color: #6c757d;
    margin-left: 0.5rem;
}

.toc-page-number::before {
    content: "...";
    letter-spacing: 2px;
    margin-right: 0.25rem;
}
"#;

        let style_specific = match config.toc.style {
            TocStyle::Simple => r#"
.toc-simple .toc-entry {
    border-bottom: 1px dotted #dee2e6;
    padding-bottom: 0.25rem;
    margin-bottom: 0.25rem;
}

.toc-simple .toc-level-1 { font-weight: 600; }
.toc-simple .toc-level-2 { font-weight: 500; margin-left: 1rem; }
.toc-simple .toc-level-3 { font-weight: 400; margin-left: 2rem; }
.toc-simple .toc-level-4 { font-weight: 400; margin-left: 3rem; color: #6c757d; }
.toc-simple .toc-level-5 { font-weight: 400; margin-left: 4rem; color: #6c757d; }
.toc-simple .toc-level-6 { font-weight: 400; margin-left: 5rem; color: #6c757d; }
"#,
            TocStyle::Numbered => r#"
.toc-numbered .toc-entry {
    margin-bottom: 0.5rem;
}

.toc-numbered .toc-level-1 { 
    font-weight: 600; 
    font-size: 1.1em;
    margin-bottom: 0.75rem;
}
.toc-numbered .toc-level-2 { font-weight: 500; margin-left: 1rem; }
.toc-numbered .toc-level-3 { font-weight: 400; margin-left: 2rem; }
.toc-numbered .toc-level-4 { font-weight: 400; margin-left: 3rem; color: #6c757d; }
.toc-numbered .toc-level-5 { font-weight: 400; margin-left: 4rem; color: #6c757d; }
.toc-numbered .toc-level-6 { font-weight: 400; margin-left: 5rem; color: #6c757d; }
"#,
            TocStyle::Indented => r#"
.toc-indented {
    margin-left: 1.5rem;
}

.toc-indented > .toc-entry {
    position: relative;
}

.toc-indented > .toc-entry::before {
    content: "•";
    color: #3498db;
    margin-right: 0.5rem;
    position: absolute;
    left: -1rem;
}

.toc-level-1 { font-weight: 600; }
.toc-level-2 { font-weight: 500; }
.toc-level-3 { font-weight: 400; }
.toc-level-4 { font-weight: 400; color: #6c757d; }
.toc-level-5 { font-weight: 400; color: #6c757d; }
.toc-level-6 { font-weight: 400; color: #6c757d; }
"#,
        };

        format!("{}{}", base_styles, style_specific)
    }

    pub fn generate_footnote_styles(config: &Config) -> String {
        let base_styles = r#"
/* Footnote Styles */
.footnote-ref {
    font-size: 0.8em;
    line-height: 0;
}

.footnote-ref a {
    color: #3498db;
    text-decoration: none;
    font-weight: 500;
}

.footnote-ref a:hover {
    text-decoration: underline;
}

.footnotes {
    margin-top: 3rem;
    padding-top: 1rem;
    border-top: 2px solid #e9ecef;
    font-size: 0.9rem;
}

.footnotes hr {
    display: none; /* We use border-top instead */
}

.footnotes-list {
    padding-left: 1.5rem;
    margin: 1rem 0;
}

.footnote-item {
    margin-bottom: 0.5rem;
    line-height: 1.5;
}

.footnote-backref {
    color: #3498db;
    text-decoration: none;
    font-weight: 500;
    margin-left: 0.25rem;
}

.footnote-backref:hover {
    text-decoration: underline;
}
"#;

        let style_specific = match config.references.footnotes.style {
            FootnoteStyle::Bottom => "", // Default styles work for bottom
            FootnoteStyle::End => r#"
.footnotes {
    page-break-before: always;
    margin-top: 0;
}
"#,
            FootnoteStyle::Margin => r#"
.footnote-margin {
    position: absolute;
    right: 1rem;
    width: 200px;
    font-size: 0.8rem;
    background-color: #f8f9fa;
    padding: 0.5rem;
    border-left: 3px solid #3498db;
    margin-top: -1rem;
}
"#,
        };

        format!("{}{}", base_styles, style_specific)
    }

    pub fn generate_code_styles(config: &Config) -> String {
        let mut styles = r#"
/* Enhanced Code Block Styles */
.code-header {
    background-color: #2c3e50;
    color: white;
    padding: 0.5rem 1rem;
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border-radius: 6px 6px 0 0;
    margin-bottom: 0;
}

.code-block {
    background-color: #f8f9fa;
    border: 1px solid #e9ecef;
    border-radius: 6px;
    margin: 1rem 0;
    overflow-x: auto;
    font-family: 'JetBrains Mono', 'Fira Code', Consolas, 'Liberation Mono', Menlo, Courier, monospace;
}

.code-block code {
    background: none;
    border: none;
    padding: 0;
}
"#.to_string();

        if config.code.line_numbers {
            styles.push_str(r#"
.code-table {
    width: 100%;
    border-collapse: collapse;
    margin: 0;
    background-color: transparent;
}

.code-table tbody tr {
    background-color: transparent;
}

.code-table tbody tr:hover {
    background-color: #f1f3f4;
}

.line-number {
    background-color: #f1f3f4;
    color: #6c757d;
    padding: 0.25rem 0.75rem;
    text-align: right;
    border-right: 1px solid #dee2e6;
    user-select: none;
    font-size: 0.85em;
    width: 1%;
    white-space: nowrap;
}

.line-number::before {
    content: attr(data-line-number);
}

.line-code {
    padding: 0.25rem 0.75rem;
    white-space: pre;
    overflow-x: auto;
}

.line-code code {
    background: none;
    padding: 0;
    font-size: inherit;
}
"#);
        } else {
            styles.push_str(r#"
.code-block {
    padding: 1rem;
}
"#);
        }

        if config.code.word_wrap {
            styles.push_str(r#"
.code-block code {
    white-space: pre-wrap;
    word-wrap: break-word;
}
"#);
        }

        styles
    }

    pub fn generate_reference_styles() -> String {
        r#"
/* Cross-reference Styles */
.cross-ref {
    color: #3498db;
    text-decoration: none;
    font-weight: 500;
}

.cross-ref:hover {
    text-decoration: underline;
}

.cross-ref-fig { color: #e74c3c; }
.cross-ref-table { color: #27ae60; }
.cross-ref-eq { color: #9b59b6; }
.cross-ref-sec { color: #f39c12; }

/* Citation Styles */
.citation {
    color: #3498db;
    text-decoration: none;
    font-weight: 500;
}

.citation:hover {
    text-decoration: underline;
}

.citation-number {
    font-size: 0.9em;
}

/* Bibliography Styles */
.bibliography {
    margin-top: 3rem;
    page-break-before: auto;
}

.bibliography h2 {
    color: #2c3e50;
    border-bottom: 2px solid #3498db;
    padding-bottom: 0.5rem;
}

.bibliography-list {
    list-style: none;
    padding-left: 0;
}

.bibliography-item {
    margin-bottom: 1rem;
    padding-left: 2rem;
    text-indent: -2rem;
    line-height: 1.5;
}

.bibliography-item::before {
    content: "[" counter(bibliography-counter) "]";
    counter-increment: bibliography-counter;
    font-weight: 600;
    color: #3498db;
    margin-right: 0.5rem;
}

.bibliography {
    counter-reset: bibliography-counter;
}

/* Figure and Table Numbering */
.figure-number,
.table-number,
.equation-number {
    font-weight: 600;
    color: #2c3e50;
}

/* Enhanced Table Styles */
table {
    counter-increment: table-counter;
}

.table-caption::before {
    content: "Table " counter(table-counter) ": ";
    font-weight: 600;
    color: #2c3e50;
}

/* Enhanced Figure Styles */
.figure {
    counter-increment: figure-counter;
    margin: 2rem 0;
    text-align: center;
}

.figure-caption::before {
    content: "Figure " counter(figure-counter) ": ";
    font-weight: 600;
    color: #2c3e50;
}

/* Global counters */
body {
    counter-reset: figure-counter table-counter equation-counter;
}
"#.to_string()
    }

    pub fn generate_advanced_print_styles() -> String {
        r#"
/* Advanced Print Styles */
@media print {
    .toc {
        page-break-after: always;
    }
    
    .footnotes {
        page-break-before: avoid;
    }
    
    .figure,
    .code-block,
    .toc {
        page-break-inside: avoid;
    }
    
    .cross-ref::after {
        content: " (page " target-counter(attr(href), page) ")";
        font-size: 0.8em;
        color: #666;
    }
    
    .toc-page-number::after {
        content: target-counter(attr(data-ref), page);
    }
    
    .ref-number::after {
        content: target-counter(attr(data-ref), figure-counter);
    }
    
    .citation-number::after {
        content: target-counter(attr(data-cite), bibliography-counter);
    }
}
"#.to_string()
    }

    pub fn get_all_advanced_styles(config: &Config) -> String {
        let mut styles = String::new();
        
        if config.toc.enabled {
            styles.push_str(&Self::generate_toc_styles(config));
        }
        
        if config.references.footnotes.enabled {
            styles.push_str(&Self::generate_footnote_styles(config));
        }
        
        styles.push_str(&Self::generate_code_styles(config));
        styles.push_str(&Self::generate_reference_styles());
        styles.push_str(&Self::generate_advanced_print_styles());
        
        styles
    }
}