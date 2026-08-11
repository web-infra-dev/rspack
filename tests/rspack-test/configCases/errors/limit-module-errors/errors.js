module.exports = [
	...Array.from({ length: 198 }, () => ({
		message: /emitted error/
	})),
	...Array.from({ length: 2 }, () => ({
		code: /ModuleErrorsLimit/,
		message: /Only the first 99 errors are shown/
	}))
];
