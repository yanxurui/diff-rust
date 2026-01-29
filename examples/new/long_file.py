#!/usr/bin/env python3
"""
A sample Python module demonstrating various features.
This file is intentionally long to test the collapsed view.
"""

import os
import sys
import json
import logging
from typing import List, Dict, Optional
from dataclasses import dataclass
from pathlib import Path

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


@dataclass
class Config:
    """Configuration settings for the application."""
    name: str
    version: str
    debug: bool = False
    max_retries: int = 5
    timeout: float = 30.0


class DataProcessor:
    """Process data from various sources."""
    
    def __init__(self, config: Config):
        self.config = config
        self.data: List[Dict] = []
        self._initialized = False
    
    def initialize(self) -> bool:
        """Initialize the processor."""
        logger.info(f"Initializing {self.config.name}")
        self._initialized = True
        return True
    
    def load_data(self, filepath: Path) -> bool:
        """Load data from a JSON file."""
        if not self._initialized:
            raise RuntimeError("Processor not initialized")
        
        try:
            with open(filepath, 'r') as f:
                self.data = json.load(f)
            logger.info(f"Loaded {len(self.data)} records")
            return True
        except FileNotFoundError:
            logger.error(f"File not found: {filepath}")
            return False
        except json.JSONDecodeError as e:
            logger.error(f"Invalid JSON: {e}")
            return False
    
    def process(self) -> List[Dict]:
        """Process the loaded data."""
        if not self.data:
            return []
        
        results = []
        for item in self.data:
            processed = self._process_item(item)
            if processed:
                results.append(processed)
        
        return results
    
    def _process_item(self, item: Dict) -> Optional[Dict]:
        """Process a single item."""
        if 'id' not in item:
            return None
        
        return {
            'id': item['id'],
            'processed': True,
            'original': item
        }
    
    def save_results(self, results: List[Dict], output_path: Path) -> bool:
        """Save processed results to a file."""
        try:
            with open(output_path, 'w') as f:
                json.dump(results, f, indent=2)
            logger.info(f"Saved {len(results)} results to {output_path}")
            return True
        except IOError as e:
            logger.error(f"Failed to save: {e}")
            return False


def validate_config(config: Config) -> bool:
    """Validate configuration settings."""
    if not config.name:
        return False
    if config.max_retries < 0:
        return False
    if config.timeout <= 0:
        return False
    return True


def create_default_config() -> Config:
    """Create a default configuration."""
    return Config(
        name="default",
        version="1.0.0",
        debug=True,
        max_retries=5,
        timeout=30.0
    )


def main():
    """Main entry point."""
    config = create_default_config()
    
    if not validate_config(config):
        logger.error("Invalid configuration")
        sys.exit(1)
    
    processor = DataProcessor(config)
    processor.initialize()
    
    # Example usage
    input_file = Path("data/input.json")
    output_file = Path("data/output.json")
    
    if processor.load_data(input_file):
        results = processor.process()
        processor.save_results(results, output_file)
        logger.info("Processing complete")
    else:
        logger.error("Failed to load data")
        sys.exit(1)


if __name__ == "__main__":
    main()
