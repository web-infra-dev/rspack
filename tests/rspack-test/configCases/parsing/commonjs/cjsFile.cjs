const value = () => 42

// Make the export immutable
Object.defineProperty(module, 'exports', {
	enumerable: true,
	get: value
});
