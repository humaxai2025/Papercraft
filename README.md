# 🎨 PaperCraft

**A professional Markdown to PDF and DOCX converter with beautiful themes and advanced configuration.**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

PaperCraft transforms your Markdown documents into stunning PDF files and professional DOCX documents with beautiful themes, advanced typography, and comprehensive customization options. Whether you're creating technical documentation, academic papers, or business reports, PaperCraft delivers publication-quality results in both formats.

## ✨ Features

### 🎯 **Core Functionality**
- **Multi-Format Output**: Convert to both PDF and DOCX formats with a single tool
- **High-Quality PDF Generation**: Professional output with precise typography
- **DOCX Document Creation**: Industry-standard Microsoft Word compatible documents
- **Zero Dependencies**: Automatically downloads Chrome Headless Shell - no manual setup required
- **Beautiful Built-in Themes**: Academic, minimal, modern, dark, and default styles
- **Custom Theme Support**: Create and use your own CSS themes (PDF only)
- **Advanced Typography**: Configurable fonts, sizes, and spacing
- **Table of Contents**: Automatic generation with customizable styling
- **Code Syntax Highlighting**: Over 100 programming languages supported
- **Mathematical Expressions**: LaTeX math rendering support (PDF only)
- **Image Optimization**: Automatic resizing and compression

### 🚀 **Performance & Reliability**
- **Server Ready**: Works on headless servers, Docker containers, and CI/CD environments
- **Concurrent Processing**: Multi-threaded batch operations
- **Memory Optimization**: Efficient handling of large documents
- **Progress Tracking**: Real-time progress indicators
- **Resume Capability**: Continue interrupted batch jobs
- **Error Recovery**: Detailed error reporting with suggestions
- **Validation**: Pre-conversion markdown quality checks

### 🛠️ **Developer Experience**
- **Dry Run Mode**: Preview changes without conversion
- **Verbose Logging**: Detailed operation insights
- **Configuration Wizard**: Interactive first-time setup
- **Batch Processing**: Convert entire directories
- **Directory Watching**: Auto-regeneration on file changes
- **Cross-Platform**: Windows, macOS, and Linux support
- **Format Selection**: Choose between PDF and DOCX output

## 🚀 Quick Start

### Installation

1. **Download the latest release** from the releases page
2. **Extract** the executable to your preferred location
3. **Add to PATH** (optional but recommended)
4. **That's it!** - Chrome Headless Shell downloads automatically on first use (PDF only)

### Basic Usage

```bash
# Convert to PDF (default)
papercraft -i document.md -o document.pdf

# Convert to DOCX
papercraft -i document.md -o document.docx --format docx

# Use a built-in theme (PDF only)
papercraft -i document.md -o document.pdf --theme modern

# Batch convert a directory to PDF
papercraft -i docs/ -o pdfs/ --batch

# Batch convert a directory to DOCX
papercraft -i docs/ -o docx-files/ --batch --format docx

# Interactive setup wizard
papercraft --setup-wizard
```

## 📖 Output Formats

### PDF Output
- **Chrome-based rendering** for pixel-perfect output
- **Custom CSS themes** support
- **Advanced typography** with web fonts
- **Mathematical expressions** with LaTeX support
- **Syntax highlighting** for code blocks
- **Print-optimized** layouts

### DOCX Output
- **Microsoft Word compatible** documents
- **Structured formatting** with proper headings, paragraphs, and lists
- **Text formatting** including bold, italic, strikethrough
- **Code blocks** with monospace fonts
- **Tables** converted to text representation
- **Cross-platform compatibility** with all major word processors

## 🎨 Themes (PDF Only)

PaperCraft includes several professionally designed themes for PDF output:

- **Default** - Clean and versatile for any document type
- **Academic** - Perfect for research papers and academic documents
- **Modern** - Contemporary design with vibrant accents
- **Minimal** - Clean and distraction-free layout
- **Dark** - Dark theme for reduced eye strain

*Note: DOCX output uses standard document formatting and doesn't support custom themes.*

## ⚙️ Configuration

### Quick Configuration

Generate a sample configuration file:

```bash
papercraft --generate-config papercraft.toml
```

### Basic Configuration Example

```toml
[output]
format = "pdf"  # or "docx"

[theme]
built_in = "modern"  # PDF only

[page]
size = "A4"
orientation = "portrait"
margins = "1in"

[fonts]
family = "Inter"
size = "11pt"

[toc]
enabled = true
title = "Table of Contents"

[code]
line_numbers = true
highlight_theme = "Solarized (dark)"  # PDF only
```

## 🔧 Advanced Features

### Multi-Format Batch Processing

