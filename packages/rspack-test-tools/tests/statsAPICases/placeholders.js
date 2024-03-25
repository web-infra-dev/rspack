let stats;

class TestPlugin {
	apply(compiler) {
		compiler.hooks.thisCompilation.tap("custom", compilation => {
			compilation.hooks.optimizeModules.tap("test plugin", () => {
				stats = compiler.compilation.getStats().toJson({});
			});
		});
	}
}

module.exports = {
	description: "should have null as placeholders in stats before chunkIds",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/a",
			plugins: [new TestPlugin()]
		};
	},
	async check() {
		expect(stats.entrypoints).toMatchInlineSnapshot(`
      Object {
        "main": Object {
          "assets": Array [],
          "assetsSize": 0,
          "chunks": Array [
            null,
          ],
          "name": "main",
        },
      }
    `);
	}
};
