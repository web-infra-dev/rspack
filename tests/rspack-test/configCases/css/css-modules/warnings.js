"use strict";

const missingExportWarnings = Array.from({ length: 4 }).flatMap(() => [
	/export \x27nested2\x27 \(imported as \x27style\x27\) was not found/,
	/export \x27global-color\x27 \(imported as \x27style\x27\) was not found/,
	/export \x27GLOBAL-COLOR\x27 \(imported as \x27style\x27\) was not found/
]);

const parseWarnings = Array.from({ length: 7 }).flatMap(() => [
	/Broken \x27@value\x27 at-rule/,
	/Broken \x27@value\x27 at-rule/,
	/Missing trailing whitespace[\s\S]*:global\.class-no-space/,
	/Missing trailing whitespace[\s\S]*:global\/\*\* test \*\*\/\.class/,
	/Missing trailing whitespace[\s\S]*:local\.class/,
	/Missing trailing whitespace[\s\S]*:local\/\*\* test \*\*\/\.class/,
	/Missing trailing whitespace[\s\S]*:local\/\*\* test \*\*\/#hash/,
	/Missing trailing whitespace[\s\S]*:local\/\*\* test \*\*\/\{/
]);

module.exports = [...missingExportWarnings, ...parseWarnings];
