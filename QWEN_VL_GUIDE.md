# Qwen-VL Integration Guide

## Overview

ISKIN now supports multimodal vision-language capabilities through Qwen-VL integration. This enables:

- **Screenshot Analysis**: Understand UI layouts from screenshots
- **OCR**: Extract text from images with bounding boxes
- **Visual Testing**: Compare UI states and detect regressions
- **Element Detection**: Find UI elements by description

## Installation

### 1. Install Qwen-VL Model

Download the model weights:

```bash
# Using huggingface-cli
pip install huggingface_hub
huggingface-cli download Qwen/Qwen-VL-Chat --local-dir /models/Qwen-VL-Chat

# Or manually download from https://huggingface.co/Qwen/Qwen-VL-Chat
```

### 2. Start Vision Server

```bash
# Run Qwen-VL server on port 8081
python -m vllm.entrypoints.api_server \
    --model /models/Qwen-VL-Chat \
    --port 8081 \
    --trust-remote-code
```

### 3. Configure ISKIN

Update `config/llm.toml`:

```toml
[vision]
enabled = true
model_path = "/models/Qwen-VL-Chat"
host = "localhost"
port = 8081
max_image_size = 1024
enable_ocr = true
```

## Usage

### Analyze Screenshot

```rust
use iskin::vision::{QwenVLEngine, QwenVLConfig};

let config = QwenVLConfig::default();
let engine = QwenVLEngine::new(config);

let analysis = engine.analyze_screenshot(
    "/path/to/screenshot.png",
    "Describe what you see in this UI"
).await?;

println!("Description: {}", analysis.description);
println!("Confidence: {}", analysis.confidence);
```

### Extract Text (OCR)

```rust
let text_regions = engine.extract_text("/path/to/image.png").await?;

for region in text_regions {
    println!("Text: '{}' at {:?}", region.text, region.bbox);
}
```

### Find UI Element

```rust
let element = engine.find_element(
    "/path/to/screenshot.png",
    "the submit button in the form"
).await?;

if let Some(elem) = element {
    println!("Found at: {:?}", elem.bbox);
    println!("Type: {}", elem.element_type);
}
```

### Browser Integration

```rust
use iskin::vision::browser_integration::*;

// Analyze browser page
let analysis = analyze_browser_page(&engine, "/tmp/page.png").await?;

// Visual regression test
let is_same = visual_regression_test(
    &engine,
    "/baseline/homepage.png",
    "/current/homepage.png"
).await?;

if !is_same {
    println!("UI has changed!");
}
```

## Tauri Commands

Vision functionality is exposed via Tauri commands:

### vision_analyze_screenshot

```typescript
const result = await invoke('vision_analyze_screenshot', {
    imagePath: '/path/to/screenshot.png',
    task: 'Describe the UI layout'
});
```

### vision_extract_text

```typescript
const textRegions = await invoke('vision_extract_text', {
    imagePath: '/path/to/document.png'
});
```

### vision_find_element

```typescript
const element = await invoke('vision_find_element', {
    imagePath: '/path/to/page.png',
    description: 'the login button'
});
```

## Use Cases

### 1. Automated UI Testing

```rust
// Capture screenshot
let screenshot = browser.screenshot().await?;

// Analyze with Qwen-VL
let analysis = engine.describe_ui(&screenshot).await?;

// Verify expected elements exist
assert!(analysis.ui_elements.iter()
    .any(|e| e.element_type == "button" && e.content == Some("Submit".to_string())));
```

### 2. Accessibility Audit

```rust
let analysis = engine.analyze_screenshot(
    "/page.png",
    "Identify accessibility issues: missing labels, low contrast, small touch targets"
).await?;

for action in analysis.suggested_actions {
    println!("Fix: {}", action);
}
```

### 3. Documentation Generation

```rust
let analysis = engine.analyze_screenshot(
    "/feature.png",
    "Describe this feature for user documentation"
).await?;

generate_docs(&analysis.description);
```

### 4. Code Review from Screenshots

```rust
let analysis = engine.analyze_screenshot(
    "/error_screen.png",
    "What error is shown? What might have caused it?"
).await?;

suggest_fixes(&analysis.description);
```

## Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `model_path` | `/models/Qwen-VL-Chat` | Path to model weights |
| `host` | `localhost` | Vision server host |
| `port` | `8081` | Vision server port |
| `max_image_size` | `1024` | Maximum image dimension |
| `enable_ocr` | `true` | Enable OCR preprocessing |

## Performance Tips

1. **Resize Images**: Keep images under 1024px for faster processing
2. **Batch Operations**: Process multiple regions in one request
3. **Cache Results**: Store analysis results for unchanged screenshots
4. **Async Processing**: Use async calls to avoid blocking

## Troubleshooting

### Model Not Found

```
Error: File not found: /models/Qwen-VL-Chat
```

**Solution**: Download model weights and update `model_path` in config.

### Connection Refused

```
Error: Network error: connection refused
```

**Solution**: Start the vision server before using ISKIN.

### Out of Memory

```
Error: CUDA out of memory
```

**Solution**: 
- Reduce `max_image_size`
- Use smaller batch sizes
- Run on GPU with more VRAM

## API Reference

### QwenVLEngine

Main engine for vision operations:
- `analyze_screenshot(path, task)` - General analysis
- `describe_ui(path)` - UI-specific analysis
- `extract_text(path)` - OCR extraction
- `find_element(path, description)` - Element detection

### VisionAnalysis

Result structure:
- `description: String` - Textual description
- `detected_text: Vec<TextRegion>` - OCR results
- `ui_elements: Vec<UIElement>` - Detected UI components
- `confidence: f32` - Confidence score
- `suggested_actions: Vec<String>` - Recommendations

### VisionError

Error types:
- `FileNotFound` - Image file missing
- `ImageError` - Processing failed
- `NetworkError` - Server communication failed
- `ModelError` - Model returned error
- `ParseError` - Response parsing failed

## Examples

See `src-tauri/src/vision/mod.rs` for implementation details and additional examples.

## Support

For issues and questions:
- GitHub Issues: https://github.com/rexo-null/AI-Ecosystem-Interface/issues
- Documentation: USER_GUIDE.md
