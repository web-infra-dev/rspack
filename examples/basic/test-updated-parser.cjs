#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');

async function main() {
  // Use optimize function directly, it handles WASM loading internally
  
  console.log('=== TESTING UPDATED PARSER WITH SPLIT CHUNK ===');

// Test with the lodash split chunk
const chunkPath = path.join(__dirname, 'dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.js');
const chunkContent = fs.readFileSync(chunkPath, 'utf-8');

console.log('\nOriginal chunk size:', chunkContent.length);

// First, let's test with a simple tree-shake config to see if parser works
const testConfig = {
  used_exports: ['debounce', 'throttle'],
  unused_exports: ['chunk', 'compact'],
  possibly_unused_exports: []
};

console.log('\nTest config:', testConfig);

try {
  console.log('\n=== RUNNING OPTIMIZATION ===');
  const result = optimize(chunkContent, JSON.stringify({
    lodash_usage: testConfig,
    enable_webpack_tree_shaking: true
  }));
  
  console.log('Optimization completed successfully!');
  console.log('Result size:', result.length);
  console.log('Size reduction:', chunkContent.length - result.length, 'bytes');
  
  // Check if any modules were detected
  const moduleCount = (result.match(/"[^"]+"\s*:\s*(?:\/\*[!*][^*]*\*+(?:[^/*][^*]*\*+)*\/\s*)?(?:\()?function/g) || []).length;
  console.log('Modules remaining:', moduleCount);
  
  // Write result for inspection
  fs.writeFileSync('test-parser-result.js', result);
  console.log('\nResult written to test-parser-result.js');
  
} catch (err) {
  console.error('\nError during optimization:', err);
  console.error('Stack:', err.stack);
}

// Also test with standard webpack format if we have one
console.log('\n\n=== TESTING WITH STANDARD WEBPACK FORMAT ===');

// Create a mock standard webpack bundle
const mockWebpackBundle = `
var __webpack_modules__ = ({
  100: function(module, exports, __webpack_require__) {
    var dep1 = __webpack_require__(200);
    var dep2 = __webpack_require__(300);
    exports.foo = function() { return dep1() + dep2(); };
  },
  200: function(module, exports, __webpack_require__) {
    exports.default = function() { return "module 200"; };
  },
  300: function(module, exports, __webpack_require__) {
    exports.default = function() { return "module 300"; };
  },
  400: function(module, exports, __webpack_require__) {
    // Unreachable module
    exports.default = function() { return "module 400"; };
  }
});

// Entry point
__webpack_require__(100);
`;

console.log('Mock webpack bundle size:', mockWebpackBundle.length);

try {
  const result = optimize(mockWebpackBundle, JSON.stringify({
    enable_webpack_tree_shaking: true
  }));
  
  console.log('Standard format optimization completed!');
  console.log('Result size:', result.length);
  console.log('Size reduction:', mockWebpackBundle.length - result.length, 'bytes');
  
  // Check if unreachable module was removed
  if (result.includes('module 400')) {
    console.log('❌ Unreachable module 400 was NOT removed');
  } else {
    console.log('✅ Unreachable module 400 was removed');
  }
  
} catch (err) {
  console.error('\nError with standard format:', err);
}
}

main().catch(console.error);