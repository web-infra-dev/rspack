// Test file for allowed URIs functionality
// The imports are structured to test different allowedUris scenarios

// Import modules using different URL patterns to test the allowedUris functionality
// These will only work if they match the allowedUris patterns in webpack.config.js
import allowedModule from "http://localhost:8999/allowed-module.js";
import regexModule from "http://localhost:8999/regex-module.js";

// Importing restricted module should fail - we test this in a try/catch below
let restrictedModuleResult = null;
try {
  // This should be blocked as it's not in allowedUris
  restrictedModuleResult = require("http://localhost:8999/restricted-module.js");
  console.log("⚠️ Unexpectedly imported restricted module");
} catch (error) {
  console.log("✅ Correctly blocked restricted module:", error.message);
}

// Simple Jest tests to verify allowedUris functionality
it("should load a module from an allowed URI using string pattern", () => {
  expect(allowedModule).toBe("This module is from an allowed URI");
});

it("should load a module from an allowed URI using regex pattern", () => {
  expect(regexModule).toBe("This module is from a regex-matched URI");
});

it("should block a module from a non-allowed URI", () => {
  // This module should not be loaded as it doesn't match any allowed URI pattern
  expect(restrictedModuleResult).toBe(null);
});

// Export results for verification in other tests if needed
module.exports = {
  allowedModuleLoaded: typeof allowedModule === "string",
  regexModuleLoaded: typeof regexModule === "string",
  restrictedModuleLoaded: restrictedModuleResult !== null
};
