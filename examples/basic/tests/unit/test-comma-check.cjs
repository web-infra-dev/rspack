const fs = require('node:fs');
const path = require('node:path');

console.log('🔍 Checking comma positioning in generated file...');

const targetFile = path.join(__dirname, '../../dist/cjs-modules_module-exports-pattern_js.js');
const content = fs.readFileSync(targetFile, 'utf8');

// Look for correct pattern: /* @common:if [...] */ property, /* @common:endif */
const correctPattern = /\/\*\s*@common:if\s*\[condition="[^"]+"\]\s*\*\/\s*\w+,\s*\/\*\s*@common:endif\s*\*\//g;
const correctMatches = content.match(correctPattern);

// Look for incorrect pattern: /* @common:if [...] */ property /* @common:endif */,
const incorrectPattern = /\/\*\s*@common:if\s*\[condition="[^"]+"\]\s*\*\/\s*\w+\s*\/\*\s*@common:endif\s*\*\/\s*,/g;
const incorrectMatches = content.match(incorrectPattern);

console.log(`✅ Found ${correctMatches ? correctMatches.length : 0} correctly positioned commas`);
console.log(`❌ Found ${incorrectMatches ? incorrectMatches.length : 0} incorrectly positioned commas`);

if (correctMatches) {
  console.log('✅ Example correct format:', correctMatches[0]);
}

if (incorrectMatches) {
  console.log('❌ Example incorrect format:', incorrectMatches[0]);
  process.exit(1);
}

if (!correctMatches || correctMatches.length === 0) {
  console.log('❌ No correctly positioned commas found');
  process.exit(1);
}

console.log('🎉 All comma positioning tests passed!');