"use strict";

module.exports = Array.from({ length: 4 }).flatMap(() => [
	[/export 'nested2' \(imported as 'style'\) was not found/],
	[/export 'global-color' \(imported as 'style'\) was not found/],
	[/export 'GLOBAL-COLOR' \(imported as 'style'\) was not found/]
]);
