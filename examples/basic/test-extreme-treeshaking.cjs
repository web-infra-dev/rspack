#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');

const chunkPath = path.join(__dirname, 'dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.js');
const chunkContent = fs.readFileSync(chunkPath, 'utf-8');

// Load usage data to get all export names
const usageDataPath = path.join(__dirname, 'dist/share-usage.json');
const shareUsageData = JSON.parse(fs.readFileSync(usageDataPath, 'utf-8'));
const lodashData = shareUsageData.consume_shared_modules['lodash-es'];

// Get all export names
const allExports = [
  ...lodashData.used_exports,
  ...lodashData.unused_exports,
  ...lodashData.possibly_unused_exports
];

console.log('=== EXTREME TREE SHAKING TESTS ===\n');
console.log('Original chunk size:', chunkContent.length, 'bytes');
console.log('Total exports:', allExports.length);

// Test 1: ALL exports false
console.log('\n--- Test 1: ALL EXPORTS FALSE ---');
const allFalseUsage = {
  used_exports: [],
  unused_exports: allExports,
  possibly_unused_exports: []
};

const result1 = optimize(chunkContent, JSON.stringify({
  lodash_usage: allFalseUsage,
  enable_webpack_tree_shaking: true
}));

const modules1 = (result1.match(/"\.\.\/\.\.\/node_modules\/\.pnpm\/lodash-es[^"]+"\s*:/g) || []).length;
console.log('Result size:', result1.length, 'bytes');
console.log('Modules remaining:', modules1);
console.log('Size reduction:', ((1 - result1.length / chunkContent.length) * 100).toFixed(1) + '%');

// Test 2: Only VERSION export
console.log('\n--- Test 2: ONLY VERSION EXPORT ---');
const versionOnlyUsage = {
  used_exports: ['VERSION'],
  unused_exports: allExports.filter(e => e !== 'VERSION'),
  possibly_unused_exports: []
};

const result2 = optimize(chunkContent, JSON.stringify({
  lodash_usage: versionOnlyUsage,
  enable_webpack_tree_shaking: true
}));

const modules2 = (result2.match(/"\.\.\/\.\.\/node_modules\/\.pnpm\/lodash-es[^"]+"\s*:/g) || []).length;
console.log('Result size:', result2.length, 'bytes');
console.log('Modules remaining:', modules2);
console.log('Size reduction:', ((1 - result2.length / chunkContent.length) * 100).toFixed(1) + '%');

// Test 3: Only simple exports (no dependencies)
console.log('\n--- Test 3: ONLY SIMPLE EXPORTS (VERSION, noop, stubTrue, stubFalse) ---');
const simpleExports = ['VERSION', 'noop', 'stubTrue', 'stubFalse', 'stubArray', 'stubObject'];
const simpleUsage = {
  used_exports: simpleExports,
  unused_exports: allExports.filter(e => !simpleExports.includes(e)),
  possibly_unused_exports: []
};

const result3 = optimize(chunkContent, JSON.stringify({
  lodash_usage: simpleUsage,
  enable_webpack_tree_shaking: true
}));

const modules3 = (result3.match(/"\.\.\/\.\.\/node_modules\/\.pnpm\/lodash-es[^"]+"\s*:/g) || []).length;
console.log('Result size:', result3.length, 'bytes');
console.log('Modules remaining:', modules3);
console.log('Size reduction:', ((1 - result3.length / chunkContent.length) * 100).toFixed(1) + '%');

// Test 4: One complex function
console.log('\n--- Test 4: ONLY ONE COMPLEX FUNCTION (debounce) ---');
const debounceOnlyUsage = {
  used_exports: ['debounce'],
  unused_exports: allExports.filter(e => e !== 'debounce'),
  possibly_unused_exports: []
};

const result4 = optimize(chunkContent, JSON.stringify({
  lodash_usage: debounceOnlyUsage,
  enable_webpack_tree_shaking: true
}));

const modules4 = (result4.match(/"\.\.\/\.\.\/node_modules\/\.pnpm\/lodash-es[^"]+"\s*:/g) || []).length;
console.log('Result size:', result4.length, 'bytes');
console.log('Modules remaining:', modules4);
console.log('Size reduction:', ((1 - result4.length / chunkContent.length) * 100).toFixed(1) + '%');

// Summary
console.log('\n=== SUMMARY ===');
console.log('All exports false:   ', modules1, 'modules');
console.log('Only VERSION:        ', modules2, 'modules');
console.log('Simple exports only: ', modules3, 'modules');
console.log('Only debounce:       ', modules4, 'modules');

// Write results for inspection
fs.writeFileSync('test-all-false-result.js', result1);
fs.writeFileSync('test-version-only-result.js', result2);
console.log('\nResults written to test-all-false-result.js and test-version-only-result.js');

// Check if any modules were completely removed
console.log('\n=== MODULE REMOVAL ANALYSIS ===');
if (modules1 === 0) {
  console.log('✅ SUCCESS: All modules removed when all exports are false!');
} else {
  console.log('❌ ISSUE: ' + modules1 + ' modules remain even with all exports false');
  console.log('This suggests the tree shaker is not working for split chunks');
}