import "./lib";

// Export something to test
export const testExport = "Hello from main module";
export default { message: "Default export from main" };

// Console.log the exports for testing
console.log("Module exports:", {
	testExport,
	default: { message: "Default export from main" }
});
