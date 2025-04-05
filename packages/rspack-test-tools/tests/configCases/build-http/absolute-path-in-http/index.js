// Import content that contains absolute paths
const { message, getMessage } = require("https://example.com/absolute-path-test.js");

it("should correctly import real module using absolute path from HTTP-imported content", () => {
  // Verify that we can access the real module's exports
  expect(message).toBe("Hello from real module!");
  expect(getMessage()).toBe("Hello from real module!");
});

it("should treat absolute paths as local filesystem paths", () => {
  // The imported content should contain absolute paths that should be treated as local paths
  // and not as HTTP URLs
  expect(typeof message).toBe("string");
  expect(typeof getMessage).toBe("function");
});
