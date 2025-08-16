const fs = require('node:fs');

console.log('=== DEBUGGING WEBPACK BUNDLE PARSING ===');

// Read the original lodash chunk
const originalCode = fs.readFileSync('./dist/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.js', 'utf8');

console.log('Original file size:', originalCode.length);
console.log('First 500 chars:');
console.log('===');
console.log(originalCode.substring(0, 500));
console.log('===');

// Look for webpack module structure patterns
console.log('\n=== ANALYZING WEBPACK STRUCTURE ===');

// Check for webpack_modules declaration
const webpackModulesMatch = originalCode.match(/var\s+__webpack_modules__\s*=\s*\(/);
if (webpackModulesMatch) {
    console.log('✅ Found __webpack_modules__ declaration:', webpackModulesMatch[0]);
} else {
    console.log('❌ No __webpack_modules__ declaration found');
    
    // Look for alternative patterns
    const altPatterns = [
        /\[\s*"vendors-[^"]+"\s*\],\s*\{/,
        /"[^"]+"\s*:\s*function/,
        /__webpack_require__/
    ];
    
    altPatterns.forEach((pattern, i) => {
        const matches = originalCode.match(pattern);
        if (matches) {
            console.log(`Pattern ${i+1} match:`, matches[0]);
        }
    });
}

// Try to extract the first few module definitions
const modulePattern = /"([^"]+)"\s*:\s*function\s*\([^)]*\)\s*\{/g;
const modules = [];
let match;
let count = 0;

while ((match = modulePattern.exec(originalCode)) !== null && count < 10) {
    modules.push(match[1]);
    count++;
}

console.log('\nFirst 10 module IDs found:', modules);

// Look for entry point calls
const entryPattern = /__webpack_require__\s*\(\s*\/\*[^*]*\*\/\s*"([^"]+)"/g;
const entryPoints = [];
count = 0;

while ((match = entryPattern.exec(originalCode)) !== null && count < 5) {
    entryPoints.push(match[1]);
    count++;
}

console.log('Entry points found:', entryPoints);

// Check the webpack bundle format this parser expects
console.log('\n=== EXPECTED WEBPACK FORMAT ===');
console.log('The webpack_graph parser expects format like:');
console.log('var __webpack_modules__ = ({');
console.log('  100: function(module, exports, __webpack_require__) { ... },');  
console.log('  200: function(module, exports, __webpack_require__) { ... }');
console.log('});');
console.log('__webpack_require__(100);');

console.log('\n=== ACTUAL FORMAT DETECTED ===');
// Show the actual structure pattern
const structureMatch = originalCode.match(/\(self\["webpackChunk[^"]+"\][^{]+\{([^}]+)/);
if (structureMatch) {
    console.log('Detected rspack chunk format:');
    console.log(`${structureMatch[0]}...`);
    console.log('\nThis is an rspack chunk, not a standard webpack bundle!');
    console.log('Our tree shaker expects standard webpack format, not rspack chunks.');
} else {
    console.log('Unknown format detected');
}

// Test if we can find the webpack modules object within the chunk
const innerWebpackMatch = originalCode.match(/\{\s*"[^"]+"\s*:\s*function/);
if (innerWebpackMatch) {
    console.log('\n✅ Found webpack modules object within rspack chunk');
    console.log('Location:', originalCode.indexOf(innerWebpackMatch[0]));
    
    // Try to extract just the modules object
    const startIdx = originalCode.indexOf(innerWebpackMatch[0]);
    const moduleSection = originalCode.substring(startIdx, startIdx + 1000);
    console.log('Modules section preview:');
    console.log(`${moduleSection}...`);
} else {
    console.log('\n❌ No webpack modules object found within rspack chunk');
}

console.log('\n=== CONCLUSION ===');
console.log('The issue is likely that our webpack_graph parser expects');
console.log('standard webpack bundle format, but rspack generates a different');
console.log('chunk format that wraps the webpack modules.');
console.log('');
console.log('To fix this, we need to either:');
console.log('1. Extract the webpack modules from within the rspack chunk');
console.log('2. Enhance webpack_graph to understand rspack chunk format');
console.log('3. Apply tree-shaking at the rspack chunk level instead');