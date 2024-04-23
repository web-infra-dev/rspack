// @ts-nocheck
// Remove the "|" padding from miette,
// used to ensure no line breaks with padding being returned,
// miette generates diagnostics lines with respect to terminal size
// and this might varies among different `process.cwd()` being used,
// which breaks local and CI checks.
const replace = s => s.replace(/\r?\n[ ]+│ /g, "");

// HOW THIS WORKS:
// 1. Remove potential line break and "|"
// 2. Save the JS stack for each line
// 3. If the current line was splitted because of terminal size, merge them together
const replaceStack = s => s.replace(/(?:\s|│)*(at.*)(\s|│)*/g, "\n$1");

module.exports = {
	replace,
	replaceStack
};
