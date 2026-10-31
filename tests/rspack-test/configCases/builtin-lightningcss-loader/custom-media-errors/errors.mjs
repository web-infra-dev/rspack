export default [
	// Custom media is resolved per module, so definitions in another module are not visible.
	{ moduleName: /component\.css/, message: /Custom media query --wide is not defined/ },
	{ moduleName: /cyclic\.css/, message: /Circular custom media query --a detected/ },
];
