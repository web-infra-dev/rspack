// Test dynamic imports from HTTP URLs
console.log("Testing dynamic imports from HTTP URLs...");

const dynamicImportContainer = document.getElementById("dynamic-import-result");
if (dynamicImportContainer) {
	dynamicImportContainer.innerHTML = "Loading dynamic import...";

	// Dynamic import from HTTP URL
	import("https://esm.sh/lodash-es@4.17.21/camelCase")
		.then(module => {
			console.log("Successfully loaded module:", module);
			// Test the module
			const result = module.default("hello world");
			console.log("Module function result:", result);
			dynamicImportContainer.innerHTML = `Dynamic import successful: camelCase('hello world') = "${result}"`;
		})
		.catch(error => {
			console.error("Error loading module:", error);
			dynamicImportContainer.innerHTML = `Error loading dynamic import: ${error.message}`;
		});
}
