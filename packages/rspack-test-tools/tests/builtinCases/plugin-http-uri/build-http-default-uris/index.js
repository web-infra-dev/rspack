// Test file for default allowedUris behavior (no allowedUris specified)
// When no allowedUris are specified, all URLs should be allowed by default
import allowedModule from "http://localhost:8999/allowed-module.js";
import regexModule from "http://localhost:8999/regex-module.js";
import restrictedModule from "http://localhost:8999/restricted-module.js";

// Test all three modules should load successfully when no allowedUris are specified
it("should load all modules when no allowedUris are specified (allowed module)", () => {
  expect(allowedModule).toBe("This module is from an allowed URI");
});

it("should load all modules when no allowedUris are specified (regex module)", () => {
  expect(regexModule).toBe("This module is from a regex-matched URI");
});

it("should load all modules when no allowedUris are specified (restricted module)", () => {
  // With no allowedUris, even the "restricted" module should load
  expect(restrictedModule).toBe("This module is from a restricted URI");
});

// Export results for validation in other tests if needed
module.exports = {
  allowedModuleLoaded: typeof allowedModule === "string",
  regexModuleLoaded: typeof regexModule === "string",
  restrictedModuleLoaded: typeof restrictedModule === "string"
};
