const addEntryMockFn = rstest.fn();
const succeedEntryMockFn = rstest.fn();
const failedEntryMockFn = rstest.fn();

class EntryHooksErrorTestPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("EntryHooksErrorTestPlugin", compilation => {
			compilation.hooks.addEntry.tap("EntryHooksErrorTestPlugin", (entry, options) => {
				addEntryMockFn({ entry, options });
			});

			compilation.hooks.succeedEntry.tap("EntryHooksErrorTestPlugin", (entry, options, module) => {
				succeedEntryMockFn({ entry, options, module });
			});

			compilation.hooks.failedEntry.tap("EntryHooksErrorTestPlugin", (entry, options, error) => {
				failedEntryMockFn({ entry, options, error });
			});
		});
	}
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should call failedEntry hook when entry fails",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				main: "./non-existent-file.js"
			},
			plugins: [new EntryHooksErrorTestPlugin()]
		};
	},
	async build(_, compiler) {
		await new Promise((resolve) => {
			compiler.run((err, stats) => {
				// We expect errors but continue to test hooks
				resolve();
			});
		});
	},
	async check() {
		// Check that addEntry hook was called even for failed entries
		expect(addEntryMockFn).toBeCalled();
		const addEntryCalls = addEntryMockFn.mock.calls;
		expect(addEntryCalls.length).toBeGreaterThan(0);

		// Check that failedEntry hook was called for non-existent file
		expect(failedEntryMockFn).toBeCalled();
		const failedCalls = failedEntryMockFn.mock.calls;
		expect(failedCalls.length).toBeGreaterThan(0);

		// Failed entry should have entry, options, and error
		const firstFailedCall = failedCalls[0][0];
		expect(firstFailedCall).toHaveProperty('entry');
		expect(firstFailedCall).toHaveProperty('options');
		expect(firstFailedCall).toHaveProperty('error');
		expect(firstFailedCall.error).toBeDefined();
	}
};