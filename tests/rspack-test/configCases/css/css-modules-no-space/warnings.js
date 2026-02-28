"use strict";

module.exports = [
	[/Missing trailing whitespace/, /:global\.class/],
	[
		/Missing trailing whitespace/, /:global\/\*\* test \*\*\/\.class/
	],
	[/Missing trailing whitespace/, /:local\.class/],
	[/Missing trailing whitespace/, /:local\/\*\* test \*\*\/\.class/],
	[/Missing trailing whitespace/, /:local\/\*\* test \*\*\/#hash/],
];
