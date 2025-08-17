# 🎨 PaperCraft

**A professional Markdown to PDF converter with beautiful themes and advanced configuration.**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

PaperCraft transforms your Markdown documents into stunning PDF files with professional themes, advanced typography, and comprehensive customization options. Whether you're creating technical documentation, academic papers, or business reports, PaperCraft delivers publication-quality results.

## ✨ Features

### 🎯 **Core Functionality**
- **High-Quality PDF Generation**: Professional output with precise typography
- **Zero Dependencies**: Automatically downloads Chrome Headless Shell - no manual setup required
- **Beautiful Built-in Themes**: Academic, minimal, modern, dark, and default styles
- **Custom Theme Support**: Create and use your own CSS themes
- **Advanced Typography**: Configurable fonts, sizes, and spacing
- **Table of Contents**: Automatic generation with customizable styling
- **Code Syntax Highlighting**: Over 100 programming languages supported
- **Mathematical Expressions**: LaTeX math rendering support
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

## 🚀 Quick Start

### Installation

1. **Download the latest release** from the releases page
2. **Extract** the executable to your preferred location
3. **Add to PATH** (optional but recommended)
4. **That's it!** - Chrome Headless Shell downloads automatically on first use

### Basic Usage

```bash
# Convert a single file
papercraft -i document.md -o document.pdf

# Use a built-in theme
papercraft -i document.md -o document.pdf --theme modern

# Batch convert a directory
papercraft -i docs/ -o pdfs/ --batch

# Interactive setup wizard
papercraft --setup-wizard
```

## 📖 Documentation

- **[User Guide](USERGUIDE.md)** - Comprehensive usage guide
- **[Configuration Reference](docs/configuration.md)** - All configuration options
- **[Theme Development](docs/themes.md)** - Creating custom themes
- **[CLI Reference](docs/cli.md)** - Complete command-line interface

## 🎨 Themes

PaperCraft includes several professionally designed themes:

- **Default** - Clean and versatile for any document type
- **Academic** - Perfect for research papers and academic documents
- **Modern** - Contemporary design with vibrant accents
- **Minimal** - Clean and distraction-free layout
- **Dark** - Dark theme for reduced eye strain

## ⚙️ Configuration

### Quick Configuration

Generate a sample configuration file:

```bash
papercraft --generate-config papercraft.toml
```

### Basic Configuration Example

```toml
[theme]
built_in = "modern"

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
highlight_theme = "Solarized (dark)"
```

## 🔧 Advanced Features

### Batch Processing with Concurrency

```bash
# Process multiple files concurrently
papercraft -i docs/ -o output/ --batch --concurrent --jobs 4

# With progress tracking and validation
papercraft -i docs/ -o output/ --batch --verbose --validate
```

### Dry Run and Validation

```bash
# Preview what will happen without converting
papercraft -i docs/ -o output/ --dry-run --validate

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
```

## 📋 Examples

### Technical Documentation

```bash
papercraft -i api-docs/ -o documentation.pdf \
  --theme academic \
  --toc \
  --line-numbers \
  --optimize-images \
  --page-numbers
```

### Academic Paper

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

### Business Report

```bash
papercraft -i quarterly-report.md -o report.pdf \
  --theme modern \
  --header-template "<div>Quarterly Report Q4 2024</div>" \
  --footer-template "<div>Page {page} of {total}</div>" \
  --toc \
  --optimize-images
```

## 🛟 Troubleshooting

### Common Issues

**Issue**: First run takes longer than expected
**Solution**: Chrome Headless Shell (~50MB) downloads automatically on first use

**Issue**: Large files cause memory errors
**Solution**: Use `--max-memory` flag or enable image optimization

**Issue**: Fonts not rendering correctly
**Solution**: Ensure fonts are installed system-wide or use web fonts

### Debug Mode

For detailed troubleshooting information:

```bash
papercraft -i input.md -o output.pdf --debug --verbose
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
- **No runtime dependencies** - Chrome Headless Shell downloads automatically
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
- **Syntax Highlighting**: [syntect](https://github.com/trishume/syntect)
- **CLI Framework**: [clap](https://github.com/clap-rs/clap)

---

**Made with ❤️ by the PaperCraft team**

Transform your ideas into beautiful documents with PaperCraft! 🎨✨