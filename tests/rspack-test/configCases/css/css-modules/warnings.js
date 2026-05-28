"use strict";

const repeat = (count, warning) =>
	Array.from({ length: count }, () => [warning]);

module.exports = [
	[/Broken '@value' at-rule/],
	...repeat(7, /Inconsistent rule global\/local/),
	...repeat(2, /A ':global\(' is not allowed inside of a ':local\(\)' or ':global\(\)'/),
	...repeat(2, /A ':local\(' is not allowed inside of a ':local\(\)' or ':global\(\)'/),
	...repeat(6, /Invalid class selector syntax/),
	[/Invalid id selector syntax/],
	...repeat(13, /Expected ident during parsing of '@keyframes' name/),
	...repeat(4, /Expected starts with '--' during parsing of '@property'/),
	...repeat(4, /Expected string or ident during parsing of 'composes'/),
	[/Expected ident during parsing of 'composes'/],
	...repeat(4, /Expected '\{' during parsing of '@keyframes'/),
	...repeat(6, /Expected starts with '--' during parsing of 'var\(\)'/)
];
