const path = require("path");
let plugin = path.join(__dirname,'my_first_plugin.wasm')
/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: {
		main: "./src/index.jsx"
	},
	module: {
		rules: [{
			test: /\.jsx$/,

			use: {
				loader: "builtin:swc-loader",
				options: {
					// Enable source map
					sourceMap: true,
					jsc: {
						parser: {
							syntax: "ecmascript",
							jsx: true,
						},
						experimental:{
							plugins:[
								[plugin,{}]
							]
						},
						transform: {
							react: {
								pragma: "React.createElement",
								pragmaFrag: "React.Fragment",
								throwIfNamespace: true,
								development: false,
								useBuiltins: false,
							},
						},
					},
				},
			},
			type: 'jsx',
		},
			{
				test: /\.(png|svg|jpg)$/,
				type: "asset/resource"
			}
		]
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	}
};
module.exports = config;
