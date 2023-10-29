// expect 2 warnings
// the sequence of warnings is not guaranteed
module.exports = [
	[
		/There are multiple modules with names that only differ in casing/,
		/(case-sensitive.A\.js)|(case-sensitive.B.file\.js)/,
		/(case-sensitive.a\.js)|(case-sensitive.b.file\.js)/
	],
	[
		/There are multiple modules with names that only differ in casing/,
		/(case-sensitive.A\.js)|(case-sensitive.B.file\.js)/,
		/(case-sensitive.a\.js)|(case-sensitive.b.file\.js)/
	]
];
