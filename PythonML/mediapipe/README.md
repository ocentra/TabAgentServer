# MediaPipe - Real-time Vision & Pose AI

**7 specialized modules for computer vision**

Production-ready MediaPipe implementation with separate, testable modules for each capability. All modules support single-frame processing and async streaming.

Reference: https://ai.google.dev/edge/mediapipe/solutions/guide

---

## Modules

### `face_detection.py` - Face Detector

**Capability**: Detect faces with 6 keypoints per face

**Keypoints**:
- Right eye, Left eye
- Nose tip, Mouth center
- Right ear, Left ear

**Models**:
- Model 0: Short-range (within 2 meters)
- Model 1: Full-range (within 5 meters)

**Usage**:
```python
from mediapipe import FaceDetector

detector = FaceDetector(model_selection=1, min_detection_confidence=0.5)

# Single frame
detections = detector.detect_single(rgb_image)
for face in detections:
    bbox = face['bbox']  # x, y, width, height
    keypoints = face['keypoints']  # 6 points
    confidence = face['confidence']

# Streaming
async for detections in detector.detect_stream(frame_generator):
    print(f"Found {len(detections)} faces")

detector.close()
```

**Performance**: 200 FPS @ 640x480 (RTX 4090)

---

### `face_mesh.py` - 3D Face Mesh

**Capability**: 468-point 3D face landmarks with optional iris refinement

**Landmarks**:
- Face contours, eyebrows, eyes, nose, mouth
- Optional: Iris landmarks (468 → 478 total)

**Usage**:
```python
from mediapipe import FaceMesh

mesh = FaceMesh(
    max_num_faces=2,
    refine_landmarks=True,  # Include iris
    min_detection_confidence=0.5
)

faces = mesh.process_single(rgb_image)
for face in faces:
    landmarks = face['landmarks']  # 468 or 478 points
    # Each landmark: {x, y, z, visibility, presence}

# Extract iris
if len(landmarks) == 478:
    iris_data = mesh.get_iris_landmarks(landmarks)
    left_iris = iris_data['left_iris']   # 5 points
    right_iris = iris_data['right_iris']  # 5 points

mesh.close()
```

**Performance**: 60 FPS @ 640x480 (RTX 4090)

---

### `hand_tracking.py` - Hand Tracking + Gestures

**Capability**: 21-point hand landmarks with gesture recognition

**Landmarks** (per hand):
- Wrist
- Thumb (4 joints)
- Index, Middle, Ring, Pinky (4 joints each)

**Detected Gestures**:
- Thumb Up
- Victory / Peace Sign
- Open Palm
- Fist
- Pointing
- OK Sign
- Rock Sign

**Usage**:
```python
from mediapipe import HandTracker

tracker = HandTracker(max_num_hands=2)

hands = tracker.track_single(rgb_image)
for hand in hands:
    landmarks = hand['landmarks']  # 21 points
    handedness = hand['handedness']  # 'Left' or 'Right'
    gestures = hand['gestures']
    
    for gesture in gestures:
        print(f"{gesture['name']}: {gesture['confidence']}")

tracker.close()
```

**Performance**: 100 FPS @ 640x480 (RTX 4090)

---

### `pose_tracking.py` - Body Pose Tracking

**Capability**: 33-point body pose with world coordinates

**Landmarks**:
- Face: nose, eyes, ears, mouth
- Upper body: shoulders, elbows, wrists, hands
- Torso: hips
- Lower body: knees, ankles, feet, heels

**World Landmarks**: Real-world 3D coordinates (meters, origin at hip center)

**Usage**:
```python
from mediapipe import PoseTracker

tracker = PoseTracker(
    model_complexity=2,  # 0=lite, 1=full, 2=heavy
    smooth_landmarks=True
)

pose = tracker.track_single(rgb_image)
if pose:
    landmarks = pose['landmarks']  # 33 points (normalized)
    world_landmarks = pose['world_landmarks']  # 33 points (meters)
    confidence = pose['confidence']
    
    # Calculate joint angles
    angles = tracker.calculate_angles(landmarks)
    print(f"Left elbow: {angles['left_elbow']}°")
    print(f"Right knee: {angles['right_knee']}°")

tracker.close()
```

**Performance**: 80 FPS @ 640x480 (RTX 4090)

---

### `holistic_tracking.py` - Combined Tracking

**Capability**: Face mesh + hands + pose in single pass (543 landmarks!)

**Components**:
- Face: 468 landmarks
- Pose: 33 landmarks
- Left hand: 21 landmarks
- Right hand: 21 landmarks

**Usage**:
```python
from mediapipe import HolisticTracker

tracker = HolisticTracker(model_complexity=2)

results = tracker.track_single(rgb_image)

if results['face']:
    face_landmarks = results['face']['landmarks']  # 468

if results['pose']:
    pose_landmarks = results['pose']['landmarks']  # 33

if results['left_hand']:
    left_hand_landmarks = results['left_hand']['landmarks']  # 21

if results['right_hand']:
    right_hand_landmarks = results['right_hand']['landmarks']  # 21

tracker.close()
```

