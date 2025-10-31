const addEntryMockFn = rstest.fn();
const succeedEntryMockFn = rstest.fn();
const failedEntryMockFn = rstest.fn();

class EntryHooksTestPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("EntryHooksTestPlugin", compilation => {
			compilation.hooks.addEntry.tap("EntryHooksTestPlugin", (entry, options) => {
				addEntryMockFn({ entry, options });
			});

			compilation.hooks.succeedEntry.tap("EntryHooksTestPlugin", (entry, options, module) => {
				succeedEntryMockFn({ entry, options, module });
			});

			compilation.hooks.failedEntry.tap("EntryHooksTestPlugin", (entry, options, error) => {
				failedEntryMockFn({ entry, options, error });
			});
		});
	}
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should call entry hooks at correct timing",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				main: "./d"
			},
			plugins: [new EntryHooksTestPlugin()]
		};
	},
	async build(_, compiler) {
		await new Promise((resolve, reject) => {
			compiler.run((err) => {
				if (err) reject(err);
				else resolve();
			});
		});
	},
	async check() {
		// Check that addEntry hook was called
		expect(addEntryMockFn).toBeCalled();
		const addEntryCalls = addEntryMockFn.mock.calls;
		expect(addEntryCalls.length).toBeGreaterThan(0);

		// Check that entry and options are passed correctly
		const firstCall = addEntryCalls[0][0];
		expect(firstCall).toHaveProperty('entry');
		expect(firstCall).toHaveProperty('options');
		expect(firstCall.options).toHaveProperty('name');

		// Check that succeedEntry hook was called for successful entries
		expect(succeedEntryMockFn).toBeCalled();
		const succeedCalls = succeedEntryMockFn.mock.calls;
		expect(succeedCalls.length).toBeGreaterThan(0);

		// Each successful entry should have entry, options, and module
		const firstSucceedCall = succeedCalls[0][0];
		expect(firstSucceedCall).toHaveProperty('entry');
		expect(firstSucceedCall).toHaveProperty('options');
		expect(firstSucceedCall).toHaveProperty('module');

		// failedEntry hook should not be called if all entries are successful
		expect(failedEntryMockFn).not.toBeCalled();
	}
};