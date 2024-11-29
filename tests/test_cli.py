import pytest
from click.testing import CliRunner
from self_encryption.cli import cli
import tempfile
import os
from pathlib import Path

@pytest.fixture
def runner():
    return CliRunner()

@pytest.fixture
def test_data():
    return b"Hello, World!" * 1000  # Make it large enough to encrypt

def test_encrypt_decrypt_flow(runner, test_data):
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create input file
        input_file = Path(tmpdir) / "input.dat"
        input_file.write_bytes(test_data)
        
        # Create chunks directory
        chunks_dir = Path(tmpdir) / "chunks"
        chunks_dir.mkdir()
        
        # Create data map file
        data_map_file = Path(tmpdir) / "data_map.json"
        output_file = Path(tmpdir) / "output.dat"
        
        # Test encryption
        result = runner.invoke(cli, ['encrypt-file', str(input_file), str(chunks_dir)])
        assert result.exit_code == 0
        
        # Save data map
        data_map_file.write_text(result.output)
        
        # Test decryption
        result = runner.invoke(cli, [
            'decrypt-file',
            str(data_map_file),
            str(chunks_dir),
            str(output_file)
        ])
        assert result.exit_code == 0
        
        # Verify content
        assert output_file.read_bytes() == test_data

def test_streaming_decrypt(runner, test_data):
    with tempfile.TemporaryDirectory() as tmpdir:
        input_file = Path(tmpdir) / "input.dat"
        input_file.write_bytes(test_data)
        chunks_dir = Path(tmpdir) / "chunks"
        chunks_dir.mkdir()
        data_map_file = Path(tmpdir) / "data_map.json"
        output_file = Path(tmpdir) / "output.dat"
        
        # Encrypt
        result = runner.invoke(cli, ['encrypt-file', str(input_file), str(chunks_dir)])
        assert result.exit_code == 0
        data_map_file.write_text(result.output)
        
        # Test streaming decryption
        result = runner.invoke(cli, [
            'decrypt-file',
            '--streaming',
            str(data_map_file),
            str(chunks_dir),
            str(output_file)
        ])
        assert result.exit_code == 0
        assert output_file.read_bytes() == test_data 