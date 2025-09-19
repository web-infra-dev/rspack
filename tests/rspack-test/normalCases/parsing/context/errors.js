module.exports = [
	[
		/Module parse failed/,
		{ moduleName: /dump-file\.txt/ },
		// CHANGE: module identifier
		// { moduleTrace: /templates\/ sync/ }
		{ moduleTrace: /templates|sync/ }
	]
];
