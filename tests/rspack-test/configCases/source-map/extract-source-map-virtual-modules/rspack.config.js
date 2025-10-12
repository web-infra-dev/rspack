const {
	experiments: { VirtualModulesPlugin }
} = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	entry: {
		main: "./index.js"
	},
	devtool: "source-map",
	module: {
		rules: [
			{
				extractSourceMap: true
			}
		]
	},
	plugins: [
		new VirtualModulesPlugin({
			// Regular virtual modules
			"virtual-module-with-sourcemap.js": `const a = 1;
//    @    sourceMappingURL    =    data:application/source-map;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidmlydHVhbC1tb2R1bGUtd2l0aC1zb3VyY2VtYXAuanMiLCJzb3VyY2VzIjpbInZpcnR1YWwtdGVzdC50eHQiXSwic291cmNlc0NvbnRlbnQiOlsidmlydHVhbCBtb2R1bGUgd2l0aCBzb3VyY2VtYXAiXSwibWFwcGluZ3MiOiJBQUFBIn0=
// comment`,
			"virtual-module-without-sourcemap.js": `const b = 2;
// regular comment without source map`,
			"virtual-test.txt": "virtual module content",

			// Virtual modules in subdirectories
			"lib/components/virtual-component.js": `export const Component = "virtual component";
// @sourceMappingURL=virtual-component.js.map`,
			"lib/components/virtual-component.js.map": `{"version":3,"file":"lib/components/virtual-component.js","sources":["../../src/components/virtual-component.js"],"sourcesContent":["export const Component: string = 'virtual component'"],"mappings":"AAAA"}`
		})
	]
};