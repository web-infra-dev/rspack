module.exports = [
	// SWC Next recovers from the first malformed expression statement and then
	// reports the independent malformed `with` statement on the next code line.
	[
		/Expected a semicolon or an implicit semicolon after a statement, but found 'is'/,
		{ moduleName: /dump-file\.txt/ },
		{ moduleTrace: /templates|sync/ }
	],
	[
		/Expected '\(' after 'with', but found 'some'/,
		{ moduleName: /dump-file\.txt/ },
		{ moduleTrace: /templates|sync/ }
	]
];
