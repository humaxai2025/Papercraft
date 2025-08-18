# 📚 PaperCraft User Guide

Welcome to the comprehensive user guide for PaperCraft! This guide will walk you through everything you need to know about converting Markdown documents to beautiful PDF and DOCX files.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Output Formats](#output-formats)
3. [Command Line Interface](#command-line-interface)
4. [Configuration](#configuration)
5. [Themes and Styling](#themes-and-styling)
6. [Batch Processing](#batch-processing)
7. [Advanced Features](#advanced-features)
8. [Troubleshooting](#troubleshooting)
9. [Examples](#examples)

## Getting Started

### Installation

PaperCraft is distributed as a single executable file. Simply download it and you're ready to go!

1. Download the latest release for your platform
2. Extract the executable to a directory of your choice
3. (Optional) Add the directory to your PATH for easy access

### First Conversion

Let's start with a simple example:

```bash
# Convert a Markdown file to PDF (default format)
papercraft -i my-document.md -o my-document.pdf

# Convert the same file to DOCX
papercraft -i my-document.md -o my-document.docx --format docx
```

That's it! PaperCraft will handle the rest.

## Output Formats

PaperCraft supports two output formats, each with its own strengths:

### PDF Output

PDF is the default format, perfect for:
- **Professional documents** requiring precise formatting
- **Print-ready materials** with exact page layouts
- **Documents with complex styling** using CSS themes
- **Academic papers** with citations and mathematical expressions
- **Technical documentation** with syntax-highlighted code

**Features:**
- Chrome-based rendering for pixel-perfect output
- Custom CSS themes and styling
- LaTeX math expression support
- Advanced typography with web fonts
- Print optimization

### DOCX Output

DOCX format is ideal for:
- **Collaborative documents** that need editing in Microsoft Word
- **Business documents** requiring further formatting
- **Templates** that will be modified by others
- **Cross-platform compatibility** with various word processors
- **Documents requiring tracked changes** and comments

**Features:**
- Microsoft Word compatible format
- Structured headings and paragraphs
- Text formatting (bold, italic, strikethrough)
- Code blocks with monospace fonts
- Lists and basic table support
- Cross-platform word processor compatibility

### Choosing the Right Format

| Use Case | Recommended Format |
|----------|-------------------|
| Final documents for distribution | PDF |
| Documents for further editing | DOCX |
| Print materials | PDF |
| Collaborative editing | DOCX |
| Complex styling/themes | PDF |
| Simple business documents | DOCX |
| Academic papers with math | PDF |
| Template documents | DOCX |

## Command Line Interface

### Basic Syntax

```bash
papercraft [OPTIONS] --input <FILE/DIR> --output <FILE/DIR>
```

### Essential Options

| Option | Description | Example |
|--------|-------------|---------|
| `-i, --input <PATH>` | Input file or directory | `-i document.md` |
| `-o, --output <PATH>` | Output file or directory | `-o document.pdf` |
| `--format <FORMAT>` | Output format (pdf, docx) | `--format docx` |
| `--batch` | Process entire directories | `--batch` |
| `--theme <THEME>` | Built-in theme (PDF only) | `--theme modern` |
| `-c, --config <FILE>` | Configuration file | `-c config.toml` |
| `--verbose` | Detailed output | `--verbose` |
| `--help` | Show help information | `--help` |

### Format-Specific Options

Some options apply to specific output formats:

**PDF-Only Options:**
- `--theme` - Built-in themes (modern, academic, minimal, dark)
- `--theme-file` - Custom CSS theme file
- `--toc` / `--no-toc` - Table of contents
- `--line-numbers` - Code line numbers
- `--optimize-images` - Image optimization

**Universal Options:**
- `--paper-size` - Page size (A4, Letter, Legal, etc.)
- `--orientation` - Page orientation (portrait, landscape)
- `--margins` - Page margins
- `--font-family` - Font family
- `--font-size` - Font size

## Configuration

### Configuration File

PaperCraft supports TOML, YAML, and JSON configuration files. Generate a sample configuration:

```bash
papercraft --generate-config papercraft.toml
```

### Sample Configuration

```toml
[output]
format = "pdf"  # Default format: "pdf" or "docx"
quality = 1.0
compression = false

[page]
size = "A4"
orientation = "portrait"

[page.margins]
top = "1in"
right = "1in"
bottom = "1in"
left = "1in"

[fonts]
family = "Inter"
size = "11pt"

[theme]
built_in = "default"  # PDF only: "default", "academic", "modern", "minimal", "dark"
# css_file = "custom-theme.css"  # PDF only: path to custom CSS

[toc]
enabled = true
title = "Table of Contents"
max_depth = 3

[code]
line_numbers = true  # PDF only
highlight_theme = "Solarized (dark)"  # PDF only

[images]
optimization = true
max_width = 800
max_height = 600

[references]
footnotes.enabled = true
bibliography.enabled = true
```

### Environment Variables

You can also use environment variables:

```bash
export PAPERCRAFT_FORMAT=docx
export PAPERCRAFT_THEME=modern
export PAPERCRAFT_VERBOSE=true
```

## Themes and Styling

### Built-in Themes (PDF Only)

PaperCraft includes several professional themes for PDF output:

#### Default Theme
Clean and versatile design suitable for any document type.

```bash
papercraft -i doc.md -o doc.pdf --theme default
```

#### Academic Theme
Perfect for research papers, theses, and academic documents.

```bash
papercraft -i paper.md -o paper.pdf --theme academic
```

#### Modern Theme
Contemporary design with vibrant accents and modern typography.

```bash
papercraft -i report.md -o report.pdf --theme modern
```

#### Minimal Theme
Clean, distraction-free layout focusing on content.

```bash
papercraft -i article.md -o article.pdf --theme minimal
```

#### Dark Theme
Dark background theme for reduced eye strain.

```bash
papercraft -i doc.md -o doc.pdf --theme dark
```

### Custom Themes (PDF Only)

Create your own CSS theme file:

```css
/* custom-theme.css */
body {
    font-family: 'Georgia', serif;
    line-height: 1.6;
    color: #333;
}

h1 {
    color: #2c5aa0;
    border-bottom: 2px solid #2c5aa0;
}

code {
    background-color: #f5f5f5;
    padding: 2px 4px;
    border-radius: 3px;
}
```

Apply your custom theme:

```bash
papercraft -i doc.md -o doc.pdf --theme-file custom-theme.css
```

### DOCX Styling

DOCX output uses structured formatting:
- **Headings** are properly structured (H1, H2, H3, etc.)
- **Text formatting** includes bold, italic, strikethrough
- **Code blocks** use monospace fonts
- **Lists** are properly indented
- **Page settings** respect margins and paper size from configuration

## Batch Processing

### Basic Batch Processing

Convert entire directories:

```bash
# Convert all .md files in docs/ to PDF
papercraft -i docs/ -o pdf-output/ --batch

# Convert all .md files in docs/ to DOCX
papercraft -i docs/ -o docx-output/ --batch --format docx
```

### Concurrent Processing

Speed up batch operations with concurrent processing:

```bash
# Use 4 concurrent threads
papercraft -i docs/ -o output/ --batch --concurrent --jobs 4

# Use all available CPU cores
papercraft -i docs/ -o output/ --batch --concurrent
```

### Directory Structure

PaperCraft preserves your directory structure:

```
docs/
├── chapter1/
│   ├── intro.md
│   └── overview.md
├── chapter2/
│   └── details.md
└── conclusion.md
```

Becomes:

```
output/
├── chapter1/
│   ├── intro.pdf (or .docx)
│   └── overview.pdf (or .docx)
├── chapter2/
│   └── details.pdf (or .docx)
└── conclusion.pdf (or .docx)
```

### Progress Tracking

Monitor batch processing progress:

```bash
papercraft -i large-docs/ -o output/ --batch --verbose
```

Output:
```
🔄 Starting batch processing: job_1234567890 (25 files)
📄 Processing: chapter1/intro.md → intro.pdf
  ✓ Completed: intro.pdf
📄 Processing: chapter1/overview.md → overview.pdf
  ✓ Completed: overview.pdf
...
🎉 Batch processing complete!
  ✓ Successfully processed: 23 files
  ✗ Failed: 2 files
```

## Advanced Features

### Directory Watching

Automatically regenerate documents when Markdown files change:

```bash
# Watch for changes and regenerate PDFs
papercraft -i docs/ -o output/ --watch

# Watch for changes and regenerate DOCX files
papercraft -i docs/ -o output/ --watch --format docx
```

This is perfect for:
- **Live preview** during document writing
- **Continuous integration** setups
- **Documentation websites** with auto-updating PDFs/DOCX

### Resume Capability

For large batch jobs, PaperCraft can resume interrupted processing:

```bash
# List incomplete jobs
papercraft --list-jobs

# Resume a specific job
papercraft --resume job_1234567890

# Cancel a running job
papercraft --cancel-job job_1234567890
```

### Dry Run Mode

Preview what will happen without actually converting files:

```bash
# See what files would be processed
papercraft -i docs/ -o output/ --dry-run

# Include validation checks
papercraft -i docs/ -o output/ --dry-run --validate

# Show detailed validation results
papercraft -i docs/ -o output/ --dry-run --show-validation-details
```

### Validation

Check your Markdown files for potential issues:

```bash
# Validate before conversion
papercraft -i docs/ -o output/ --validate

# Skip validation
papercraft -i docs/ -o output/ --no-validate
```

Common validation checks:
- Broken internal links
- Missing images
- Malformed tables
- Invalid frontmatter
- Encoding issues

### Memory Management

For large documents or batch operations:

```bash
# Limit memory usage to 512MB
papercraft -i huge-docs/ -o output/ --batch --max-memory 512

# Enable image optimization to reduce memory usage
papercraft -i docs/ -o output/ --batch --optimize-images
```

## Troubleshooting

### Common Issues and Solutions

#### PDF-Specific Issues

**Issue**: First conversion takes a long time  
**Solution**: Chrome Headless Shell (~50MB) downloads automatically on first use. Subsequent conversions will be much faster.

**Issue**: Fonts not displaying correctly  
**Solution**: 
- Ensure fonts are installed system-wide
- Use web fonts in your CSS
- Check font names in configuration

**Issue**: Math expressions not rendering  
**Solution**: Ensure your Markdown uses proper LaTeX syntax: `$inline math$` or `$$display math$$`

#### DOCX-Specific Issues

**Issue**: Document appears as plain text  
**Solution**: 
- Ensure you're opening with Microsoft Word or compatible software
- Try different word processors (LibreOffice, Google Docs)
- Check file association settings

**Issue**: Formatting not preserved  
**Solution**: 
- DOCX format has different capabilities than PDF
- Complex styling may not translate perfectly
- Consider using PDF for documents requiring exact formatting

#### General Issues

**Issue**: Large files cause memory errors  
**Solution**:
```bash
papercraft -i large-file.md -o output.pdf --max-memory 2048 --optimize-images
```

**Issue**: Batch processing fails on some files  
**Solution**:
```bash
papercraft -i docs/ -o output/ --batch --verbose --validate
```

**Issue**: Images not loading  
**Solution**: 
- Check image paths are relative to the Markdown file
- Ensure image files exist
- Use supported formats (PNG, JPEG, GIF, etc.)

### Debug Mode

Enable detailed logging for troubleshooting:

```bash
papercraft -i input.md -o output.pdf --debug --verbose
```

This provides:
- Detailed processing steps
- Error messages with context
- Performance timing information
- Memory usage statistics
- File processing details

### Getting Help

1. **Check this guide** for common solutions
2. **Run with `--verbose`** for detailed output
3. **Use `--dry-run`** to preview operations
4. **Enable `--debug`** for technical details
5. **Check GitHub issues** for known problems

## Examples

### Document Types

#### Technical Documentation

**PDF Version:**
```bash
papercraft -i api-docs/ -o documentation.pdf \
  --theme academic \
  --toc \
  --line-numbers \
  --optimize-images \
  --page-numbers \
  --batch
```

**DOCX Version:**
```bash
papercraft -i api-docs/ -o documentation.docx \
  --format docx \
  --paper-size A4 \
  --margins "1in" \
  --font-family "Calibri" \
  --batch
```

#### Academic Paper

**PDF Version:**
```bash
papercraft -i research-paper.md -o paper.pdf \
  --theme academic \
  --paper-size A4 \
  --margins "1in" \
  --font-family "Times New Roman" \
  --font-size "12pt" \
  --footnotes \
  --bibliography \
  --toc
```

**DOCX Version:**
```bash
papercraft -i research-paper.md -o paper.docx \
  --format docx \
  --paper-size A4 \
  --margins "1in" \
  --font-family "Times New Roman" \
  --font-size "12pt"
```

#### Business Report

**PDF Version:**
```bash
papercraft -i quarterly-report.md -o report.pdf \
  --theme modern \
  --header-template "<div>Quarterly Report Q4 2024</div>" \
  --footer-template "<div>Page {page} of {total}</div>" \
  --toc \
  --optimize-images
```

**DOCX Version:**
```bash
papercraft -i quarterly-report.md -o report.docx \
  --format docx \
  --paper-size Letter \
  --margins "1in" \
  --font-family "Arial"
```

#### Personal Blog

**PDF Version:**
```bash
papercraft -i blog-posts/ -o blog-pdf/ \
  --theme minimal \
  --batch \
  --concurrent \
  --optimize-images
```

**DOCX Version:**
```bash
papercraft -i blog-posts/ -o blog-docx/ \
  --format docx \
  --batch \
  --concurrent \
  --paper-size A4
```

### Workflow Examples

#### Documentation Website

```bash
# Generate both PDF and DOCX versions
papercraft -i docs/ -o dist/pdf/ --batch --theme modern --toc
papercraft -i docs/ -o dist/docx/ --batch --format docx

# Watch for changes during development
papercraft -i docs/ -o dist/pdf/ --watch --theme modern
```

#### Academic Workflow

```bash
# Draft in DOCX for collaboration
papercraft -i thesis.md -o thesis-draft.docx --format docx

# Final version in PDF for submission
papercraft -i thesis.md -o thesis-final.pdf --theme academic --toc --bibliography
```

#### Corporate Documentation

```bash
# Generate employee handbook in both formats
papercraft -i handbook/ -o dist/ --batch --concurrent

# PDF for official distribution
papercraft -i handbook/ -o official-handbook.pdf --theme modern --toc

# DOCX for departmental customization
papercraft -i handbook/ -o editable/ --batch --format docx
```

## Tips and Best Practices

### Markdown Best Practices

1. **Use proper heading hierarchy** (H1 → H2 → H3)
2. **Include alt text for images** for accessibility
3. **Use relative paths** for images and links
4. **Validate your Markdown** before conversion
5. **Test with both formats** to ensure compatibility

### Performance Optimization

1. **Use `--concurrent`** for batch operations
2. **Enable `--optimize-images`** for large images
3. **Set appropriate `--max-memory`** limits
4. **Use `--dry-run`** to preview large operations

### Format Selection Guidelines

**Choose PDF when:**
- You need pixel-perfect formatting
- The document is final/read-only
- You require custom styling/themes
- You need mathematical expressions
- Print quality is important

**Choose DOCX when:**
- The document needs further editing
- You're collaborating with others
- You need template documents
- Cross-platform editing is required
- Simple formatting is sufficient

---

## Conclusion

PaperCraft provides a powerful and flexible solution for converting Markdown to both PDF and DOCX formats. Whether you're creating technical documentation, academic papers, or business reports, PaperCraft has the features and flexibility to meet your needs.

For more help:
- Use `papercraft --help` for quick reference
- Run `papercraft --setup-wizard` for interactive setup
- Check the GitHub repository for updates and community support

Happy document creation! 🎨📄✨