const path = require('path');

const mockFn = jest.fn(() => {});

class MyPlugin {
	apply(compiler) {
		compiler.hooks.watchRun.tap("MyPlugin", mockFn)
	}
}

/** @type {import('../../dist').TCompilerCaseConfig} */
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
						compiler.watching.lazyCompilationInvalidate(path.resolve(__dirname, "../fixtures/a.js"));
						compiler.watching.lazyCompilationInvalidate(path.resolve(__dirname, "../fixtures/b.js"));
						setTimeout(() => {
							resolve()
						}, 2000)
					}
				});
			});
		} catch(err) {
			throw err
		}

	},
	async check() {
		expect(mockFn).toHaveBeenCalledTimes(3);
	}
};
