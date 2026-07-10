const warnings = [
	/module\.createRequire supports only file URLs and absolute paths/,
	/module\.createRequire supports only file URLs and absolute paths/,
	/module\.createRequire supports only file URLs and absolute paths/,
	/module\.createRequire supports only file URLs and absolute paths/,
	/module\.createRequire supports only file URLs and absolute paths/,
	/module\.createRequire failed parsing argument/,
	/module\.createRequire failed parsing argument/,
	/module\.createRequire failed parsing argument/,
	/module\.createRequire failed parsing argument/,
	/module\.createRequire does not support spread arguments/,
	/the request of a dependency is an expression/
];

if (process.platform === "win32") {
	warnings.push(
		/module\.createRequire failed parsing argument/,
		/module\.createRequire failed parsing argument/
	);
}

module.exports = warnings;
