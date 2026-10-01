const expected = (file, received) => ({
	moduleName: new RegExp(`\\./${file.replace(".", "\\.")}`),
	message: new RegExp(
		'`webpackMode` expected "lazy", "lazy-once", "eager" or "weak", but received: ' +
			received
	)
});

export default [
	expected("lazy.js", '"invalid"'),
	expected("lazy-once.js", '"sync"'),
	expected("eager.js", '"async-weak"'),
	expected("weak.js", "true")
];
