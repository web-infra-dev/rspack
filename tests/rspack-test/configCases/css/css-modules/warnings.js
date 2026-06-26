"use strict";

const cssModuleWarnings = [
	[/export 'nested2' \(imported as 'style'\) was not found/],
	[/export 'global-color' \(imported as 'style'\) was not found/],
	[/export 'GLOBAL-COLOR' \(imported as 'style'\) was not found/],
	[/Broken '@value' at-rule/]
];

module.exports = [
	...cssModuleWarnings,
	...cssModuleWarnings,
	...cssModuleWarnings,
	...cssModuleWarnings,
	[/Broken '@value' at-rule/],
	[/Broken '@value' at-rule/],
	[/Broken '@value' at-rule/]
];
