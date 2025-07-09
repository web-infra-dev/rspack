#!/usr/bin/env node

const fs = require('fs');
const { execSync } = require('child_process');

// Simple split chunk
const splitChunk = `
(self["webpackChunk"] = self["webpackChunk"] || []).push([["test"], {
  "mod1": function(module, exports, __webpack_require__) {
    __webpack_require__("mod2");
  },
  "mod2": function(module, exports) {
    exports.foo = "bar";
  },
  "mod3": function(module, exports) {
    exports.unused = "should be removed";
  }
}]);
`;

fs.writeFileSync('test-split.js', splitChunk);

console.log('=== TESTING SPLIT CHUNK TREE SHAKING WITH DEBUG ===');
console.log('Input:', splitChunk.length, 'bytes');

try {
  // Run optimization with stderr captured
  const output = execSync(`
    RUST_LOG=debug node -e "
      const fs = require('fs');
      const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');
      const input = fs.readFileSync('test-split.js', 'utf-8');
      const result = optimize(input, JSON.stringify({ enable_webpack_tree_shaking: true }));
      console.log('RESULT_SIZE:' + result.length);
      fs.writeFileSync('test-split-output.js', result);
    " 2>&1
  `, { encoding: 'utf8' });
  
  console.log('\nOutput (with debug):', output);
  
  // Check the result
  if (fs.existsSync('test-split-output.js')) {
    const result = fs.readFileSync('test-split-output.js', 'utf-8');
    console.log('\n=== RESULT ANALYSIS ===');
    console.log('Size:', result.length, 'bytes');
    console.log('Contains mod3:', result.includes('mod3'));
    console.log('\nFirst 500 chars:\n', result.substring(0, 500));
  }
  
} catch (err) {
  console.error('Error:', err.toString());
}

// Cleanup
if (fs.existsSync('test-split.js')) fs.unlinkSync('test-split.js');
if (fs.existsSync('test-split-output.js')) fs.unlinkSync('test-split-output.js');