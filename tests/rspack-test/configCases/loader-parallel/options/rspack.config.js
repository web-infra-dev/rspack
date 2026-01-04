/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "none",
	optimization: {
		moduleIds: "named"
	},
	module: {
		rules: [
			{
				test: /a\.js$/,
				use: [
					{
						loader: "./loader",
						parallel: { maxWorkers: 4 },
						options: {
							arg: true,
							arg1: null,
							arg2: undefined,
							arg3: 1234567890,
							arg4: "string",
							arg5: [1, 2, 3],
							arg6: { foo: "value", bar: { baz: "other-value" } }
						}
					}
				]
			},
			{
				test: /b\.js$/,
				use: [
					{
						loader: "./loader-1",
						parallel: { maxWorkers: 4 },
						options: {
							arg: true,
							arg1: null,
							arg2: undefined,
							arg3: 1234567890,
							arg4: "string",
							arg5: [1, 2, 3],
							arg6: { foo: "value", bar: { baz: "other-value" } }
						}
					}
				]
			},
			{
				test: /c\.js$/,
				loader: "./loader-1",
				// This does not support by `parllel`, `Rule.use.options` must be an
				// object.
				options: JSON.stringify({
					arg: true,
					arg1: null,
					arg2: undefined,
					arg3: 1234567890,
					arg4: "string",
					arg5: [1, 2, 3],
					arg6: { foo: "value", bar: { baz: "other-value" } }
				})
			},
			{
				test: /d\.js$/,
				loader: "./loader-1",
				options: "arg4=text"
			},
			{
				test: /d\.js$/,
				loader: "./loader",
				options: ""
			},
			{
				test: /f\.js$/,
				loader: "./loader",
				options: "name=cheesecake&slices=8&delicious&warm=false"
			},
			{
				test: /g\.js$/,
				loader: "./loader",
				options: "%3d=%3D"
			},
			{
				test: /h\.js$/,
				loader: "./loader",
				options: "foo=bar"
			},
			{
				test: /i\.js$/,
				loader: "./loader",
				options: `${JSON.stringify({
					foo: "bar"
				})}`
			},
			{
				test: /error1\.js$/,
				use: [
					{
						loader: "./loader-1",
						parallel: { maxWorkers: 4 },
						options: {
							arg6: { foo: "value", bar: { baz: 42 } }
						}
					}
				]
			},
			{
				test: /error2\.js$/,
				use: [
					{
						loader: "./loader-2",
						parallel: { maxWorkers: 4 },
						options: {
							arg: false
						}
					}
				]
			}
		]
	},
	experiments: {
		parallelLoader: true
	}
};
