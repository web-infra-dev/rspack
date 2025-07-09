#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');

async function main() {
  console.log('=== TESTING PARSER WITH VERBOSE OUTPUT ===\n');

  // Test 1: Standard webpack format
  const standardBundle = `
var __webpack_modules__ = ({
  100: function(module, exports, __webpack_require__) {
    var dep1 = __webpack_require__(200);
    exports.foo = function() { return dep1(); };
  },
  200: function(module, exports) {
    exports.default = function() { return "reachable"; };
  },
  300: function(module, exports) {
    exports.default = function() { return "unreachable"; };
  }
});
__webpack_require__(100);
`;

  console.log('Test 1: Standard webpack format');
  console.log('Input size:', standardBundle.length);
  
  try {
    // Capture stdout and stderr by running in subprocess
    const { execSync } = require('child_process');
    const testFile = 'test-standard.js';
    fs.writeFileSync(testFile, standardBundle);
    
    const output = execSync(`node -e "
      const fs = require('fs');
      const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');
      const input = fs.readFileSync('${testFile}', 'utf-8');
      const result = optimize(input, JSON.stringify({ enable_webpack_tree_shaking: true }));
      console.log('RESULT_SIZE:' + result.length);
      console.log('CONTAINS_300:' + result.includes('300'));
    "`, { encoding: 'utf8', stdio: 'pipe' });
    
    console.log('Output:', output);
    
    fs.unlinkSync(testFile);
  } catch (err) {
    console.error('Error:', err.toString());
  }

  // Test 2: Split chunk format
  console.log('\n\nTest 2: Split chunk format (first 500 chars)');
  const chunkPath = path.join(__dirname, 'dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.js');
  const chunkContent = fs.readFileSync(chunkPath, 'utf-8');
  console.log(chunkContent.substring(0, 500));
  
  // Test optimization with debugging
  console.log('\n\nRunning optimization on split chunk...');
  const testFile2 = 'test-chunk.js';
  fs.writeFileSync(testFile2, chunkContent);
  
  try {
    const { execSync } = require('child_process');
    const output = execSync(`node -e "
      const fs = require('fs');
      const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');
      const input = fs.readFileSync('${testFile2}', 'utf-8');
      const config = {
        lodash_usage: {
          used_exports: ['debounce'],
          unused_exports: ['chunk'],
          possibly_unused_exports: []
        },
        enable_webpack_tree_shaking: true
      };
      console.log('Starting optimization...');
      const result = optimize(input, JSON.stringify(config));
      console.log('RESULT_SIZE:' + result.length);
      console.log('REDUCTION:' + (input.length - result.length));
      
      // Count modules
      const moduleMatches = result.match(/\"[^\"]+\"\\s*:\\s*(?:\\/\\*[!*][^*]*\\*+(?:[^/*][^*]*\\*+)*\\/\\s*)?(?:\\()?function/g) || [];
      console.log('MODULES_COUNT:' + moduleMatches.length);
    "`, { encoding: 'utf8', stdio: ['pipe', 'pipe', 'pipe'], maxBuffer: 10 * 1024 * 1024 });
    
    console.log('Output:', output);
    
  } catch (err) {
    console.error('Error:', err.toString());
    if (err.stderr) console.error('Stderr:', err.stderr.toString());
  } finally {
    if (fs.existsSync(testFile2)) fs.unlinkSync(testFile2);
  }
}

main().catch(console.error);