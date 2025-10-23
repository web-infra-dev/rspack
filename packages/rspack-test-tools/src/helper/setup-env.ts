// @ts-nocheck
const path = require("node:path");

// Setup environment variable for binding testing
if (process.env.RSPACK_BINDING_BUILDER_TESTING) {
	process.env.RSPACK_BINDING = path.resolve(
		__dirname,
		"../../node_modules/@rspack/binding-testing"
	);
}

if (process.env.RSTEST) {
	global.printLogger ??= process.env.printLogger === "true";
	global.__TEST_FIXTURES_PATH__ ??= process.env.__TEST_FIXTURES_PATH__;
	global.updateSnapshot ??= process.env.updateSnapshot === "true";
	global.testFilter ??= process.env.testFilter;
	global.__TEST_PATH__ ??= process.env.__TEST_PATH__;
	global.__TEST_DIST_PATH__ ??= process.env.__TEST_DIST_PATH__;
	global.__ROOT_PATH__ ??= process.env.__ROOT_PATH__;
	global.__RSPACK_PATH__ ??= process.env.__RSPACK_PATH__;
	global.__RSPACK_TEST_TOOLS_PATH__ ??= process.env.__RSPACK_TEST_TOOLS_PATH__;
	global.__DEBUG__ ??= process.env.DEBUG === "test";
}

if (process.env.ALTERNATIVE_SORT) {
	const oldSort = Array.prototype.sort;

	Array.prototype.sort = function (cmp) {
		oldSort.call(this, cmp);
		if (cmp) {
			for (let i = 1; i < this.length; i++) {
				if (cmp(this[i - 1], this[i]) === 0) {
					let j = i + 1;
					for (; j < this.length; j++) {
						if (cmp(this[j - 1], this[j]) !== 0) {
							break;
						}
					}
					for (let x = i - 1, y = j - 1; x < y; x++, y--) {
						const temp = this[x];
						this[x] = this[y];
						this[y] = temp;
					}
					i = j;
				}
			}
		}
		return this;
	};
}

// Setup debugging info for tests
if (process.env.DEBUG_INFO) {
	const addDebugInfo = it => {
		return (name, fn, timeout) => {
			if (fn.length === 0) {
				it(
					name,
					() => {
						process.stdout.write(`START1 ${name}\n`);
						try {
							const promise = fn();
							if (promise?.then) {
								return promise.then(
									r => {
										process.stdout.write(`DONE OK ${name}\n`);
										return r;
									},
									e => {
										process.stdout.write(`DONE FAIL ${name}\n`);
										throw e;
									}
								);
							}
							process.stdout.write(`DONE OK ${name}\n`);
						} catch (e) {
							process.stdout.write(`DONE FAIL ${name}\n`);
							throw e;
						}
					},
					timeout
				);
			} else {
				it(
					name,
					() =>
						new Promise((resolve, reject) => {
							const done = err => (err ? reject(err) : resolve());
							process.stdout.write(`START2 ${name}\n`);
							return fn(err => {
								if (err) {
									process.stdout.write(`DONE FAIL ${name}\n`);
								} else {
									process.stdout.write(`DONE OK ${name}\n`);
								}
								return done(err);
							});
						}),
					timeout
				);
			}
		};
	};
	// eslint-disable-next-line no-global-assign
	it = addDebugInfo(it);
}

// cspell:word wabt
// Workaround for a memory leak in wabt
// It leaks an Error object on construction
// so it leaks the whole stack trace
require("wast-loader");
process.removeAllListeners("uncaughtException");
process.removeAllListeners("unhandledRejection");
