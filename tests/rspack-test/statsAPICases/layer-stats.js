/** @type {import('@rspack/test-tools').TStatsAPICaseConfig} */
module.exports = {
	description: "should have module layer",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				main: {
					import: "./fixtures/abc",
					layer: "test"
				},
				legacy: {
					import: "./fixtures/abc",
					layer: "legacy"
				}
			},
		};
	},
	async check(stats) {
		const options = {
			all: false,
			modules: true,
			groupModulesByLayer: true
		};
		const statsData = stats?.toJson(options);
		statsData.modules.forEach(mod => {
			mod.issuer = "";
			mod.issuerName = "";
			mod.children = [];
		});

		expect(statsData).toMatchInlineSnapshot(`
		Object {
		  filteredModules: undefined,
		  modules: Array [
		    Object {
		      children: Array [],
		      issuer: ,
		      issuerName: ,
		      layer: test,
		      size: 304,
		      sizes: Object {
		        javascript: 304,
		      },
		      type: modules by layer,
		    },
		    Object {
		      children: Array [],
		      issuer: ,
		      issuerName: ,
		      layer: legacy,
		      size: 304,
		      sizes: Object {
		        javascript: 304,
		      },
		      type: modules by layer,
		    },
		  ],
		}
	`);
	}
};
