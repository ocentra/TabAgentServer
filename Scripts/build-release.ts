#!/usr/bin/env node

/**
 * TabAgent Release Builder
 * 
 * Builds production binaries for all platforms:
 * 1. Rust server (tabagent-server.exe/binary)
 * 2. Tauri desktop app (TabAgent.exe/.app/.AppImage)
 * 
 * Copies to TabAgentDist submodule for distribution.
 */

import { spawn, exec as execCallback } from 'child_process';
import { promisify } from 'util';
import process from 'process';
import fs from 'fs';
import path from 'path';
import { detectLibclang } from './detect-libclang.js';

const exec = promisify(execCallback);

interface BuildConfig {
  platform: 'windows' | 'linux' | 'macos';
  rustTarget: string;
  serverBinary: string;
  tauriBinary: string;
  distPath: string;
}

/**
 * Get platform-specific build configuration
 */
function getBuildConfig(): BuildConfig {
  const platform = process.platform;
  
  if (platform === 'win32') {
    return {
      platform: 'windows',
      rustTarget: 'x86_64-pc-windows-msvc',
      serverBinary: 'tabagent-server.exe',
      tauriBinary: 'TabAgent.exe',
      distPath: 'NativeApp/binaries/windows'
    };
  } else if (platform === 'linux') {
    return {
      platform: 'linux',
      rustTarget: 'x86_64-unknown-linux-gnu',
      serverBinary: 'tabagent-server',
      tauriBinary: 'TabAgent.AppImage', // Or .deb depending on Tauri config
      distPath: 'NativeApp/binaries/linux'
    };
  } else if (platform === 'darwin') {
    return {
      platform: 'macos',
      rustTarget: 'aarch64-apple-darwin', // Apple Silicon, use x86_64-apple-darwin for Intel
      serverBinary: 'tabagent-server',
      tauriBinary: 'TabAgent.app',
      distPath: 'NativeApp/binaries/macos'
    };
  } else {
    throw new Error(`Unsupported platform: ${platform}`);
  }
}

/**
 * Run command with live output
 */
async function runCommand(command: string, args: string[], cwd: string): Promise<void> {
  return new Promise((resolve, reject) => {
    console.log(`\n▶️  Running: ${command} ${args.join(' ')}`);
    console.log(`   CWD: ${cwd}\n`);
    
    const child = spawn(command, args, {
      cwd,
      stdio: 'inherit',
      shell: true
    });
    
    child.on('error', reject);
    child.on('exit', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`Command failed with exit code ${code}`));
      }
    });
  });
}

/**
 * Check if TabAgentDist submodule exists
 */
function checkDistSubmodule(): string {
  const distPath = path.join(process.cwd(), 'TabAgentDist');
  
  if (!fs.existsSync(distPath)) {
    console.error('\n❌ ERROR: TabAgentDist submodule not found!');
    console.error('   Expected at:', distPath);
    console.error('\n   Initialize it with:');
    console.error('   git submodule update --init --recursive\n');
    process.exit(1);
  }
  
  return distPath;
}

/**
 * Copy file with error handling
 */
function copyFile(src: string, dest: string): void {
  try {
    // Ensure destination directory exists
    const destDir = path.dirname(dest);
    if (!fs.existsSync(destDir)) {
      fs.mkdirSync(destDir, { recursive: true });
    }
    
    fs.copyFileSync(src, dest);
    console.log(`   ✅ Copied: ${path.basename(src)} → ${dest}`);
  } catch (err) {
    console.error(`   ❌ Failed to copy ${src}:`, (err as Error).message);
    throw err;
  }
}

/**
 * Copy directory recursively (for .app bundles on macOS)
 */
function copyDirectory(src: string, dest: string): void {
  try {
    if (!fs.existsSync(dest)) {
      fs.mkdirSync(dest, { recursive: true });
    }
    
    const entries = fs.readdirSync(src, { withFileTypes: true });
    
    for (const entry of entries) {
      const srcPath = path.join(src, entry.name);
      const destPath = path.join(dest, entry.name);
      
      if (entry.isDirectory()) {
        copyDirectory(srcPath, destPath);
      } else {
        fs.copyFileSync(srcPath, destPath);
      }
    }
    
    console.log(`   ✅ Copied directory: ${path.basename(src)} → ${dest}`);
  } catch (err) {
    console.error(`   ❌ Failed to copy directory ${src}:`, (err as Error).message);
    throw err;
  }
}

/**
 * Main build process
 */
