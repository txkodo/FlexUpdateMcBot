#!/usr/bin/env python3

import os
import re
import json
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Optional, Dict, Any
import requests
import toml

class AzaleaUpdater:
    def __init__(self):
        self.workspace_root = Path()
        self.codegen_dir = self.workspace_root / "codegen"
        self.versions_dir = self.workspace_root / "versions"
        self.azalea_repo = "https://github.com/azalea-rs/azalea"
        
    def get_latest_azalea_commit(self) -> str:
        """Get the latest commit hash from azalea repository"""
        response = requests.get(f"https://api.github.com/repos/azalea-rs/azalea/commits/main")
        response.raise_for_status()
        return response.json()["sha"]
    
    def get_commit_info(self, commit_hash: str) -> Dict[str, Any]:
        """Get commit information including date"""
        response = requests.get(f"https://api.github.com/repos/azalea-rs/azalea/commits/{commit_hash}")
        response.raise_for_status()
        return response.json()
    
    def get_azalea_cargo_toml(self, commit_hash: str) -> str:
        """Get Cargo.toml content from azalea repository"""
        response = requests.get(f"https://raw.githubusercontent.com/azalea-rs/azalea/{commit_hash}/Cargo.toml")
        response.raise_for_status()
        return response.text
    
    def get_azalea_cargo_lock(self, commit_hash: str) -> str:
        """Get Cargo.lock content from azalea repository"""
        response = requests.get(f"https://raw.githubusercontent.com/azalea-rs/azalea/{commit_hash}/Cargo.lock")
        response.raise_for_status()
        return response.text
    
    def extract_mc_version(self, cargo_toml_content: str) -> str:
        """Extract minecraft version from azalea Cargo.toml"""
        cargo_data = toml.loads(cargo_toml_content)
        version = cargo_data["workspace"]["package"]["version"]
        # Extract MC version from format like "0.13.0+mc1.21.7"
        match = re.search(r'\+mc(.+)', version)
        if not match:
            # フォールバック: azaleaのREADMEから抽出を試行
            try:
                response = requests.get(f"https://raw.githubusercontent.com/azalea-rs/azalea/{commit_hash}/README.md")
                response.raise_for_status()
                readme_content = response.text
                readme_match = re.search(r'Currently supported Minecraft version:\s*`?([0-9]+\.[0-9]+(?:\.[0-9]+)?)`?', readme_content, re.IGNORECASE)
                if readme_match:
                    return readme_match.group(1)
            except Exception:
                pass
            raise ValueError(f"Could not extract MC version from: {version}")
        return match.group(1)
    
    def get_current_azalea_oid(self) -> str:
        """Get current azalea OID from tracking file"""
        oid_file = self.codegen_dir / "LATEST_AZALEA_OID"
        if oid_file.exists():
            return oid_file.read_text().strip()
        return ""
    
    def get_current_mc_version(self) -> str:
        """Get current MC version from tracking file"""
        version_file = self.codegen_dir / "LATEST_MC_VERSION"
        if version_file.exists():
            return version_file.read_text().strip()
        return ""
    
    def update_azalea_oid(self, oid: str):
        """Update azalea OID tracking file"""
        oid_file = self.codegen_dir / "LATEST_AZALEA_OID"
        oid_file.write_text(oid)
    
    def update_mc_version(self, version: str):
        """Update MC version tracking file"""
        version_file = self.codegen_dir / "LATEST_MC_VERSION"
        version_file.write_text(version)
    
    def copy_version_directory(self, from_version: str, to_version: str):
        """Copy version directory from one MC version to another"""
        from_dir = self.versions_dir / from_version
        to_dir = self.versions_dir / to_version
        
        if to_dir.exists():
            return
        
        if not from_dir.exists():
            raise FileNotFoundError(f"Source version directory not found: {from_dir}")
        
        shutil.copytree(from_dir, to_dir)
    
    def update_cargo_toml_revisions(self, mc_version: str, azalea_commit: str):
        """Update azalea revisions in Cargo.toml"""
        cargo_toml_path = self.versions_dir / mc_version / "Cargo.toml"
        
        with open(cargo_toml_path, 'r') as f:
            content = f.read()
        
        # Update azalea revisions
        content = re.sub(
            r'(azalea[^=]*=\s*{[^}]*rev\s*=\s*")[^"]*(")',
            rf'\g<1>{azalea_commit}\g<2>',
            content
        )
        
        with open(cargo_toml_path, 'w') as f:
            f.write(content)
    
    def update_dependency_versions(self, mc_version: str, azalea_cargo_lock: str):
        """Update anyhow and tokio versions to match azalea"""
        cargo_toml_path = self.versions_dir / mc_version / "Cargo.toml"
        
        # Parse azalea's Cargo.lock to get dependency versions
        anyhow_version = self.extract_version_from_lock(azalea_cargo_lock, "anyhow")
        tokio_version = self.extract_version_from_lock(azalea_cargo_lock, "tokio")
        
        # Update Cargo.toml
        with open(cargo_toml_path, 'r') as f:
            content = f.read()
        
        if anyhow_version:
            content = re.sub(
                r'(anyhow\s*=\s*")[^"]*(")',
                rf'\g<1>{anyhow_version}\g<2>',
                content
            )
        
        if tokio_version:
            content = re.sub(
                r'(tokio\s*=\s*")[^"]*(")',
                rf'\g<1>{tokio_version}\g<2>',
                content
            )
        
        with open(cargo_toml_path, 'w') as f:
            f.write(content)
    
    def extract_version_from_lock(self, cargo_lock_content: str, package_name: str) -> Optional[str]:
        """Extract version of a package from Cargo.lock content"""
        lines = cargo_lock_content.split('\n')
        for i, line in enumerate(lines):
            if line.startswith(f'name = "{package_name}"'):
                # Look for version in the next few lines
                for j in range(i + 1, min(i + 5, len(lines))):
                    if lines[j].startswith('version = '):
                        return lines[j].split('"')[1]
        return None
    
    def update_cargo_lock(self, mc_version: str, azalea_cargo_lock: str):
        """Update Cargo.lock to match azalea's"""
        cargo_lock_path = self.versions_dir / mc_version / "Cargo.lock"
        with open(cargo_lock_path, 'w') as f:
            f.write(azalea_cargo_lock)
    
    def update_rust_toolchain(self, mc_version: str, commit_date: str):
        """Update rust-toolchain to match azalea's commit date"""
        toolchain_path = self.versions_dir / mc_version / "rust-toolchain"
        
        # Convert commit date to nightly format
        # commit_date format: "2024-01-15T10:30:00Z"
        date_part = commit_date.split('T')[0]
        nightly_version = f"nightly-{date_part}"
        
        with open(toolchain_path, 'w') as f:
            f.write(nightly_version)
    
    def run_cargo_update(self, mc_version: str) -> bool:
        """Run cargo update in the version directory"""
        version_dir = self.versions_dir / mc_version
        
        try:
            # Change to version directory and run cargo update
            result = subprocess.run(
                ["cargo", "update"],
                cwd=version_dir,
                capture_output=True,
                text=True,
                timeout=300  # 5 minutes timeout
            )
            
            if result.returncode == 0:
                print(f"cargo update successful for {mc_version}")
                return True
            else:
                print(f"cargo update failed for {mc_version}: {result.stderr}")
                return False
                
        except subprocess.TimeoutExpired:
            print(f"cargo update timed out for {mc_version}")
            return False
        except Exception as e:
            print(f"Error running cargo update for {mc_version}: {e}")
            return False
    
    def update_azalea(self, commit_hash: Optional[str] = None) -> Dict[str, Any]:
        """Main function to update azalea"""
        if commit_hash is None:
            commit_hash = self.get_latest_azalea_commit()
        
        current_oid = self.get_current_azalea_oid()
        
        # Check if already up to date
        if current_oid == commit_hash:
            return {
                "status": "up_to_date",
                "message": "Already up to date"
            }
        
        try:
            # Get commit info and azalea files
            commit_info = self.get_commit_info(commit_hash)
            cargo_toml = self.get_azalea_cargo_toml(commit_hash)
            cargo_lock = self.get_azalea_cargo_lock(commit_hash)
            
            # Extract MC version
            mc_version = self.extract_mc_version(cargo_toml)
            current_mc_version = self.get_current_mc_version()
            
            # Create version directory if it doesn't exist
            version_dir = self.versions_dir / mc_version
            if not version_dir.exists():
                if current_mc_version:
                    self.copy_version_directory(current_mc_version, mc_version)
                else:
                    raise FileNotFoundError(f"No base version to copy from")
            
            # Update files
            self.update_mc_version(mc_version)
            self.update_cargo_toml_revisions(mc_version, commit_hash)
            self.update_dependency_versions(mc_version, cargo_lock)
            self.update_cargo_lock(mc_version, cargo_lock)
            self.update_rust_toolchain(mc_version, commit_info["commit"]["committer"]["date"])
            self.update_azalea_oid(commit_hash)
            
            return {
                "status": "updated",
                "mc_version": mc_version,
                "message": f"Updated to {commit_hash[:8]} (MC {mc_version})",
            }
            
        except Exception as e:
            return {
                "status": "error",
                "message": f"Failed to update: {e}"
            }

def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="Update azalea dependencies")
    parser.add_argument("--commit", help="Specific commit hash to update to")
    
    args = parser.parse_args()
    
    updater = AzaleaUpdater()
    result = updater.update_azalea(args.commit)
    
    print(json.dumps(result, indent=2))

    with open(os.environ["GITHUB_OUTPUT"], "a") as f:
        f.write(f"status={result['status']}\n")
        f.write(f"message={result['message']}\n")
        if "mc_version" in result:
            f.write(f"mc_version={result['mc_version']}\n")
    
    if result["status"] == "error":
        sys.exit(1)

if __name__ == "__main__":
    main()