// Replace dynamic imports with static ESM imports
import module from "http://localhost:8999/module.js";
import module2 from "http://localhost:8999/module2.js";

it("should load a module via http", () => {
  expect(module).toBe("Module from HTTP server");
});

it("should load a module with dependencies via http", () => {
  expect(module2).toBe("Module2 with dependency: Module from HTTP server");
});
