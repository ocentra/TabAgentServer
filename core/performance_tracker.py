"""
Performance Tracking Utilities

Tracks inference performance metrics:
- TTFT (Time To First Token)
- TPS (Tokens Per Second)
- Token counts
- Total generation time
"""

import time
from typing import Optional, Dict, Any
from dataclasses import dataclass, field


@dataclass
class GenerationMetrics:
    """
    Metrics for a single generation request.
    
    Attributes:
        start_time: When generation started
        first_token_time: When first token was generated
        end_time: When generation completed
        input_tokens: Number of input tokens
        output_tokens: Number of output tokens generated
    """
    start_time: float = field(default_factory=time.time)
    first_token_time: Optional[float] = None
    end_time: Optional[float] = None
    input_tokens: int = 0
    output_tokens: int = 0
    
    def mark_first_token(self) -> None:
        """Mark when first token is generated"""
        if self.first_token_time is None:
            self.first_token_time = time.time()
    
    def mark_complete(self) -> None:
        """Mark when generation is complete"""
        if self.end_time is None:
            self.end_time = time.time()
    
    def increment_output_tokens(self, count: int = 1) -> None:
        """Increment output token count"""
        self.output_tokens += count
    
    @property
    def ttft(self) -> Optional[float]:
        """
        Time To First Token in milliseconds.
        
        Returns:
            TTFT in ms or None if first token not yet generated
        """
        if self.first_token_time is None:
            return None
        return (self.first_token_time - self.start_time) * 1000
    
    @property
    def total_time(self) -> Optional[float]:
        """
        Total generation time in milliseconds.
        
        Returns:
            Total time in ms or None if not complete
        """
        if self.end_time is None:
            return None
        return (self.end_time - self.start_time) * 1000
    
    @property
    def tps(self) -> Optional[float]:
        """
        Tokens Per Second (output tokens only).
        
        Returns:
            TPS or None if not complete
        """
        if self.end_time is None or self.output_tokens == 0:
            return None
        
        generation_time = self.end_time - (self.first_token_time or self.start_time)
        if generation_time <= 0:
            return None
        
        return self.output_tokens / generation_time
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert metrics to dictionary"""
        return {
            "time_to_first_token": self.ttft,
            "tokens_per_second": self.tps,
            "input_tokens": self.input_tokens,
            "output_tokens": self.output_tokens,
            "total_time": self.total_time
        }


class PerformanceTracker:
    """
    Tracks performance metrics across multiple generations.
    
    Maintains both current generation metrics and aggregate statistics.
    """
    
    def __init__(self):
        """Initialize performance tracker"""
        self._current_metrics: Optional[GenerationMetrics] = None
        self._last_completed_metrics: Optional[GenerationMetrics] = None
        
        # Aggregate stats
        self._total_requests: int = 0
        self._total_input_tokens: int = 0
        self._total_output_tokens: int = 0
        self._total_generation_time: float = 0.0
    
    def start_generation(self, input_tokens: int = 0) -> GenerationMetrics:
        """
        Start tracking a new generation.
        
        Args:
            input_tokens: Number of input tokens
            
        Returns:
            New GenerationMetrics instance
        """
        self._current_metrics = GenerationMetrics(input_tokens=input_tokens)
        return self._current_metrics
    
    def mark_first_token(self) -> None:
        """Mark first token generated for current generation"""
        if self._current_metrics:
            self._current_metrics.mark_first_token()
    
    def increment_output_tokens(self, count: int = 1) -> None:
        """Increment output token count for current generation"""
        if self._current_metrics:
            self._current_metrics.increment_output_tokens(count)
    
    def complete_generation(self) -> None:
        """Mark current generation as complete and update aggregates"""
        if self._current_metrics:
            self._current_metrics.mark_complete()
            
            # Update aggregates
            self._total_requests += 1
            self._total_input_tokens += self._current_metrics.input_tokens
            self._total_output_tokens += self._current_metrics.output_tokens
            
            if self._current_metrics.total_time:
                self._total_generation_time += self._current_metrics.total_time / 1000  # Convert to seconds
            
            # Save as last completed
            self._last_completed_metrics = self._current_metrics
            self._current_metrics = None
    
    def get_current_stats(self) -> Dict[str, Any]:
        """
        Get current generation statistics.
        
        Returns:
            Dictionary with current metrics or last completed metrics
        """
        metrics = self._current_metrics or self._last_completed_metrics
        
        if metrics:
            return metrics.to_dict()
        
        return {
            "time_to_first_token": None,
            "tokens_per_second": None,
            "input_tokens": 0,
            "output_tokens": 0,
            "total_time": None
        }
    
    def get_aggregate_stats(self) -> Dict[str, Any]:
        """
        Get aggregate statistics across all generations.
        
        Returns:
            Dictionary with aggregate metrics
        """
        avg_tps = None
        if self._total_generation_time > 0:
            avg_tps = self._total_output_tokens / self._total_generation_time
        
        return {
            "total_requests": self._total_requests,
            "total_input_tokens": self._total_input_tokens,
            "total_output_tokens": self._total_output_tokens,
            "average_tokens_per_second": avg_tps,
            "total_generation_time": self._total_generation_time
        }
    
    def reset_aggregates(self) -> None:
        """Reset aggregate statistics"""
        self._total_requests = 0
        self._total_input_tokens = 0
        self._total_output_tokens = 0
        self._total_generation_time = 0.0

