/// YOLOv8 Object Detection Pipeline
///
/// Implements Pipeline trait for YOLO models (v8, v9, v10, v11)
/// 
/// Reference: https://github.com/pykeio/ort/blob/main/examples/yolov8/yolov8.rs

use crate::base::Pipeline;
use crate::error::Result;
use crate::types::PipelineType;
use serde_json::Value;
use tabagent_onnx_loader::OnnxSession;

/// YOLO object detection pipeline
pub struct YoloPipeline {
    session: Option<OnnxSession>,
    confidence_threshold: f32,
    iou_threshold: f32,
}

impl YoloPipeline {
    pub fn new() -> Self {
        Self {
            session: None,
            confidence_threshold: 0.5,
            iou_threshold: 0.45,
        }
    }
    
    pub fn with_thresholds(confidence: f32, iou: f32) -> Self {
        Self {
            session: None,
            confidence_threshold: confidence,
            iou_threshold: iou,
        }
    }
}

impl Pipeline for YoloPipeline {
    fn pipeline_type(&self) -> PipelineType {
        PipelineType::ObjectDetection
    }
    
    fn is_loaded(&self) -> bool {
        self.session.is_some()
    }
    
    fn load(&mut self, model_id: &str, options: Option<Value>) -> Result<()> {
        log::info!("Loading YOLO model: {}", model_id);
        
        // Extract options
        let opts = options.unwrap_or(Value::Null);
        let model_path = opts["model_path"]
            .as_str()
            .unwrap_or(model_id);
        
        // Load ONNX session (no tokenizer needed for vision)
        let session = OnnxSession::load(model_path)
            .map_err(|e| crate::error::PipelineError::BackendError(e.to_string()))?;
        
        self.session = Some(session);
        
        // Extract optional thresholds
        if let Some(conf) = opts["confidence_threshold"].as_f64() {
            self.confidence_threshold = conf as f32;
        }
        if let Some(iou) = opts["iou_threshold"].as_f64() {
            self.iou_threshold = iou as f32;
        }
        
        log::info!("YOLO model loaded successfully");
        Ok(())
    }
    
    fn generate(&self, input: Value) -> Result<Value> {
        let session = self.session.as_ref()
            .ok_or(crate::error::PipelineError::ModelNotLoaded)?;
        
        // Extract image data
        let image_data = input.get("image")
            .ok_or_else(|| crate::error::PipelineError::InvalidInput(
                "image data is required".to_string()
            ))?;
        
        // TODO: Implement YOLO preprocessing
        // 1. Decode image (base64 or file path)
        // 2. Resize to 640x640
        // 3. Normalize [0, 1]
        // 4. Convert to CHW format (channels first)
        // 5. Create ndarray tensor
        
        // TODO: Run inference via onnx-loader
        // let outputs = session.session().lock().unwrap()
        //     .run(inputs!["images" => preprocessed])?;
        
        // TODO: Implement YOLO postprocessing
        // 1. Extract output tensor [1, 84, 8400]
        // 2. Transpose to [8400, 84]
        // 3. Filter by confidence threshold
        // 4. Parse bounding boxes (x, y, w, h)
        // 5. Apply NMS (Non-Maximum Suppression)
        // 6. Map class IDs to labels
        
        // Placeholder response
        Ok(serde_json::json!({
            "detections": [],
            "note": "YOLO pipeline stub - preprocessing/postprocessing TODO"
        }))
    }
    
    fn unload(&mut self) -> Result<()> {
        self.session = None;
        log::info!("YOLO model unloaded");
        Ok(())
    }
}

/// YOLO class labels (COCO 80 classes)
#[rustfmt::skip]
pub const YOLO_CLASS_LABELS: [&str; 80] = [
    "person", "bicycle", "car", "motorcycle", "airplane", "bus", "train", "truck", "boat", "traffic light",
    "fire hydrant", "stop sign", "parking meter", "bench", "bird", "cat", "dog", "horse", "sheep", "cow",
    "elephant", "bear", "zebra", "giraffe", "backpack", "umbrella", "handbag", "tie", "suitcase", "frisbee",
    "skis", "snowboard", "sports ball", "kite", "baseball bat", "baseball glove", "skateboard", "surfboard",
    "tennis racket", "bottle", "wine glass", "cup", "fork", "knife", "spoon", "bowl", "banana", "apple",
    "sandwich", "orange", "broccoli", "carrot", "hot dog", "pizza", "donut", "cake", "chair", "couch",
    "potted plant", "bed", "dining table", "toilet", "tv", "laptop", "mouse", "remote", "keyboard",
    "cell phone", "microwave", "oven", "toaster", "sink", "refrigerator", "book", "clock", "vase",
    "scissors", "teddy bear", "hair drier", "toothbrush"
];

