//! Qwen-VL Integration for Visual Analysis
//! 
//! Provides multimodal vision-language capabilities for screenshot analysis,
//! UI understanding, and visual code review.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for Qwen-VL model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenVLConfig {
    /// Path to Qwen-VL model weights
    pub model_path: String,
    /// Host for vision model server
    pub host: String,
    /// Port for vision model server
    pub port: u16,
    /// Maximum image size (pixels)
    pub max_image_size: u32,
    /// Enable OCR preprocessing
    pub enable_ocr: bool,
}

impl Default for QwenVLConfig {
    fn default() -> Self {
        Self {
            model_path: "/models/Qwen-VL-Chat".to_string(),
            host: "localhost".to_string(),
            port: 8081,
            max_image_size: 1024,
            enable_ocr: true,
        }
    }
}

/// Image input for vision model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInput {
    /// Path to image file or base64-encoded data
    pub source: ImageSource,
    /// Optional prompt for the image
    pub prompt: Option<String>,
}

/// Image source type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ImageSource {
    /// File path
    FilePath { path: String },
    /// Base64-encoded image data
    Base64 { data: String, format: ImageFormat },
    /// URL to remote image
    Url { url: String },
}

/// Image format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    PNG,
    JPEG,
    WEBP,
}

/// Vision analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionAnalysis {
    /// Textual description of the image
    pub description: String,
    /// Detected text (OCR results)
    pub detected_text: Vec<TextRegion>,
    /// Identified UI elements
    pub ui_elements: Vec<UIElement>,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Suggested actions based on analysis
    pub suggested_actions: Vec<String>,
}

/// Text region from OCR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRegion {
    /// Bounding box [x, y, width, height]
    pub bbox: [i32; 4],
    /// Recognized text
    pub text: String,
    /// Confidence score
    pub confidence: f32,
}

/// UI element detected in screenshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIElement {
    /// Element type (button, input, label, etc.)
    pub element_type: String,
    /// Bounding box [x, y, width, height]
    pub bbox: [i32; 4],
    /// Element text/content
    pub content: Option<String>,
    /// Interactive state
    pub interactive: bool,
}

/// Vision engine for Qwen-VL integration
pub struct QwenVLEngine {
    config: QwenVLConfig,
    client: reqwest::Client,
}

impl QwenVLEngine {
    /// Create new vision engine with config
    pub fn new(config: QwenVLConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
    
    /// Analyze a screenshot
    pub async fn analyze_screenshot(
        &self,
        image_path: &str,
        task: &str,
    ) -> Result<VisionAnalysis, VisionError> {
        // Validate image exists
        let path = PathBuf::from(image_path);
        if !path.exists() {
            return Err(VisionError::FileNotFound(image_path.to_string()));
        }
        
        // Resize if needed
        let image_data = self.prepare_image(&path)?;
        
        // Send to vision model
        let response = self.send_to_model(&image_data, task).await?;
        
        // Parse response
        let analysis = self.parse_response(response)?;
        
        Ok(analysis)
    }
    
    /// Analyze base64-encoded image
    pub async fn analyze_base64(
        &self,
        image_data: &str,
        task: &str,
    ) -> Result<VisionAnalysis, VisionError> {
        let response = self.send_to_model(image_data, task).await?;
        let analysis = self.parse_response(response)?;
        Ok(analysis)
    }
    
    /// Describe UI layout from screenshot
    pub async fn describe_ui(&self, screenshot_path: &str) -> Result<VisionAnalysis, VisionError> {
        self.analyze_screenshot(
            screenshot_path,
            "Describe the UI layout, identify all interactive elements, buttons, inputs, and their purposes."
        ).await
    }
    
    /// Extract text from image (OCR)
    pub async fn extract_text(&self, image_path: &str) -> Result<Vec<TextRegion>, VisionError> {
        let analysis = self.analyze_screenshot(
            image_path,
            "Extract all visible text from this image with precise bounding boxes."
        ).await?;
        
        Ok(analysis.detected_text)
    }
    
    /// Find UI element by description
    pub async fn find_element(
        &self,
        screenshot_path: &str,
        description: &str,
    ) -> Result<Option<UIElement>, VisionError> {
        let analysis = self.analyze_screenshot(
            screenshot_path,
            &format!("Find the UI element matching: {}", description)
        ).await?;
        
        Ok(analysis.ui_elements.into_iter().next())
    }
    
    /// Prepare image for model (resize, convert)
    fn prepare_image(&self, path: &PathBuf) -> Result<String, VisionError> {
        use image::GenericImageView;
        
        let img = image::open(path)
            .map_err(|e| VisionError::ImageError(e.to_string()))?;
        
        let max_size = self.config.max_image_size;
        let (width, height) = img.dimensions();
        
        // Resize if larger than max
        let resized = if width > max_size || height > max_size {
            img.thumbnail(max_size, max_size)
        } else {
            img
        };
        
        // Convert to base64
        let mut buffer = Vec::new();
        resized.write_to(
            &mut std::io::Cursor::new(&mut buffer),
            image::ImageOutputFormat::Png
        ).map_err(|e| VisionError::ImageError(e.to_string()))?;
        
        let base64_data = base64::encode(&buffer);
        Ok(base64_data)
    }
    
    /// Send image to vision model server
    async fn send_to_model(
        &self,
        image_data: &str,
        task: &str,
    ) -> Result<serde_json::Value, VisionError> {
        let url = format!("http://{}:{}/v1/chat/completions", self.config.host, self.config.port);
        
        let payload = serde_json::json!({
            "model": "qwen-vl",
            "messages": [{
                "role": "user",
                "content": [
                    {
                        "type": "image",
                        "image": format!("data:image/png;base64,{}", image_data)
                    },
                    {
                        "type": "text",
                        "text": task
                    }
                ]
            }]
        });
        
        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| VisionError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(VisionError::ModelError(
                format!("Model returned status {}", response.status())
            ));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| VisionError::ParseError(e.to_string()))?;
        
        Ok(json)
    }
    
