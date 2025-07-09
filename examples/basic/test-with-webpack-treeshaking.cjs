#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');

const chunkPath = path.join(__dirname, 'dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.js');
const chunkContent = fs.readFileSync(chunkPath, 'utf-8');

// Load usage data
const usageDataPath = path.join(__dirname, 'dist/share-usage.json');
const shareUsageData = JSON.parse(fs.readFileSync(usageDataPath, 'utf-8'));
const lodashData = shareUsageData.consume_shared_modules['lodash-es'];

const lodashUsage = {
  used_exports: lodashData.used_exports || [],
  unused_exports: lodashData.unused_exports || [],
  possibly_unused_exports: lodashData.possibly_unused_exports || []
};

console.log('=== TESTING WITH AND WITHOUT WEBPACK TREE-SHAKING ===\n');
console.log('Original size:', chunkContent.length, 'bytes');
console.log('Used exports:', lodashUsage.used_exports.length);
console.log('Unused exports:', lodashUsage.unused_exports.length);

// Test 1: Without webpack tree-shaking
console.log('\n--- Test 1: WITHOUT webpack tree-shaking ---');
const result1 = optimize(chunkContent, JSON.stringify({
  lodash_usage: lodashUsage,
  enable_webpack_tree_shaking: false
}));
console.log('Result size:', result1.length, 'bytes');
console.log('Reduction:', chunkContent.length - result1.length, 'bytes');

// Count modules
const modules1 = (result1.match(/"\.\.\/\.\.\/node_modules\/\.pnpm\/lodash-es/g) || []).length;
console.log('Modules remaining:', modules1);

// Test 2: With webpack tree-shaking
console.log('\n--- Test 2: WITH webpack tree-shaking ---');
const result2 = optimize(chunkContent, JSON.stringify({
  lodash_usage: lodashUsage,
  enable_webpack_tree_shaking: true
}));
console.log('Result size:', result2.length, 'bytes');
console.log('Reduction:', chunkContent.length - result2.length, 'bytes');

// Count modules
const modules2 = (result2.match(/"\.\.\/\.\.\/node_modules\/\.pnpm\/lodash-es/g) || []).length;
console.log('Modules remaining:', modules2);

console.log('\n=== COMPARISON ===');
console.log('Additional reduction with webpack tree-shaking:', result1.length - result2.length, 'bytes');
console.log('Additional modules removed:', modules1 - modules2);

if (result1.length === result2.length) {
  console.log('\n❌ Webpack tree-shaking made NO difference - split chunk format may not be fully supported');
} else {
  console.log('\n✅ Webpack tree-shaking is working and removed additional dead code');
}