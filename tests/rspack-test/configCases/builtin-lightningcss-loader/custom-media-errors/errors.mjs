export default [
	{ moduleName: /(^|[\\/])cyclic\.css/, message: /Circular custom media query --a detected/ },
	// A circular reference is an error even when an undefined name is also used.
	{ moduleName: /missing-before-cyclic\.css/, message: /Circular custom media query --a detected/ },
	{ moduleName: /cyclic-before-missing\.css/, message: /Circular custom media query --a detected/ },
];
