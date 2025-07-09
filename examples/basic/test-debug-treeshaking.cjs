#!/usr/bin/env node

const fs = require('fs');
const { execSync } = require('child_process');

// Create a simple split chunk for testing
const testChunk = `
"use strict";
(self["webpackChunktest"] = self["webpackChunktest"] || []).push([["test"], {
  "module1": function(module, __webpack_exports__, __webpack_require__) {
    __webpack_require__.d(__webpack_exports__, {
      foo: () => foo
    });
    const dep = __webpack_require__("module2");
    const foo = () => dep() + " from module1";
  },
  "module2": function(module, __webpack_exports__, __webpack_require__) {
    __webpack_require__.d(__webpack_exports__, {
      bar: () => bar
    });
    const bar = () => "module2";
  },
  "module3": function(module, __webpack_exports__, __webpack_require__) {
    __webpack_require__.d(__webpack_exports__, {
      baz: () => baz
    });
    const baz = () => "module3 - no dependencies";
  }
}]);
`;

fs.writeFileSync('test-chunk.js', testChunk);

console.log('Testing with simple split chunk...');
console.log('Input size:', testChunk.length);

// Run with stderr captured
try {
  const output = execSync(`node -e "
    const fs = require('fs');
    const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');
    const input = fs.readFileSync('test-chunk.js', 'utf-8');
    
    console.error('Running optimization...');
    const result = optimize(input, JSON.stringify({
      enable_webpack_tree_shaking: true
    }));
    
    console.log('RESULT_LENGTH:' + result.length);
    console.log('CONTAINS_MODULE3:' + result.includes('module3'));
    
    fs.writeFileSync('test-chunk-output.js', result);
  " 2>&1`, { encoding: 'utf8' });
  
  console.log('Output:', output);
} catch (err) {
  console.error('Error:', err.toString());
  if (err.stdout) console.log('Stdout:', err.stdout.toString());
  if (err.stderr) console.log('Stderr:', err.stderr.toString());
}

// Clean up
if (fs.existsSync('test-chunk.js')) fs.unlinkSync('test-chunk.js');