// module.exports = [
	// [/Module not found/, /recursive-file\/a/, /Recursion in resolving/],
	// [/Module not found/, /recursive-file\/b/, /Recursion in resolving/],
	// [/Module not found/, /recursive-file\/c/, /Recursion in resolving/],
	// [/Module not found/, /recursive-file\/d/, /Recursion in resolving/]
// ];
// Rspack emits similar errors:
module.exports = [
	[/maybe it had cycle alias/],
	[/maybe it had cycle alias/],
	[/maybe it had cycle alias/],
	[/maybe it had cycle alias/]
];
