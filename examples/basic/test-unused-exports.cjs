#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');

// Read the optimized lodash chunk (with exports already removed)
const optimizedPath = path.join(__dirname, 'dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.local-optimized.js');
const optimizedContent = fs.readFileSync(optimizedPath, 'utf-8');

console.log('=== CHECKING FOR UNUSED EXPORTS IN OPTIMIZED CHUNK ===\n');
console.log('Optimized chunk size:', optimizedContent.length, 'bytes');

// Count modules
const moduleCount = (optimizedContent.match(/"\.\.\/\.\.\/node_modules\/\.pnpm\/lodash-es[^"]+"\s*:/g) || []).length;
console.log('Total modules:', moduleCount);

// Look for export statements
const exportStatements = optimizedContent.match(/__webpack_require__\.d\(__webpack_exports__[^)]+\)/g) || [];
console.log('\nExport statements found:', exportStatements.length);

// Check specific exports that should have been removed
const unusedExports = ['chunk', 'compact', 'concat', 'difference', 'drop', 'dropRight'];
console.log('\n--- Checking if unused exports were removed ---');

for (const exportName of unusedExports) {
  // Check if export definition exists
  const exportRegex = new RegExp(`["']${exportName}["']\\s*:\\s*\\(\\)\\s*=>`, 'g');
  const found = exportRegex.test(optimizedContent);
  console.log(`Export '${exportName}':`, found ? '❌ STILL EXISTS' : '✅ REMOVED');
}

// Check for the main lodash module that re-exports everything
console.log('\n--- Checking main lodash module ---');
const mainModuleRegex = /"\.\.\/\.\.\/node_modules\/\.pnpm\/lodash-es[^"]*\/lodash\.js"\s*:/;
const hasMainModule = mainModuleRegex.test(optimizedContent);
console.log('Main lodash.js module:', hasMainModule ? 'EXISTS' : 'REMOVED');

if (hasMainModule) {
  // Extract the main module content
  const match = optimizedContent.match(/"\.\.\/\.\.\/node_modules\/\.pnpm\/lodash-es[^"]*\/lodash\.js"\s*:\s*function[^{]*\{([^}]+(?:\{[^}]*\}[^}]*)*)\}/);
  if (match) {
    const mainModuleContent = match[1];
    const exportCount = (mainModuleContent.match(/__webpack_require__\.d\(__webpack_exports__/g) || []).length;
    console.log('Export definitions in main module:', exportCount);
    
    // Show a sample of exports
    const sampleExports = mainModuleContent.match(/["'](\w+)["']\s*:\s*\(\)\s*=>/g) || [];
    console.log('Sample exports (first 10):', sampleExports.slice(0, 10).map(e => e.match(/["'](\w+)["']/)[1]));
  }
}

// Check for modules that should be completely unreachable
console.log('\n--- Checking for completely unreachable modules ---');
const usedFunctions = ['map', 'filter', 'debounce', 'throttle'];
console.log('Used functions:', usedFunctions);

// Get all module paths
const modulePaths = optimizedContent.match(/"\.\.\/\.\.\/node_modules\/\.pnpm\/lodash-es[^"]+"/g) || [];
const uniqueModules = [...new Set(modulePaths)];
console.log('\nTotal unique module paths:', uniqueModules.length);

// Sample some module paths to see what's included
console.log('\nSample module paths (first 20):');
uniqueModules.slice(0, 20).forEach(path => {
  const moduleName = path.match(/\/([^/]+)\.js"/)?.[1] || 'unknown';
  console.log(`  - ${moduleName}`);
});