async function main(): Promise<void> {
  console.log('\n╔════════════════════════════════════════╗');
  console.log('║   TabAgent Release Build System       ║');
  console.log('╚════════════════════════════════════════╝\n');
  
  // Step 0: Detect platform
  const config = getBuildConfig();
  console.log(`🖥️  Platform: ${config.platform}`);
  console.log(`🎯 Rust Target: ${config.rustTarget}`);
  console.log('');
  
  // Step 1: Auto-detect libclang for Windows
  if (config.platform === 'windows') {
    const libclangPath = detectLibclang();
    if (libclangPath) {
      process.env.LIBCLANG_PATH = libclangPath;
    }
  }
  
  // Step 2: Check TabAgentDist submodule exists
  console.log('1️⃣  Checking TabAgentDist submodule...');
  const distRoot = checkDistSubmodule();
  console.log(`   ✅ Found at: ${distRoot}\n`);
  
  // Step 3: Build Rust server
  console.log('2️⃣  Building Rust server (tabagent-server)...');
  const rustDir = path.join(process.cwd(), 'Rust');
  
  try {
    await runCommand('cargo', ['build', '--release', '--bin', 'tabagent-server'], rustDir);
    console.log('   ✅ Rust server built successfully!\n');
  } catch (err) {
    console.error('   ❌ Rust build failed:', (err as Error).message);
    process.exit(1);
  }
  
  // Step 4: Build Tauri desktop app
  console.log('3️⃣  Building Tauri desktop app (TabAgent)...');
  
  try {
    // Build frontends first
    console.log('   📦 Building Dashboard...');
    await runCommand('npm', ['run', 'build'], path.join(process.cwd(), 'dashboard'));
    
    console.log('   📦 Building Agent Builder...');
    await runCommand('npm', ['run', 'build'], path.join(process.cwd(), 'agent-builder'));
    
    // Build Tauri
    console.log('   🏗️  Building Tauri bundle...');
    await runCommand('npm', ['run', 'build'], process.cwd());
    
    console.log('   ✅ Tauri app built successfully!\n');
  } catch (err) {
    console.error('   ❌ Tauri build failed:', (err as Error).message);
    process.exit(1);
  }
  
  // Step 5: Copy binaries to TabAgentDist
  console.log('4️⃣  Copying binaries to TabAgentDist...\n');
  
  const distBinariesPath = path.join(distRoot, config.distPath);
  
  try {
    // Copy Rust server binary
    const serverSrc = path.join(rustDir, 'target', 'release', config.serverBinary);
    const serverDest = path.join(distBinariesPath, config.serverBinary);
    
    if (fs.existsSync(serverSrc)) {
      copyFile(serverSrc, serverDest);
    } else {
      console.warn(`   ⚠️  Server binary not found: ${serverSrc}`);
    }
    
    // Copy Tauri binary (platform-specific)
    if (config.platform === 'windows') {
      const tauriSrc = path.join(process.cwd(), 'src-tauri', 'target', 'release', config.tauriBinary);
      const tauriDest = path.join(distBinariesPath, config.tauriBinary);
      
      if (fs.existsSync(tauriSrc)) {
        copyFile(tauriSrc, tauriDest);
      } else {
        console.warn(`   ⚠️  Tauri binary not found: ${tauriSrc}`);
      }
    } else if (config.platform === 'linux') {
      // Linux: Look for .AppImage or .deb in bundle directory
      const bundleDir = path.join(process.cwd(), 'src-tauri', 'target', 'release', 'bundle');
      
      // Try AppImage first
      const appImagePath = path.join(bundleDir, 'appimage', config.tauriBinary);
      if (fs.existsSync(appImagePath)) {
        copyFile(appImagePath, path.join(distBinariesPath, config.tauriBinary));
      } else {
        console.warn(`   ⚠️  AppImage not found, checking for .deb...`);
        // Could also check for .deb files here
      }
    } else if (config.platform === 'macos') {
      // macOS: Copy entire .app bundle
      const appBundleSrc = path.join(process.cwd(), 'src-tauri', 'target', 'release', 'bundle', 'macos', config.tauriBinary);
      const appBundleDest = path.join(distBinariesPath, config.tauriBinary);
      
      if (fs.existsSync(appBundleSrc)) {
        copyDirectory(appBundleSrc, appBundleDest);
      } else {
        console.warn(`   ⚠️  .app bundle not found: ${appBundleSrc}`);
      }
    }
    
  } catch (err) {
    console.error('\n❌ Failed to copy binaries:', (err as Error).message);
    process.exit(1);
  }
  
  // Step 6: Success summary
  console.log('\n╔════════════════════════════════════════╗');
  console.log('║   ✅ Build Complete!                   ║');
  console.log('╚════════════════════════════════════════╝\n');
  
  console.log('📋 Build Summary:');
  console.log(`   Platform: ${config.platform}`);
  console.log(`   Binaries copied to: ${distBinariesPath}`);
  console.log('');
  console.log('📦 Next Steps:');
  console.log('   1. Review binaries in TabAgentDist/');
  console.log('   2. Commit to TabAgentDist submodule:');
  console.log('      cd TabAgentDist');
  console.log('      git add NativeApp/binaries/');
  console.log(`      git commit -m "Update ${config.platform} binaries v{version}"`);
  console.log('      git push');
  console.log('   3. GitHub Actions will create installers!');
  console.log('');
}

// Run
main().catch((err) => {
  console.error('\n❌ Fatal error:', err);
  process.exit(1);
});