    /// Parse model response into structured analysis
    fn parse_response(&self, response: serde_json::Value) -> Result<VisionAnalysis, VisionError> {
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("");
        
        // Parse structured output (expecting JSON in response)
        let analysis: VisionAnalysis = serde_json::from_str(content)
            .unwrap_or_else(|_| VisionAnalysis {
                description: content.to_string(),
                detected_text: vec![],
                ui_elements: vec![],
                confidence: 0.8,
                suggested_actions: vec![],
            });
        
        Ok(analysis)
    }
}

/// Vision engine errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisionError {
    /// Image file not found
    FileNotFound(String),
    /// Image processing error
    ImageError(String),
    /// Network error communicating with model
    NetworkError(String),
    /// Model returned error
    ModelError(String),
    /// Failed to parse response
    ParseError(String),
    /// Configuration error
    ConfigError(String),
}

impl std::fmt::Display for VisionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VisionError::FileNotFound(path) => write!(f, "File not found: {}", path),
            VisionError::ImageError(msg) => write!(f, "Image error: {}", msg),
            VisionError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            VisionError::ModelError(msg) => write!(f, "Model error: {}", msg),
            VisionError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            VisionError::ConfigError(msg) => write!(f, "Config error: {}", msg),
        }
    }
}

impl std::error::Error for VisionError {}

/// Integration with browser automation for visual testing
pub mod browser_integration {
    use super::*;
    
    /// Take screenshot and analyze with Qwen-VL
    pub async fn analyze_browser_page(
        engine: &QwenVLEngine,
        screenshot_path: &str,
    ) -> Result<VisionAnalysis, VisionError> {
        engine.describe_ui(screenshot_path).await
    }
    
    /// Visual regression test - compare screenshots
    pub async fn visual_regression_test(
        engine: &QwenVLEngine,
        baseline_path: &str,
        current_path: &str,
    ) -> Result<bool, VisionError> {
        let baseline = engine.describe_ui(baseline_path).await?;
        let current = engine.describe_ui(current_path).await?;
        
        // Compare UI element structure
        Ok(baseline.ui_elements.len() == current.ui_elements.len())
    }
}

/// Tauri commands for vision integration
#[cfg(feature = "tauri")]
pub mod tauri_commands {
    use super::*;
    use tauri::State;
    
    pub struct VisionState(tokio::sync::Mutex<Option<QwenVLEngine>>);
    
    #[tauri::command]
    pub async fn vision_analyze_screenshot(
        state: State<'_, VisionState>,
        image_path: String,
        task: String,
    ) -> Result<VisionAnalysis, String> {
        let engine = state.0.lock().await;
        let engine = engine.as_ref()
            .ok_or_else(|| "Vision engine not initialized".to_string())?;
        
        engine.analyze_screenshot(&image_path, &task)
            .await
            .map_err(|e| e.to_string())
    }
    
    #[tauri::command]
    pub async fn vision_extract_text(
        state: State<'_, VisionState>,
        image_path: String,
    ) -> Result<Vec<TextRegion>, String> {
        let engine = state.0.lock().await;
        let engine = engine.as_ref()
            .ok_or_else(|| "Vision engine not initialized".to_string())?;
        
        engine.extract_text(&image_path)
            .await
            .map_err(|e| e.to_string())
    }
    
    #[tauri::command]
    pub async fn vision_find_element(
        state: State<'_, VisionState>,
        image_path: String,
        description: String,
    ) -> Result<Option<UIElement>, String> {
        let engine = state.0.lock().await;
        let engine = engine.as_ref()
            .ok_or_else(|| "Vision engine not initialized".to_string())?;
        
        engine.find_element(&image_path, &description)
            .await
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_default() {
        let config = QwenVLConfig::default();
        assert_eq!(config.port, 8081);
        assert!(config.enable_ocr);
    }
    
    #[test]
    fn test_image_source_serialization() {
        let source = ImageSource::FilePath { path: "/test.png".to_string() };
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("FilePath"));
    }
    
    #[test]
    fn test_vision_error_display() {
        let error = VisionError::FileNotFound("/missing.png".to_string());
        assert!(error.to_string().contains("/missing.png"));
    }
}