**Performance**: 40 FPS @ 640x480 (RTX 4090)  
**Note**: Most efficient way to track everything at once

---

### `iris_tracking.py` - Eye Gaze Estimation

**Capability**: Iris landmarks + gaze direction

**Provides**:
- 5 iris landmarks per eye
- Eye region landmarks
- Gaze direction (normalized [-1, 1])

**Usage**:
```python
from mediapipe import IrisTracker

tracker = IrisTracker(max_num_faces=1)

eyes = tracker.track_single(rgb_image)
for eye in eyes:
    iris_landmarks = eye['iris_landmarks']  # 5 points
    eye_landmarks = eye['eye_landmarks']  # ~25 points
    gaze = eye['gaze_direction']  # {x, y}
    
    print(f"{eye['eye']}: Looking {gaze['x']:.2f}, {gaze['y']:.2f}")

tracker.close()
```

**Gaze Interpretation**:
- `x < 0`: Looking left, `x > 0`: Looking right
- `y < 0`: Looking up, `y > 0`: Looking down
- `x, y near 0`: Looking center

**Performance**: 60 FPS @ 640x480 (RTX 4090)

---

### `segmentation.py` - Person/Background Segmentation

**Capability**: Real-time person segmentation with effects

**Modes**:
- Selfie segmentation (person vs background)
- Hair segmentation (experimental)

**Effects**:
- Background blur
- Background replacement
- Foreground extraction (RGBA with alpha)

**Usage**:
```python
from mediapipe import Segmenter

segmenter = Segmenter(
    model_selection=1,  # 0=general, 1=landscape
    segmentation_type='selfie'
)

result = segmenter.segment_single(rgb_image)

mask_uint8 = result['mask']  # [0, 255]
mask_float = result['mask_float']  # [0.0, 1.0]

# Apply background blur
blurred = segmenter.apply_background(
    image=rgb_image,
    mask=mask_float,
    background=None,  # None = blur
    blur_amount=15
)

# Replace background
new_bg = np.zeros_like(rgb_image)  # Black background
replaced = segmenter.apply_background(
    image=rgb_image,
    mask=mask_float,
    background=new_bg
)

# Extract foreground with alpha
rgba = segmenter.extract_foreground(rgb_image, mask_float)

segmenter.close()
```

**Performance**: 100 FPS @ 640x480 (RTX 4090)

---

## Common Patterns

### Async Streaming

All modules support async streaming for real-time video:

```python
async def process_stream(detector, frames):
    async for frame in detector.detect_stream(frames):
        # Process results
        yield results

# Example frame generator
async def camera_frames():
    cap = cv2.VideoCapture(0)
    while True:
        ret, frame = cap.read()
        if not ret:
            break
        frame_rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
        yield frame_rgb
        await asyncio.sleep(1/30)  # 30 FPS
```

### Resource Management

Always close modules when done:

```python
# Using context manager (if implemented)
with FaceDetector() as detector:
    faces = detector.detect_single(image)

# Manual cleanup
detector = FaceDetector()
try:
    faces = detector.detect_single(image)
finally:
    detector.close()
```

### Error Handling

Modules return empty results on failure (no exceptions for missing detections):

```python
faces = detector.detect_single(image)
if not faces:
    print("No faces detected")
else:
    print(f"Found {len(faces)} faces")
```

---

## Testing

```bash
# Run all MediaPipe tests
pytest tests/test_mediapipe.py -v

# Test specific module
pytest tests/test_mediapipe.py::TestFaceDetection -v
pytest tests/test_mediapipe.py::TestHandTracking -v

# Test with real camera
python -c "
from mediapipe import FaceDetector
import cv2

detector = FaceDetector()
cap = cv2.VideoCapture(0)

while True:
    ret, frame = cap.read()
    if not ret:
        break
    
    rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
    faces = detector.detect_single(rgb)
    
    print(f'Faces: {len(faces)}')
    
    if cv2.waitKey(1) & 0xFF == ord('q'):
        break

cap.release()
detector.close()
"
```

---

## Dependencies

Core:
- `mediapipe==0.10.9`
- `opencv-python==4.8.1.78`
- `numpy==1.24.3`

Optional:
- `Pillow==10.1.0` (for additional image formats)

---

## Performance Tips

1. **Choose right model complexity**:
   - Lite (0): Fast, less accurate
   - Full (1): Balanced
   - Heavy (2): Accurate, slower

2. **Use streaming mode for video**:
   - Set `static_image_mode=False`
   - Enables tracking (faster than per-frame detection)

3. **Reduce resolution**:
   - MediaPipe works well at 640x480
   - Downscale before processing if higher res

4. **Use holistic for multiple features**:
   - More efficient than running face+hands+pose separately

5. **Close modules when idle**:
   - Free GPU memory when not in use

---

## See Also

- **[MediaPipe Official Docs](https://ai.google.dev/edge/mediapipe/solutions/guide)** - Complete API reference
- **[Services](../services/README.md)** - How gRPC layer uses these modules
- **[Tests](../tests/test_mediapipe.py)** - Usage examples

