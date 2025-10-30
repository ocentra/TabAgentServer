"""
Segmentation - Real-time person/background segmentation

MediaPipe Selfie Segmentation and Hair Segmentation.
Reference: https://ai.google.dev/edge/mediapipe/solutions/vision/image_segmenter
"""

import logging
import numpy as np
from typing import Optional, AsyncGenerator

logger = logging.getLogger(__name__)


class Segmenter:
    """
    MediaPipe Segmentation.
    
    Provides real-time segmentation for:
    - Selfie segmentation (person vs background)
    - Hair segmentation
    - Clothing segmentation (future)
    
    Useful for:
    - Virtual backgrounds
    - Background blur/replacement
    - Green screen effects
    - Portrait mode
    - AR effects
    """
    
    def __init__(
        self,
        model_selection: int = 1,
        segmentation_type: str = 'selfie'
    ):
        """
        Initialize segmenter.
        
        Args:
            model_selection: 
                0 = general model (landscape)
                1 = full-range model (best for selfies)
            segmentation_type: 'selfie' or 'hair'
        """
        self.model_selection = model_selection
        self.segmentation_type = segmentation_type
        self._segmenter = None
        self._mp = None
        
        logger.info(f"Segmenter initialized (type={segmentation_type}, model={model_selection})")
    
    def _ensure_initialized(self):
        """Lazy initialization of MediaPipe"""
        if self._segmenter is None:
            import mediapipe as mp
            self._mp = mp
            
            if self.segmentation_type == 'selfie':
                self._segmenter = mp.solutions.selfie_segmentation.SelfieSegmentation(
                    model_selection=self.model_selection
                )
                logger.info("MediaPipe Selfie Segmentation loaded")
            elif self.segmentation_type == 'hair':
                # Hair segmentation uses selfie segmentation with post-processing
                self._segmenter = mp.solutions.selfie_segmentation.SelfieSegmentation(
                    model_selection=self.model_selection
                )
                logger.info("MediaPipe Hair Segmentation loaded (using selfie model)")
            else:
                raise ValueError(f"Unknown segmentation type: {self.segmentation_type}")
    
    def segment_single(self, image: np.ndarray) -> dict:
        """
        Segment a single image.
        
        Args:
            image: RGB image as numpy array (H, W, 3)
        
        Returns:
            {
                'mask': np.ndarray (H, W) with values [0, 255],
                'mask_float': np.ndarray (H, W) with values [0.0, 1.0],
                'width': int,
                'height': int
            }
        """
        self._ensure_initialized()
        
        results = self._segmenter.process(image)
        
        if results.segmentation_mask is None:
            # Return empty mask
            return {
                'mask': np.zeros((image.shape[0], image.shape[1]), dtype=np.uint8),
                'mask_float': np.zeros((image.shape[0], image.shape[1]), dtype=np.float32),
                'width': image.shape[1],
                'height': image.shape[0]
            }
        
        # Convert float mask [0.0, 1.0] to uint8 [0, 255]
        mask_float = results.segmentation_mask
        mask_uint8 = (mask_float * 255).astype(np.uint8)
        
        if self.segmentation_type == 'hair':
            # For hair segmentation, apply additional processing
            # (In a real implementation, you'd train a specific hair model)
            # For now, we'll use the person mask as-is
            pass
        
        return {
            'mask': mask_uint8,
            'mask_float': mask_float,
            'width': mask_uint8.shape[1],
            'height': mask_uint8.shape[0]
        }
    
    async def segment_stream(self, frames: AsyncGenerator[np.ndarray, None]) -> AsyncGenerator[dict, None]:
        """
        Segment video stream.
        
        Args:
            frames: Async generator yielding RGB frames
        
        Yields:
            Segmentation masks per frame
        """
        self._ensure_initialized()
        
        async for frame in frames:
            mask = self.segment_single(frame)
            yield mask
    
    def apply_background(
        self,
        image: np.ndarray,
        mask: np.ndarray,
        background: Optional[np.ndarray] = None,
        blur_amount: int = 15
    ) -> np.ndarray:
        """
        Apply background replacement or blur.
        
        Args:
            image: Original RGB image
            mask: Segmentation mask (float [0, 1])
            background: Replacement background image (same size) or None for blur
            blur_amount: Blur kernel size (odd number)
        
        Returns:
            Composite image with new background
        """
        import cv2
        
        # Ensure mask is 3-channel
        mask_3ch = np.stack([mask] * 3, axis=-1)
        
        if background is None:
            # Blur background
            blurred = cv2.GaussianBlur(image, (blur_amount, blur_amount), 0)
            output = (mask_3ch * image + (1 - mask_3ch) * blurred).astype(np.uint8)
        else:
            # Replace background
            if background.shape != image.shape:
                background = cv2.resize(background, (image.shape[1], image.shape[0]))
            output = (mask_3ch * image + (1 - mask_3ch) * background).astype(np.uint8)
        
        return output
    
    def extract_foreground(self, image: np.ndarray, mask: np.ndarray) -> np.ndarray:
        """
        Extract foreground with alpha channel.
        
        Args:
            image: Original RGB image
            mask: Segmentation mask (float [0, 1])
        
        Returns:
            RGBA image with transparent background
        """
        # Convert mask to alpha channel [0, 255]
        alpha = (mask * 255).astype(np.uint8)
        
        # Create RGBA image
        rgba = np.dstack([image, alpha])
        
        return rgba
    
    def close(self):
        """Release resources"""
        if self._segmenter:
            self._segmenter.close()
            self._segmenter = None
            logger.info("Segmenter closed")

