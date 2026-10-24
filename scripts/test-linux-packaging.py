#!/usr/bin/env python3
"""Check package completeness and staging with synthetic executables, without Flatpak."""
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import sys

sys.dont_write_bytecode = True
import unittest

SCRIPTS = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('prepare_flatpak', SCRIPTS / 'prepare-flatpak.py')
packaging = importlib.util.module_from_spec(spec)
spec.loader.exec_module(packaging)


class StagingTests(unittest.TestCase):
    def package(self, root, missing=None):
        data = root / 'data.tar.gz'
        with tarfile.open(data, 'w:gz') as archive:
            for name in ['localog'] + ['localog-' + tool for tool in packaging.TOOLS]:
                if name == missing:
                    continue
                content = b'#!/bin/sh\nexit 0\n'
                item = tarfile.TarInfo('./usr/bin/' + name)
                item.size = len(content)
                item.mode = 0o755
                archive.addfile(item, io.BytesIO(content))
        deb = root / 'localog.deb'
        subprocess.run(['ar', 'rc', str(deb), str(data)], check=True, capture_output=True)
        return deb

    def test_complete_bundle_and_unchanged_input(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            deb = self.package(root)
            before = deb.read_bytes()
            manifest = packaging.prepare(deb, root / 'stage with spaces')
            config = json.loads(manifest.read_text())
            for source in config['modules'][0]['sources']:
                self.assertTrue((manifest.parent / source['path']).is_file())
            self.assertEqual((manifest.parent / 'localog.deb').read_bytes(), before)
            self.assertEqual(deb.read_bytes(), before)
            self.assertNotIn('--filesystem=home', config['finish-args'])
            self.assertNotIn('--filesystem=host', config['finish-args'])
            with self.assertRaisesRegex(ValueError, 'existing files'):
                packaging.prepare(deb, manifest.parent)

    def test_missing_sidecar_is_rejected_before_staging(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            deb = self.package(root, missing='localog-llama-server')
            with self.assertRaisesRegex(ValueError, 'localog-llama-server'):
                packaging.prepare(deb, root / 'stage')
            self.assertFalse((root / 'stage').exists())

    def test_development_launcher(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(['python3', str(SCRIPTS / 'install-linux-dev-launcher.py'),
                            '--bin-dir', str(root / 'bin with spaces'),
                            '--data-dir', str(root / 'data')], check=True, capture_output=True)
            launcher = root / 'bin with spaces/localog-dev'
            subprocess.run(['bash', '-n', str(launcher)], check=True)
            desktop = root / 'data/applications/localog-development.desktop'
            subprocess.run(['desktop-file-validate', str(desktop)], check=True)
            self.assertIn('Terminal=true', desktop.read_text())
            self.assertIn(str(launcher), desktop.read_text())


if __name__ == '__main__':
    unittest.main()
