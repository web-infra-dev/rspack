// module.exports = [
// 	[/Missing whitespace after ':global' in ':global\.class \{/],
// 	[
// 		/Missing whitespace after ':global' in ':global\/\*\* test \*\*\/\.class \{/
// 	],
// 	[/Missing whitespace after ':local' in ':local\.class \{'/],
// 	[/Missing whitespace after ':local' in ':local\/\*\* test \*\*\/\.class \{'/],
// 	[/Missing whitespace after ':local' in ':local\/\*\* test \*\*\/#hash \{'/],
// 	[/Missing whitespace after ':local' in ':local\/\*\* test \*\*\/\{/]
// ];


module.exports = [
	[/Missing trailing whitespace/, /:global\.class/],
	[/Missing trailing whitespace/, /:global\/\*\* test \*\*\/\.class/],
	[/Missing trailing whitespace/, /:local\.class/],
	[/Missing trailing whitespace/, /:local\/\*\* test \*\*\/\.class/],
	[/Missing trailing whitespace/, /:local\/\*\* test \*\*\/#hash/],
];
