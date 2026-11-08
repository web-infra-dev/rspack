export default [
	// Definitions in another module are not visible.
	{
		moduleName: /component\.css/,
		message: /Custom media query --wide is not defined, so custom media in this file is left unchanged/,
	},
];
