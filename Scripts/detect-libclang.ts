#!/usr/bin/env node

/**
 * Auto-detect libclang.dll on Windows for Rust bindgen
 * Runs before cargo build to set LIBCLANG_PATH
 */

import fs from 'fs';
import path from 'path';
import process from 'process';

/**
 * Common Visual Studio install locations
 * Ordered by priority: Visual Studio first, then LLVM, then msys64 last
 */
const VS_SEARCH_PATHS: string[] = [
  // Visual Studio 2022 (highest priority)
  'C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Tools\\Llvm\\x64\\bin',
  'C:\\Program Files\\Microsoft Visual Studio\\2022\\Professional\\VC\\Tools\\Llvm\\x64\\bin',
  'C:\\Program Files\\Microsoft Visual Studio\\2022\\Enterprise\\VC\\Tools\\Llvm\\x64\\bin',
  
  // Visual Studio 2019
  'C:\\Program Files\\Microsoft Visual Studio\\2019\\Community\\VC\\Tools\\Llvm\\x64\\bin',
  'C:\\Program Files\\Microsoft Visual Studio\\2019\\Professional\\VC\\Tools\\Llvm\\x64\\bin',
  'C:\\Program Files\\Microsoft Visual Studio\\2019\\Enterprise\\VC\\Tools\\Llvm\\x64\\bin',
  
  // Standalone LLVM installations
  'C:\\Program Files\\LLVM\\bin',
  'C:\\Program Files\\LLVM\\lib',
  'C:\\Program Files (x86)\\LLVM\\bin',
  
  // msys64 (lowest priority - often incomplete)
  'C:\\msys64\\mingw64\\bin',
  'C:\\msys64\\clang64\\bin',
];

/**
 * Check if libclang.dll exists in path
 */
function checkLibclang(searchPath: string): string | null {
  const libclangPath = path.join(searchPath, 'libclang.dll');
  const exists = fs.existsSync(libclangPath);
  
  if (exists) {
    console.log(`   🔍 Checking: ${searchPath} ✅`);
  }
  
  return exists ? searchPath : null;
}

/**
 * Auto-detect libclang.dll location
 */
export function detectLibclang(): string | null {
  // Skip if not Windows
  if (process.platform !== 'win32') {
    return null;
  }
  
  // Check if already set AND verify it's valid
  if (process.env.LIBCLANG_PATH) {
    const existingPath = process.env.LIBCLANG_PATH;
    const libclangFile = path.join(existingPath, 'libclang.dll');
    
    if (fs.existsSync(libclangFile)) {
      console.log(`✅ LIBCLANG_PATH already set and valid: ${existingPath}\n`);
      return existingPath;
    } else {
      console.warn(`⚠️  LIBCLANG_PATH set to invalid path: ${existingPath}`);
      console.warn(`   (libclang.dll not found there, searching elsewhere...)\n`);
    }
  }
  
  // Search common locations
  console.log('🔍 Searching for libclang.dll...');
  for (const searchPath of VS_SEARCH_PATHS) {
    const result = checkLibclang(searchPath);
    if (result) {
      console.log(`✅ Found libclang.dll at: ${result}\n`);
      return result;
    }
  }
  
  // Not found
  console.warn('⚠️  libclang.dll not found in common locations');
  console.warn('   Rust bindgen may fail. Install Visual Studio C++ tools or LLVM.');
  return null;
}

// Run if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const result = detectLibclang();
  if (result) {
    console.log(`\nSet environment variable:`);
    console.log(`$env:LIBCLANG_PATH="${result}"`);
  } else {
    process.exit(1);
  }
}

