const path = require('path');

const mockWatchRunFn = rstest.fn(() => { });
const mockInvalidFn = rstest.fn(() => { });

class MyPlugin {
	apply(compiler) {
		compiler.hooks.watchRun.tap("MyPlugin", mockWatchRunFn)
		compiler.hooks.invalid.tap("MyPlugin", mockInvalidFn)
	}
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should be invalidated correctly",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./abc",
			plugins: [new MyPlugin()]
		};
	},
	async build(_, compiler) {
		try {
			await new Promise((resolve, reject) => {
				let firstRun = true;
				compiler.watch({}, (err) => {
					if (err) {
						return reject(err);
					}
					if (firstRun) {
						firstRun = false;
						compiler.watching.invalidateWithChangesAndRemovals(new Set([path.resolve(__dirname, "../fixtures/a.js")]));
						compiler.watching.invalidateWithChangesAndRemovals(new Set([path.resolve(__dirname, "../fixtures/b.js")]));
						setTimeout(() => {
							resolve();
						}, 2000)
					}
				});
			});
		} catch (err) {
			throw err
		}
	},
	async check() {
		expect(mockWatchRunFn).toHaveBeenCalledTimes(3);
		expect(mockInvalidFn).toHaveBeenCalledTimes(2);
	}
};
