import os
import tempfile
from pathlib import Path
from typing import List
import pytest
from self_encryption import (
    DataMap,
    XorName,
    encrypt_from_file,
    decrypt_from_storage,
    streaming_decrypt_from_storage,
)

def test_file_encryption_decryption():
    with tempfile.TemporaryDirectory() as temp_dir:
        # Create test file
        input_path = Path(temp_dir) / "input.dat"
        data = b"x" * 10_000_000
        input_path.write_bytes(data)
        
        # Create output directory for chunks
        chunk_dir = Path(temp_dir) / "chunks"
        chunk_dir.mkdir()
        
        # Encrypt file
        data_map, chunk_names = encrypt_from_file(str(input_path), str(chunk_dir))
        
        # Create chunk retrieval function
        def get_chunk(hash_hex: str) -> bytes:
            chunk_path = chunk_dir / hash_hex
            return chunk_path.read_bytes()
        
        # Decrypt to new file
        output_path = Path(temp_dir) / "output.dat"
        decrypt_from_storage(data_map, str(output_path), get_chunk)
        
        # Verify
        assert input_path.read_bytes() == output_path.read_bytes()

def test_streaming_decryption():
    with tempfile.TemporaryDirectory() as temp_dir:
        # Create test file
        input_path = Path(temp_dir) / "input.dat"
        data = b"x" * 10_000_000  # 10MB
        input_path.write_bytes(data)
        
        # Create output directory for chunks
        chunk_dir = Path(temp_dir) / "chunks"
        chunk_dir.mkdir()
        
        # Encrypt file
        data_map, chunk_names = encrypt_from_file(str(input_path), str(chunk_dir))
        
        # Create parallel chunk retrieval function
        def get_chunks(hash_hexes: List[str]) -> List[bytes]:
            return [
                (chunk_dir / hash_hex).read_bytes()
                for hash_hex in hash_hexes
            ]
        
        # Decrypt using streaming
        output_path = Path(temp_dir) / "output.dat"
        streaming_decrypt_from_storage(data_map, str(output_path), get_chunks)
        
        # Verify
        assert input_path.read_bytes() == output_path.read_bytes()

if __name__ == "__main__":
    pytest.main([__file__]) 