const { optimize } = require('./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js');
const fs = require('fs');

// Get the optimized result from our previous test
const testCode = fs.readFileSync('./test-lodash-simulation.js', 'utf8');
const emptyConfig = { treeShake: {} };

console.log('=== FIRST PASS ===');
const firstPass = optimize(testCode, JSON.stringify(emptyConfig));
console.log('First pass size:', firstPass.length);

// Count modules after first pass
const firstPassModules = (firstPass.match(/"[^"]+"/g) || []).map(m => m.replace(/"/g, '')).sort();
console.log('First pass modules:', firstPassModules.join(', '));

console.log('\n=== FIRST PASS RESULT ===');
console.log(firstPass);

console.log('\n=== SECOND PASS ===');
// Run the optimized result through SWC again
const secondPass = optimize(firstPass, JSON.stringify(emptyConfig));
console.log('Second pass size:', secondPass.length);
console.log('Additional reduction:', firstPass.length - secondPass.length, 'bytes');

// Count modules after second pass  
const secondPassModules = (secondPass.match(/"[^"]+"/g) || []).map(m => m.replace(/"/g, '')).sort();
console.log('Second pass modules:', secondPassModules.join(', '));

const removedInSecondPass = firstPassModules.filter(m => !secondPassModules.includes(m));
if (removedInSecondPass.length > 0) {
    console.log('Modules removed in second pass:', removedInSecondPass.join(', '));
} else {
    console.log('No additional modules removed in second pass');
}

console.log('\n=== SECOND PASS RESULT ===');
console.log(secondPass);

// Test a third pass for completeness
console.log('\n=== THIRD PASS ===');
const thirdPass = optimize(secondPass, JSON.stringify(emptyConfig));
console.log('Third pass size:', thirdPass.length);
console.log('Additional reduction from third pass:', secondPass.length - thirdPass.length, 'bytes');

if (thirdPass === secondPass) {
    console.log('✅ Convergence reached: Third pass identical to second pass');
} else {
    console.log('⚠️  Third pass still making changes');
}