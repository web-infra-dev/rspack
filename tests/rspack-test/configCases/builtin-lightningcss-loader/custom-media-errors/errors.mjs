export default [
	// Definitions are only visible through unconditional `@import`s.
	{ moduleName: /component\.css/, message: /Custom media query --wide is not defined/ },
	{ moduleName: /conditional\.css/, message: /Custom media query --wide is not defined/ },
	{ moduleName: /cyclic\.css/, message: /Circular custom media query --a detected/ },
];
