// stack-trace-reporting.test.js
// Node.js native test for stack trace reporting in next build
const test = require("node:test");
const assert = require("node:assert/strict");
const { exec } = require("node:child_process");
const { join } = require("node:path");

const appDir = join(__dirname);

// Only run this test if not using Turbopack
if (!process.env.TURBOPACK) {
	test("Reports stack trace when webpack plugin stack overflows", (t, done) => {
		exec("npx next build", { cwd: appDir }, (error, stdout, stderr) => {
			// We expect the build to fail with a stack overflow
			assert.equal(error?.code, 1, "Expected exit code 1");
			assert.match(
				error.message,
				/caused by plugins in Compilation\.hooks\.processAssets/
			);
			assert.match(stderr, /Maximum call stack size exceeded/);
			assert.match(stderr, /next\.config\.js:7/);
			done();
		});
	});
}
