#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const chunkPath = path.join(__dirname, 'dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.js');
const chunkContent = fs.readFileSync(chunkPath, 'utf-8');

console.log('=== ANALYZING LODASH CHUNK EXPORTS ===\n');

// Find all module definitions
const moduleRegex = /"(\.\.\/\.\.\/node_modules\/\.pnpm\/lodash-es[^"]+)"\s*:\s*(?:\/\*[\s\S]*?\*\/\s*)?(?:\()?function\s*\([^)]*\)\s*\{/g;
const modules = [];
let match;

while ((match = moduleRegex.exec(chunkContent)) !== null) {
  modules.push({
    path: match[1],
    name: match[1].split('/').pop().replace('.js', ''),
    startIndex: match.index
  });
}

console.log('Total modules found:', modules.length);

// Analyze each module for exports
console.log('\n--- ANALYZING MODULE EXPORTS ---');
const exportingModules = [];
const nonExportingModules = [];

for (let i = 0; i < modules.length; i++) {
  const module = modules[i];
  const nextModule = modules[i + 1];
  
  // Extract module content
  const moduleStart = module.startIndex;
  const moduleEnd = nextModule ? nextModule.startIndex : chunkContent.length;
  const moduleContent = chunkContent.substring(moduleStart, moduleEnd);
  
  // Check for exports
  const hasExports = moduleContent.includes('__webpack_require__.d(__webpack_exports__');
  const exportMatches = moduleContent.match(/__webpack_require__\.d\(__webpack_exports__,\s*\{([^}]+)\}/g) || [];
  
  if (hasExports) {
    const exports = [];
    for (const exportMatch of exportMatches) {
      // Extract individual export names
      const exportDefs = exportMatch.match(/["']?(\w+)["']?\s*:\s*\(\)\s*=>/g) || [];
      exports.push(...exportDefs.map(e => e.match(/["']?(\w+)["']?/)[1]));
    }
    exportingModules.push({ ...module, exports });
  } else {
    nonExportingModules.push(module);
  }
}

console.log('\nModules with exports:', exportingModules.length);
console.log('Modules without exports:', nonExportingModules.length);

// Show main lodash module
const mainModule = exportingModules.find(m => m.name === 'lodash');
if (mainModule) {
  console.log('\n--- MAIN LODASH MODULE ---');
  console.log('Exports:', mainModule.exports.length);
  console.log('Sample exports:', mainModule.exports.slice(0, 10));
}

// Analyze dependencies
console.log('\n--- ANALYZING MODULE DEPENDENCIES ---');
const moduleWithDeps = [];
const moduleWithoutDeps = [];

for (let i = 0; i < modules.length; i++) {
  const module = modules[i];
  const nextModule = modules[i + 1];
  
  const moduleStart = module.startIndex;
  const moduleEnd = nextModule ? nextModule.startIndex : chunkContent.length;
  const moduleContent = chunkContent.substring(moduleStart, moduleEnd);
  
  // Count __webpack_require__ calls (excluding __webpack_require__.d and other methods)
  // Match patterns like __webpack_require__("../../node_modules/.pnpm/lodash-es...")
  const requirePattern = /__webpack_require__\(["']([^"']+)["']\)/g;
  const requireMatches = [];
  let requireMatch;
  while ((requireMatch = requirePattern.exec(moduleContent)) !== null) {
    // Skip if it's a method call like __webpack_require__.d
    const beforeMatch = moduleContent.substring(Math.max(0, requireMatch.index - 1), requireMatch.index);
    if (beforeMatch !== '.') {
      requireMatches.push(requireMatch[1]);
    }
  }
  const requireCalls = requireMatches.length;
  
  if (requireCalls > 0) {
    moduleWithDeps.push({ ...module, dependencies: requireCalls });
  } else {
    moduleWithoutDeps.push(module);
  }
}

console.log('Modules with dependencies:', moduleWithDeps.length);
console.log('Modules without dependencies (leaf modules):', moduleWithoutDeps.length);

// Find truly isolated modules (no deps, no exports)
const isolatedModules = modules.filter(m => {
  const isExporting = exportingModules.some(em => em.path === m.path);
  const hasDeps = moduleWithDeps.some(md => md.path === m.path);
  return !isExporting && !hasDeps;
});

console.log('\n--- ISOLATED MODULES (no exports, no dependencies) ---');
console.log('Count:', isolatedModules.length);
if (isolatedModules.length > 0) {
  console.log('Examples:', isolatedModules.slice(0, 10).map(m => m.name));
}

// Check for the actual exports we care about
console.log('\n--- CHECKING SPECIFIC EXPORTS ---');
const targetExports = ['map', 'filter', 'debounce', 'throttle', 'VERSION', 'chunk', 'compact'];
for (const exportName of targetExports) {
  const found = exportingModules.find(m => m.exports.includes(exportName));
  if (found) {
    console.log(`Export '${exportName}': Found in module '${found.name}'`);
  } else {
    console.log(`Export '${exportName}': NOT FOUND`);
  }
}

// Analyze why so many modules remain
console.log('\n=== ANALYSIS: WHY 640 MODULES REMAIN ===');
console.log('1. Split chunk format - no entry points means all modules treated as potentially reachable');
console.log('2. Module count breakdown:');
console.log(`   - Modules with exports: ${exportingModules.length}`);
console.log(`   - Modules without exports: ${nonExportingModules.length}`);
console.log(`   - Leaf modules (no deps): ${moduleWithoutDeps.length}`);
console.log(`   - Isolated modules: ${isolatedModules.length}`);

// Sample some internal modules
console.log('\n--- SAMPLE INTERNAL MODULES ---');
const internalModules = modules.filter(m => m.name.startsWith('_'));
console.log('Internal modules (start with _):', internalModules.length);
console.log('Examples:', internalModules.slice(0, 20).map(m => m.name));

// Write analysis for further inspection
const analysis = {
  totalModules: modules.length,
  exportingModules: exportingModules.length,
  nonExportingModules: nonExportingModules.length,
  isolatedModules: isolatedModules.length,
  moduleList: modules.map(m => ({
    name: m.name,
    hasExports: exportingModules.some(em => em.path === m.path),
    hasDependencies: moduleWithDeps.some(md => md.path === m.path),
    exports: exportingModules.find(em => em.path === m.path)?.exports || []
  }))
};

fs.writeFileSync('lodash-chunk-analysis.json', JSON.stringify(analysis, null, 2));
console.log('\nDetailed analysis written to lodash-chunk-analysis.json');