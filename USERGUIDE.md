# 📚 PaperCraft User Guide

**Complete guide to using PaperCraft for professional Markdown to PDF conversion**

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Installation](#installation)
3. [Basic Usage](#basic-usage)
4. [Configuration](#configuration)
5. [Themes](#themes)
6. [Advanced Features](#advanced-features)
7. [CLI Reference](#cli-reference)
8. [Troubleshooting](#troubleshooting)
9. [Best Practices](#best-practices)
10. [Examples & Tutorials](#examples--tutorials)

---

## Getting Started

### What is PaperCraft?

PaperCraft is a powerful command-line tool that converts Markdown documents to high-quality PDF files. It's designed for professionals who need beautiful, consistent document output with zero system dependencies - everything works out of the box.

### Key Benefits

- **Professional Output**: Publication-quality PDFs with precise typography
- **Zero Setup**: No Chrome installation required - downloads automatically
- **Server Ready**: Works on headless servers, Docker, CI/CD environments
- **Flexible Themes**: Built-in themes for different document types
- **Advanced Features**: Table of contents, syntax highlighting, math support
- **Batch Processing**: Convert multiple files or entire directories
- **Developer-Friendly**: Dry run mode, validation, and detailed logging

---

## Installation

### Download and Setup

1. **Download**: Get the latest release from the releases page
2. **Extract**: Unzip the executable to your preferred location
3. **PATH Setup**: (Optional) Add the executable to your system PATH
4. **Ready to Use**: Chrome Headless Shell downloads automatically on first conversion
5. **Verify**: Run `papercraft --help` to confirm installation

### System Requirements

- **Operating System**: Windows 10+, macOS 10.14+, or Linux
- **Memory**: 2GB RAM minimum, 4GB recommended for large documents
- **Storage**: 150MB total (100MB for app + 50MB for Chrome Headless Shell)
- **Internet**: Required only for initial Chrome download (~50MB)

### First-Time Setup

Run the interactive setup wizard:

```bash
papercraft --setup-wizard
```

This will guide you through:
- Theme selection
- Page settings (size, orientation, margins)
- Font preferences
- Advanced features configuration
- Creating a sample document

### Chrome Headless Shell

PaperCraft uses Chrome Headless Shell for high-quality PDF generation. **No manual installation required!**

- **First Use**: Chrome downloads automatically (~50MB, one-time)
- **Subsequent Uses**: Instant startup with bundled Chrome
- **Offline**: Works completely offline after initial download
- **Version**: Uses stable Chrome for Testing (v131.0.6778.108)
- **Location**: Stored in your system's app data directory

```bash
# Optional: Pre-download Chrome (not required)
papercraft --check-chrome
```

---

## Basic Usage

### Single File Conversion

Convert a single Markdown file to PDF:

```bash
papercraft -i document.md -o document.pdf
```

### Directory Conversion

Convert all Markdown files in a directory:

```bash
papercraft -i docs/ -o output/ --batch
```

### Using Themes

Apply a built-in theme:

```bash
papercraft -i document.md -o document.pdf --theme modern
```

### Quick Options

```bash
# Add table of contents
papercraft -i document.md -o document.pdf --toc

# Enable line numbers in code blocks
papercraft -i document.md -o document.pdf --line-numbers

# Optimize images
papercraft -i document.md -o document.pdf --optimize-images
```

---

## Configuration

### Configuration File

PaperCraft uses TOML configuration files for advanced customization. Generate a sample configuration:

```bash
papercraft --generate-config papercraft.toml
```

### Basic Configuration Structure

```toml
# papercraft.toml

[theme]
built_in = "modern"
css_file = "custom-theme.css"  # Optional custom CSS

[page]
size = "A4"                    # A4, Letter, Legal, A3, A5
orientation = "portrait"       # portrait, landscape
margins = "1in"               # CSS-style margins

[fonts]
family = "Inter"              # Font family
size = "11pt"                 # Font size

[toc]
enabled = true
title = "Table of Contents"
max_depth = 3

[code]
line_numbers = true
highlight_theme = "Solarized (dark)"
word_wrap = false

[images]
optimization = true
max_width = 800
max_height = 600

[page_numbers]
enabled = true
format = "Page {page} of {total}"
position = "footer"
```

### Theme Configuration

```toml
[theme]
built_in = "academic"         # Use built-in theme

# OR use custom CSS
css_file = "my-theme.css"

# OR define inline styles
custom_css = """
body { font-family: 'Georgia', serif; }
h1 { color: #2c3e50; }
"""
```

### Page Layout Configuration

```toml
[page]
size = "Letter"
orientation = "portrait"

[margins]
top = "1in"
right = "0.75in"
bottom = "1in"
left = "0.75in"

[header]
enabled = true
template = "<div style='text-align: center;'>Document Title</div>"
height = "0.5in"

[footer]
enabled = true
template = "<div style='text-align: center;'>Page {page}</div>"
height = "0.5in"
```

### Advanced Configuration Options

```toml
[advanced]
print_background = true       # Include background colors/images
wait_for_fonts = true        # Wait for web fonts to load
timeout = 30                 # PDF generation timeout (seconds)

[security]
allow_local_files = true     # Allow local file access
disable_javascript = false   # Disable JavaScript execution

[performance]
concurrent_jobs = 4          # Number of concurrent conversions
memory_limit = 2048         # Memory limit in MB
cache_enabled = true        # Enable caching for faster repeated runs
```

---

## Themes

### Built-in Themes

PaperCraft includes several professionally designed themes:

#### **Default Theme**
- Clean and versatile design
- Suitable for any document type
- Balanced typography and spacing

```bash
papercraft -i document.md -o output.pdf --theme default
```

#### **Academic Theme**
- Perfect for research papers and academic documents
- Professional serif fonts
- Proper citation formatting
- Clean, scholarly appearance

```bash
papercraft -i paper.md -o paper.pdf --theme academic
```

#### **Modern Theme**
- Contemporary design with vibrant accents
- Sans-serif fonts
- Modern color scheme
- Great for business documents

```bash
papercraft -i report.md -o report.pdf --theme modern
```

#### **Minimal Theme**
- Clean and distraction-free layout
- Maximum focus on content
- Subtle typography
- Perfect for documentation

```bash
papercraft -i docs.md -o docs.pdf --theme minimal
```

#### **Dark Theme**
- Dark background for reduced eye strain
- High contrast text
- Modern appearance
- Great for code-heavy documents

```bash
papercraft -i code-guide.md -o guide.pdf --theme dark
```

### Custom Themes

Create your own theme using CSS:

```css
/* custom-theme.css */

body {
    font-family: 'Source Sans Pro', sans-serif;
    color: #333;
    line-height: 1.6;
}

h1, h2, h3, h4, h5, h6 {
    color: #2c3e50;
    font-weight: 600;
}

h1 {
    border-bottom: 3px solid #3498db;
    padding-bottom: 0.5em;
}

code {
    background-color: #f8f9fa;
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-family: 'JetBrains Mono', monospace;
}

blockquote {
    border-left: 4px solid #3498db;
    padding-left: 1em;
    margin-left: 0;
    font-style: italic;
}

table {
    border-collapse: collapse;
    width: 100%;
}

th, td {
    border: 1px solid #ddd;
    padding: 8px;
    text-align: left;
}

th {
    background-color: #f2f2f2;
    font-weight: 600;
}
```

Use your custom theme:

```bash
papercraft -i document.md -o output.pdf --theme-file custom-theme.css
```

---

## Advanced Features

### Batch Processing

#### Basic Batch Processing

Convert all Markdown files in a directory:

```bash
papercraft -i docs/ -o output/ --batch
```

#### Concurrent Processing

Use multiple threads for faster processing:

```bash
papercraft -i docs/ -o output/ --batch --concurrent --jobs 4
```

#### Progress Tracking

Monitor progress with verbose output:

```bash
papercraft -i docs/ -o output/ --batch --verbose
```

### Directory Watching

Automatically regenerate PDFs when Markdown files change:

```bash
papercraft -i docs/ -o output/ --watch
```

This is perfect for:
- Live document editing
- Continuous integration
- Development workflows

### Dry Run Mode

Preview what will happen without actually converting files:

```bash
papercraft -i docs/ -o output/ --dry-run
```

Dry run shows:
- Files that will be processed
- Output locations
- Estimated processing time
- Potential issues or warnings

### Markdown Validation

Validate Markdown quality before conversion:

```bash
# Basic validation
papercraft -i document.md -o output.pdf --validate

# Detailed validation report
papercraft -i docs/ -o output/ --dry-run --validate --show-validation-details
```

Validation checks for:
- Broken links and missing images
- Malformed tables
- Empty headings
- Long lines and formatting issues
- Missing alt text for images

### Resume Capability

For long-running batch jobs, PaperCraft can resume interrupted conversions:

```bash
# List incomplete jobs
papercraft --list-jobs

# Resume a specific job
papercraft --resume job_1234567890

# Cancel a job
papercraft --cancel-job job_1234567890
```

### Memory Optimization

For large documents or constrained environments:

```bash
# Set memory limit
papercraft -i large-doc.md -o output.pdf --max-memory 1024

# Enable image optimization
papercraft -i doc.md -o output.pdf --optimize-images --max-image-width 800
```

### Logging Modes

Control output verbosity:

```bash
# Quiet mode (minimal output)
papercraft -i doc.md -o output.pdf --quiet

# Verbose mode (detailed information)
papercraft -i doc.md -o output.pdf --verbose

# Debug mode (maximum detail)
papercraft -i doc.md -o output.pdf --debug
```

---

## CLI Reference

### Input/Output Options

| Option | Description | Example |
|--------|-------------|---------|
| `-i, --input` | Input file or directory | `-i document.md` |
| `-o, --output` | Output file or directory | `-o output.pdf` |
| `--batch` | Enable batch processing mode | `--batch` |

### Theme Options

| Option | Description | Example |
|--------|-------------|---------|
| `--theme` | Built-in theme | `--theme modern` |
| `--theme-file` | Custom CSS file | `--theme-file custom.css` |

### Page Layout Options

| Option | Description | Example |
|--------|-------------|---------|
| `--paper-size` | Paper size | `--paper-size A4` |
| `--orientation` | Page orientation | `--orientation landscape` |
| `--margins` | Page margins | `--margins "1in"` |

### Typography Options

| Option | Description | Example |
|--------|-------------|---------|
| `--font-family` | Font family | `--font-family "Times New Roman"` |
| `--font-size` | Font size | `--font-size "12pt"` |

### Feature Options

| Option | Description | Example |
|--------|-------------|---------|
| `--toc` | Enable table of contents | `--toc` |
| `--no-toc` | Disable table of contents | `--no-toc` |
| `--line-numbers` | Enable code line numbers | `--line-numbers` |
| `--footnotes` | Enable footnotes | `--footnotes` |
| `--bibliography` | Enable bibliography | `--bibliography` |
| `--optimize-images` | Enable image optimization | `--optimize-images` |

### Advanced Options

| Option | Description | Example |
|--------|-------------|---------|
| `--concurrent` | Enable concurrent processing | `--concurrent` |
| `--jobs` | Number of concurrent jobs | `--jobs 4` |
| `--max-memory` | Memory limit in MB | `--max-memory 2048` |
| `--dry-run` | Preview without converting | `--dry-run` |
| `--validate` | Validate markdown | `--validate` |
| `--watch` | Watch for changes | `--watch` |

### Logging Options

| Option | Description | Example |
|--------|-------------|---------|
| `-q, --quiet` | Quiet mode | `--quiet` |
| `--verbose` | Verbose mode | `--verbose` |
| `--debug` | Debug mode | `--debug` |

### Configuration Options

| Option | Description | Example |
|--------|-------------|---------|
| `-c, --config` | Configuration file | `-c config.toml` |
| `--generate-config` | Generate sample config | `--generate-config` |
| `--setup-wizard` | Interactive setup | `--setup-wizard` |
| `--check-chrome` | Download/verify Chrome (optional) | `--check-chrome` |

### Job Management Options

| Option | Description | Example |
|--------|-------------|---------|
| `--list-jobs` | List incomplete jobs | `--list-jobs` |
| `--resume` | Resume job by ID | `--resume job_123` |
| `--cancel-job` | Cancel job by ID | `--cancel-job job_123` |

---

## Troubleshooting

### Common Issues and Solutions

#### Issue: First Run Takes Longer

**Problem**: Initial PDF conversion seems slow

**Solution**: This is normal - Chrome Headless Shell (~50MB) downloads automatically on first use. Subsequent conversions are fast.

#### Issue: Memory Errors with Large Files

**Problem**: Out of memory errors when processing large documents

**Solutions**:
1. Increase memory limit:
   ```bash
   papercraft -i large.md -o output.pdf --max-memory 4096
   ```
2. Enable image optimization:
   ```bash
   papercraft -i doc.md -o output.pdf --optimize-images
   ```
3. Process files individually instead of batch

#### Issue: Fonts Not Rendering

**Problem**: Custom fonts not appearing in PDF

**Solutions**:
1. Install fonts system-wide
2. Use web fonts in CSS:
   ```css
   @import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;600&display=swap');
   ```
3. Verify font names are correct

#### Issue: Images Missing or Broken

**Problem**: Images not appearing in PDF

**Solutions**:
1. Use absolute paths for images
2. Ensure image files exist and are accessible
3. Check image formats (PNG, JPG, SVG supported)
4. Validate markdown with `--validate` flag

#### Issue: Table of Contents Not Generated

**Problem**: TOC is empty or missing

**Solutions**:
1. Ensure document has proper headings (# ## ###)
2. Check TOC configuration:
   ```toml
   [toc]
   enabled = true
   max_depth = 3
   ```
3. Verify headings aren't in code blocks

#### Issue: Slow Processing

**Problem**: Conversion takes too long

**Solutions**:
1. Enable concurrent processing:
   ```bash
   papercraft --batch --concurrent --jobs 4
   ```
2. Optimize images:
   ```bash
   papercraft --optimize-images --max-image-width 800
   ```
3. Use SSD storage for faster I/O

### Debug Mode

For detailed troubleshooting, enable debug mode:

```bash
papercraft -i input.md -o output.pdf --debug --verbose
```

This provides:
- Detailed operation logs
- Error stack traces
- Performance metrics
- Resource usage information

### Getting Help

1. **Check Documentation**: Review this user guide and README
2. **Use Help Command**: `papercraft --help`
3. **Setup Wizard**: `papercraft --setup-wizard`
4. **Validate Input**: Use `--validate` to check markdown quality
5. **Test Configuration**: Use `--dry-run` to preview operations

---

## Best Practices

### Document Structure

#### Use Proper Heading Hierarchy

```markdown
# Main Title (H1)
## Section (H2)
### Subsection (H3)
#### Details (H4)
```

#### Add Meaningful Alt Text

```markdown
![GitHub Logo](https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png)
```

#### Use Consistent Formatting

```markdown
**Bold text** for emphasis
*Italic text* for subtle emphasis
`inline code` for technical terms
```

### Configuration Management

#### Version Control Configuration

Store your configuration files in version control:

```bash
# Add to .gitignore
*.pdf
.papercraft_state/

# But keep configuration
!papercraft.toml
!themes/
```

#### Environment-Specific Configs

Use different configurations for different environments:

```bash
# Development
papercraft -c config/dev.toml -i docs/ -o output/

# Production
papercraft -c config/prod.toml -i docs/ -o output/
```

### Performance Optimization

#### For Large Documents

1. **Enable image optimization**:
   ```bash
   papercraft --optimize-images --max-image-width 1000
   ```

2. **Use appropriate memory limits**:
   ```bash
   papercraft --max-memory 4096
   ```

3. **Process in smaller batches**:
   ```bash
   # Instead of entire directory
   papercraft -i chapter1/ -o output/ --batch
   papercraft -i chapter2/ -o output/ --batch
   ```

#### For Batch Processing

1. **Use concurrent processing**:
   ```bash
   papercraft --batch --concurrent --jobs $(nproc)
   ```

2. **Monitor with verbose output**:
   ```bash
   papercraft --batch --verbose
   ```

3. **Use resume capability for long jobs**:
   ```bash
   papercraft --batch && echo "Success" || papercraft --list-jobs
   ```

### Quality Assurance

#### Always Validate Before Production

```bash
# Full validation pipeline
papercraft -i docs/ -o output/ --dry-run --validate --show-validation-details
```

#### Test Different Themes

```bash
# Test multiple themes
for theme in default academic modern minimal; do
  papercraft -i doc.md -o "output-$theme.pdf" --theme $theme
done
```

#### Version Your Documents

Include version information in your documents:

```markdown
---
title: "Project Documentation"
version: "2.1.0"
date: "2024-12-16"
author: "Your Name"
---
```

---

## Examples & Tutorials

### Tutorial 1: Creating Your First Document

#### Step 1: Create a Markdown File

Create `my-document.md`:

```markdown
# My First PaperCraft Document

This is a sample document to demonstrate PaperCraft's capabilities.

## Features

PaperCraft supports:

- **Beautiful themes**
- Table of contents
- Code syntax highlighting
- Mathematical expressions: $E = mc^2$

## Code Example

```python
def hello_world():
    print("Hello, PaperCraft!")

hello_world()
```

## Conclusion

PaperCraft makes it easy to create professional PDF documents from Markdown.
```

#### Step 2: Convert to PDF

```bash
papercraft -i my-document.md -o my-document.pdf --theme modern --toc
```

#### Step 3: Review and Iterate

Open the PDF and adjust as needed. Try different themes:

```bash
papercraft -i my-document.md -o academic.pdf --theme academic
papercraft -i my-document.md -o minimal.pdf --theme minimal
```

### Tutorial 2: Setting Up a Documentation Workflow

#### Step 1: Create Directory Structure

```
docs/
├── README.md
├── user-guide.md
├── api-reference.md
├── images/
│   ├── logo.png
│   └── diagram.svg
└── config/
    └── papercraft.toml
```

#### Step 2: Create Configuration

`config/papercraft.toml`:

```toml
[theme]
built_in = "minimal"

[page]
size = "A4"
margins = "1in"

[toc]
enabled = true
title = "Table of Contents"

[code]
line_numbers = true
highlight_theme = "GitHub"

[images]
optimization = true
max_width = 800

[header]
enabled = true
template = "<div style='text-align: center; font-size: 10pt;'>Project Documentation</div>"

[footer]
enabled = true
template = "<div style='text-align: center; font-size: 10pt;'>Page {page} of {total}</div>"
```

#### Step 3: Set Up Batch Processing

```bash
# Convert all documentation
papercraft -c config/papercraft.toml -i docs/ -o output/ --batch --concurrent

# Watch for changes during development
papercraft -c config/papercraft.toml -i docs/ -o output/ --watch
```

#### Step 4: Add Quality Checks

```bash
# Validate before building
papercraft -c config/papercraft.toml -i docs/ -o output/ --dry-run --validate
```

### Tutorial 3: Academic Paper Workflow

#### Step 1: Paper Structure

`research-paper.md`:

```markdown
---
title: "Advanced Machine Learning Techniques"
author: "Dr. Jane Smith"
date: "December 2024"
abstract: "This paper explores advanced machine learning techniques..."
keywords: ["machine learning", "AI", "neural networks"]
---

# Advanced Machine Learning Techniques

## Abstract

This paper explores advanced machine learning techniques and their applications in modern data science.

## 1. Introduction

Machine learning has revolutionized how we process and analyze data...

### 1.1 Background

The field of machine learning emerged from...

## 2. Literature Review

Previous work in this area includes:

- Smith et al. (2023) demonstrated that...
- Johnson & Brown (2024) showed...

## 3. Methodology

Our approach consists of three main components:

1. Data preprocessing
2. Model training
3. Evaluation metrics

### 3.1 Data Preprocessing

We applied the following preprocessing steps:

```python
def preprocess_data(data):
    # Remove outliers
    clean_data = remove_outliers(data)
    
    # Normalize features
    normalized = normalize(clean_data)
    
    return normalized
```

## 4. Results

Our experiments yielded the following results:

| Model | Accuracy | Precision | Recall |
|-------|----------|-----------|--------|
| SVM   | 92.3%    | 91.1%     | 93.5%  |
| RF    | 94.7%    | 93.8%     | 95.1%  |
| NN    | 96.2%    | 95.4%     | 96.8%  |

### 4.1 Statistical Analysis

The mathematical model can be expressed as:

$$f(x) = \sum_{i=1}^{n} w_i x_i + b$$

Where $w_i$ represents the weight for feature $i$.

## 5. Discussion

The results indicate that neural networks outperform traditional methods...

## 6. Conclusion

In conclusion, our research demonstrates...

## References

1. Smith, A., et al. (2023). "Machine Learning Fundamentals." *Journal of AI Research*, 45(2), 123-145.
2. Johnson, B., & Brown, C. (2024). "Advanced Neural Networks." *ACM Computing Surveys*, 51(3), 1-28.
```

#### Step 2: Academic Configuration

`academic-config.toml`:

```toml
[theme]
built_in = "academic"

[page]
size = "A4"
orientation = "portrait"
margins = "1in"

[fonts]
family = "Times New Roman"
size = "12pt"

[toc]
enabled = true
title = "Table of Contents"
max_depth = 3

[code]
line_numbers = true
highlight_theme = "GitHub"

[footnotes]
enabled = true

[bibliography]
enabled = true

[page_numbers]
enabled = true
format = "{page}"
position = "footer"

[header]
enabled = true
template = "<div style='text-align: right; font-size: 10pt;'>Advanced ML Techniques - Smith 2024</div>"
```

#### Step 3: Generate Academic PDF

```bash
papercraft -c academic-config.toml -i research-paper.md -o research-paper.pdf --validate
```

### Tutorial 4: Business Report Automation

#### Step 1: Report Template

`quarterly-report.md`:

```markdown
# Q4 2024 Quarterly Business Report

**Prepared by**: Finance Team  
**Date**: December 31, 2024  
**Period**: October 1 - December 31, 2024

## Executive Summary

This quarter demonstrated strong performance across all key metrics...

## Financial Performance

### Revenue Analysis

Total revenue for Q4 2024: **$2.4M** (↑15% YoY)

#### Revenue Breakdown

| Product Line | Q4 2024 | Q3 2024 | Change |
|--------------|---------|---------|--------|
| Product A    | $1.2M   | $1.1M   | +9%    |
| Product B    | $800K   | $750K   | +7%    |
| Services     | $400K   | $350K   | +14%   |

### Cost Analysis

Operating expenses decreased by 3% compared to Q3...

## Market Analysis

### Competitive Landscape

Our market position strengthened this quarter...

### Customer Metrics

- New customers acquired: 450
- Customer retention rate: 94%
- Average customer value: $5,300

## Operational Highlights

### Key Achievements

1. **Product Launch**: Successfully launched Product C
2. **Team Growth**: Hired 12 new team members
3. **Process Improvement**: Reduced processing time by 25%

### Challenges and Mitigation

- **Supply Chain**: Implemented alternative suppliers
- **Market Competition**: Enhanced product differentiation
- **Staffing**: Improved retention programs

## Projections

### Q1 2025 Outlook

Based on current trends, we project:

- Revenue growth of 12-15%
- Continued market share expansion
- New product line introduction

## Appendix

### Methodology

Data collection and analysis methods...

### Detailed Financial Statements

[Detailed tables and charts would go here]
```

#### Step 2: Business Configuration

`business-config.toml`:

```toml
[theme]
built_in = "modern"

[page]
size = "Letter"
margins = "0.75in"

[fonts]
family = "Source Sans Pro"
size = "11pt"

[toc]
enabled = true
title = "Contents"

[code]
line_numbers = false

[images]
optimization = true
max_width = 1000

[header]
enabled = true
template = """
<div style='display: flex; justify-content: space-between; align-items: center; font-size: 10pt; color: #666;'>
  <div>Q4 2024 Quarterly Report</div>
  <div>Company Confidential</div>
</div>
"""

[footer]
enabled = true
template = """
<div style='display: flex; justify-content: space-between; align-items: center; font-size: 10pt; color: #666;'>
  <div>© 2024 Your Company</div>
  <div>Page {page} of {total}</div>
</div>
"""

[page_numbers]
enabled = false  # Using custom footer instead
```

#### Step 3: Automated Report Generation

Create a script for automated report generation:

```bash
#!/bin/bash
# generate-report.sh

REPORT_DATE=$(date +"%Y-%m-%d")
QUARTER="Q4-2024"

echo "Generating quarterly report for $QUARTER..."

# Validate the report
papercraft -c business-config.toml -i quarterly-report.md -o temp.pdf --dry-run --validate

if [ $? -eq 0 ]; then
    echo "Validation passed. Generating final report..."
    
    # Generate the final report
    papercraft -c business-config.toml -i quarterly-report.md -o "reports/$QUARTER-report-$REPORT_DATE.pdf"
    
    echo "Report generated successfully: reports/$QUARTER-report-$REPORT_DATE.pdf"
else
    echo "Validation failed. Please fix the issues before generating the report."
    exit 1
fi
```

### Tutorial 5: Custom Theme Development

#### Step 1: Analyze Existing Themes

First, understand how built-in themes work:

```bash
# Generate a document with different themes to see the differences
papercraft -i sample.md -o default.pdf --theme default
papercraft -i sample.md -o modern.pdf --theme modern
papercraft -i sample.md -o academic.pdf --theme academic
```

#### Step 2: Create Base Theme

Create `custom-theme.css`:

```css
/* Custom Corporate Theme */

/* Import fonts */
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap');
@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500&display=swap');

/* Base typography */
body {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    font-size: 11pt;
    line-height: 1.6;
    color: #2d3748;
    background-color: white;
    margin: 0;
    padding: 0;
}

/* Headings */
h1, h2, h3, h4, h5, h6 {
    font-weight: 600;
    margin-top: 2em;
    margin-bottom: 1em;
    line-height: 1.25;
}

h1 {
    font-size: 2.25em;
    color: #1a202c;
    border-bottom: 3px solid #3182ce;
    padding-bottom: 0.5em;
    margin-top: 0;
}

h2 {
    font-size: 1.75em;
    color: #2d3748;
    border-bottom: 1px solid #e2e8f0;
    padding-bottom: 0.3em;
}

h3 {
    font-size: 1.375em;
    color: #4a5568;
}

h4 {
    font-size: 1.125em;
    color: #4a5568;
}

/* Paragraphs and text */
p {
    margin-bottom: 1em;
    text-align: justify;
}

/* Links */
a {
    color: #3182ce;
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

/* Lists */
ul, ol {
    margin-bottom: 1em;
    padding-left: 2em;
}

li {
    margin-bottom: 0.5em;
}

/* Code styling */
code {
    font-family: 'JetBrains Mono', 'Courier New', monospace;
    background-color: #f7fafc;
    padding: 0.125em 0.25em;
    border-radius: 0.25em;
    font-size: 0.875em;
    border: 1px solid #e2e8f0;
}

pre {
    background-color: #f7fafc;
    border: 1px solid #e2e8f0;
    border-radius: 0.5em;
    padding: 1em;
    overflow-x: auto;
    margin-bottom: 1em;
    font-size: 0.875em;
}

pre code {
    background: none;
    border: none;
    padding: 0;
    font-size: inherit;
}

/* Tables */
table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 1em;
    font-size: 0.875em;
}

th, td {
    padding: 0.75em;
    text-align: left;
    border-bottom: 1px solid #e2e8f0;
}

th {
    font-weight: 600;
    background-color: #f7fafc;
    color: #2d3748;
    border-bottom: 2px solid #cbd5e0;
}

tr:nth-child(even) {
    background-color: #fafafa;
}

/* Blockquotes */
blockquote {
    border-left: 4px solid #3182ce;
    padding-left: 1em;
    margin: 1em 0;
    font-style: italic;
    color: #4a5568;
    background-color: #f7fafc;
    padding: 1em;
    border-radius: 0 0.5em 0.5em 0;
}

/* Images */
img {
    max-width: 100%;
    height: auto;
    display: block;
    margin: 1em auto;
    border-radius: 0.5em;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

/* Table of Contents */
.toc {
    background-color: #f7fafc;
    border: 1px solid #e2e8f0;
    border-radius: 0.5em;
    padding: 1.5em;
    margin-bottom: 2em;
}

.toc h2 {
    margin-top: 0;
    color: #2d3748;
    border-bottom: 1px solid #cbd5e0;
    padding-bottom: 0.5em;
}

.toc ul {
    list-style: none;
    padding-left: 0;
}

.toc li {
    margin-bottom: 0.25em;
}

.toc a {
    color: #4a5568;
    text-decoration: none;
    display: block;
    padding: 0.25em 0;
}

.toc a:hover {
    color: #3182ce;
}

/* Print-specific styles */
@media print {
    body {
        font-size: 10pt;
    }
    
    h1 {
        page-break-before: auto;
    }
    
    h1, h2, h3, h4, h5, h6 {
        page-break-after: avoid;
    }
    
    table, pre, blockquote {
        page-break-inside: avoid;
    }
    
    img {
        page-break-inside: avoid;
        max-height: 500px;
    }
}

/* Page layout */
@page {
    margin: 1in;
    
    @top-center {
        content: "Corporate Documentation";
        font-size: 9pt;
        color: #666;
    }
    
    @bottom-center {
        content: "Page " counter(page) " of " counter(pages);
        font-size: 9pt;
        color: #666;
    }
}
```

#### Step 3: Test Your Theme

```bash
papercraft -i test-document.md -o custom-output.pdf --theme-file custom-theme.css
```

#### Step 4: Refine and Iterate

Create a test document that covers all elements:

```markdown
# Theme Test Document

This document tests all styling elements.

## Typography

This is a paragraph with **bold text**, *italic text*, and `inline code`.

### Third Level Heading

Some content here.

#### Fourth Level Heading

More content.

## Lists

Unordered list:
- First item
- Second item
  - Nested item
  - Another nested item
- Third item

Ordered list:
1. First step
2. Second step
3. Third step

## Code

Inline code: `console.log("Hello, World!");`

Code block:
```javascript
function greet(name) {
    console.log(`Hello, ${name}!`);
}

greet("PaperCraft");
```

## Tables

| Feature | Status | Priority |
|---------|--------|----------|
| Themes | ✅ Complete | High |
| Batch Processing | ✅ Complete | High |
| Validation | ✅ Complete | Medium |

## Blockquotes

> This is a blockquote. It should be styled distinctly from regular paragraphs.
> 
> It can span multiple paragraphs.

## Links and Images

Visit [PaperCraft Documentation](https://example.com) for more information.

![Rust Logo](https://raw.githubusercontent.com/rust-lang/www.rust-lang.org/master/static/images/rust-logo-blk.svg)
```

#### Step 5: Create Theme Variants

Create variants for different use cases:

```css
/* dark-corporate.css - Dark variant */
body {
    background-color: #1a202c;
    color: #e2e8f0;
}

h1, h2, h3, h4, h5, h6 {
    color: #f7fafc;
}

/* print-optimized.css - Print-optimized variant */
body {
    font-size: 9pt;
    line-height: 1.4;
}

img {
    max-width: 50%;
    float: right;
    margin: 0 0 1em 1em;
}
```

---

This comprehensive user guide covers all aspects of using PaperCraft effectively. Whether you're a beginner just getting started or an advanced user looking to optimize your workflow, these guidelines and examples will help you create professional, high-quality PDF documents from your Markdown content.

For additional help, use the built-in help system (`papercraft --help`) or the interactive setup wizard (`papercraft --setup-wizard`).

**Happy documenting with PaperCraft! 🎨📚**