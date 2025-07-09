#!/usr/bin/env node

const fs = require('fs');
const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');

// Test 1: Standard webpack format with clear unreachable module
const webpackBundle = `
var __webpack_modules__ = {
  100: function(module, exports, __webpack_require__) {
    console.log("Module 100 - entry point");
    __webpack_require__(200);
  },
  200: function(module, exports, __webpack_require__) {
    console.log("Module 200 - reachable");
  },
  300: function(module, exports, __webpack_require__) {
    console.log("Module 300 - UNREACHABLE!");
  }
};

// Entry point
__webpack_require__(100);
`;

console.log('=== TEST 1: Standard Webpack Bundle ===');
console.log('Input:', webpackBundle.length, 'bytes');
console.log('Contains module 300 (unreachable):', webpackBundle.includes('Module 300'));

const result1 = optimize(webpackBundle, JSON.stringify({
  enable_webpack_tree_shaking: true
}));

console.log('\nResult:', result1.length, 'bytes');
console.log('Contains module 300 after optimization:', result1.includes('Module 300'));
console.log('Module 300 removed:', !result1.includes('Module 300') ? '✅ YES' : '❌ NO');

// Test 2: Split chunk format
const splitChunk = `
"use strict";
(self["webpackChunktest"] = self["webpackChunktest"] || []).push([["chunk"], {
  "module1": function(module, exports, __webpack_require__) {
    console.log("Module 1");
    __webpack_require__("module2");
  },
  "module2": function(module, exports, __webpack_require__) {
    console.log("Module 2");
  },
  "module3": function(module, exports, __webpack_require__) {
    console.log("Module 3 - no dependencies on it");
  }
}]);
`;

console.log('\n\n=== TEST 2: Split Chunk Format ===');
console.log('Input:', splitChunk.length, 'bytes');
console.log('Contains module3:', splitChunk.includes('Module 3'));

const result2 = optimize(splitChunk, JSON.stringify({
  enable_webpack_tree_shaking: true
}));

console.log('\nResult:', result2.length, 'bytes');
console.log('Contains module3 after optimization:', result2.includes('Module 3'));

// Write results for inspection
fs.writeFileSync('test-webpack-result.js', result1);
fs.writeFileSync('test-chunk-result.js', result2);
console.log('\n\nResults written to test-webpack-result.js and test-chunk-result.js');