```bash
# Process directory to PDF
papercraft -i docs/ -o pdf-output/ --batch --format pdf --concurrent

# Process directory to DOCX
papercraft -i docs/ -o docx-output/ --batch --format docx --concurrent

# Mixed processing with different themes
papercraft -i docs/ -o styled-pdfs/ --batch --theme academic --format pdf
```

### Format-Specific Options

```bash
# PDF with advanced styling
papercraft -i report.md -o report.pdf \
  --theme modern \
  --toc \
  --line-numbers \
  --optimize-images

# DOCX with custom page settings
papercraft -i report.md -o report.docx \
  --format docx \
  --paper-size A4 \
  --margins "1in" \
  --font-family "Times New Roman"
```

### Dry Run and Validation

```bash
# Preview what will happen without converting
papercraft -i docs/ -o output/ --dry-run --validate --format docx

# Show detailed validation results
papercraft -i docs/ -o output/ --dry-run --show-validation-details
```

### Resume Interrupted Jobs

```bash
# List incomplete jobs
papercraft --list-jobs

# Resume a specific job
papercraft --resume job_1234567890
```

### Directory Watching

```bash
# Auto-regenerate PDFs when markdown files change
papercraft -i docs/ -o output/ --watch

# Auto-regenerate DOCX files when markdown files change
papercraft -i docs/ -o output/ --watch --format docx
```

## 📋 Examples

### Technical Documentation (PDF)

```bash
papercraft -i api-docs/ -o documentation.pdf \
  --theme academic \
  --toc \
  --line-numbers \
  --optimize-images \
  --page-numbers
```

### Technical Documentation (DOCX)

```bash
papercraft -i api-docs/ -o documentation.docx \
  --format docx \
  --paper-size A4 \
  --margins "1in" \
  --font-family "Calibri"
```

### Academic Paper (PDF)

```bash
papercraft -i research-paper.md -o paper.pdf \
  --theme academic \
  --paper-size A4 \
  --margins "1in" \
  --font-family "Times New Roman" \
  --font-size "12pt" \
  --footnotes \
  --bibliography
```

### Academic Paper (DOCX)

```bash
papercraft -i research-paper.md -o paper.docx \
  --format docx \
  --paper-size A4 \
  --margins "1in" \
  --font-family "Times New Roman" \
  --font-size "12pt"
```

### Business Report

```bash
# PDF version
papercraft -i quarterly-report.md -o report.pdf \
  --theme modern \
  --header-template "<div>Quarterly Report Q4 2024</div>" \
  --footer-template "<div>Page {page} of {total}</div>" \
  --toc \
  --optimize-images

# DOCX version
papercraft -i quarterly-report.md -o report.docx \
  --format docx \
  --paper-size A4 \
  --margins "1in"
```

## 🛟 Troubleshooting

### Common Issues

**Issue**: First PDF conversion takes longer than expected  
**Solution**: Chrome Headless Shell (~50MB) downloads automatically on first use

**Issue**: Large files cause memory errors  
**Solution**: Use `--max-memory` flag or enable image optimization

**Issue**: Fonts not rendering correctly in PDF  
**Solution**: Ensure fonts are installed system-wide or use web fonts

**Issue**: DOCX formatting appears as plain text  
**Solution**: Ensure you're opening with Microsoft Word or compatible word processor

### Debug Mode

For detailed troubleshooting information:

```bash
papercraft -i input.md -o output.pdf --debug --verbose
papercraft -i input.md -o output.docx --format docx --debug --verbose
```

### Getting Help

- Check the [User Guide](USERGUIDE.md) for detailed documentation
- Use `papercraft --help` for command-line reference
- Run `papercraft --setup-wizard` for interactive configuration
- Enable `--verbose` mode for detailed operation logs

## 🔧 Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/your-org/papercraft.git
cd papercraft

# Build the project
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

### Dependencies

- **Rust 1.70+** - Modern Rust toolchain (for building from source only)
- **No runtime dependencies** - Chrome Headless Shell downloads automatically (PDF only)
- **System fonts** - Optional, for enhanced typography support

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📞 Support

- **Documentation**: [User Guide](USERGUIDE.md)
- **Issues**: [GitHub Issues](https://github.com/your-org/papercraft/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/papercraft/discussions)

## 🙏 Acknowledgments

- **Markdown Parsing**: [comrak](https://github.com/kivikakk/comrak)
- **PDF Generation**: [headless_chrome](https://github.com/atroche/rust-headless-chrome)
- **DOCX Generation**: [docx-rs](https://github.com/bokuweb/docx-rs)
- **Syntax Highlighting**: [syntect](https://github.com/trishume/syntect)
- **CLI Framework**: [clap](https://github.com/clap-rs/clap)

---

**Made with ❤️ by the PaperCraft team**

Transform your ideas into beautiful documents with PaperCraft! 🎨✨📄