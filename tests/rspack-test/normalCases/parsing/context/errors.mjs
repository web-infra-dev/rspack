export default [
	[
		/Expected a semicolon or an implicit semicolon after a statement, but found 'is'/,
		{ moduleName: /dump-file\.txt/ },
		{ moduleTrace: /templates|sync/ }
	]
];
