module.exports = [
	[
		/Module build failed: Error: Message /,
		// /Module build failed( \(from [^)]+\))?:\nMessage/,
		{ details: /Stack/ }
	]
];
