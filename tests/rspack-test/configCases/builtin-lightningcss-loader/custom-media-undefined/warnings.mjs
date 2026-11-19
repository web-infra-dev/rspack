export default [
	// Definitions in another module are not visible.
	{
		moduleName: /component\.css/,
		code: /^builtin:lightningcss-loader$/,
		message: /⚠ builtin:lightningcss-loader: Custom media query --wide is not defined in this file\. Skipping custom media expansion\./,
	},
];
