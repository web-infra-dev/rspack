import httpModule from "http://localhost/module.js";
import customHttpClient from "./custom-http-client";

it("should load a module using the custom HTTP client", () => {
  expect(httpModule).toBe("Module from custom HTTP client");
});

it("should track requests made by the custom HTTP client", () => {
  const requests = customHttpClient.getRequests();

  // At least one request should have been made
  expect(requests.length).toBeGreaterThan(0);

  // The request should be for our module
  expect(requests.some(req => req.url === "http://localhost/module.js")).toBe(true);

  // Check that headers were passed to the client
  const moduleRequest = requests.find(req => req.url === "http://localhost/module.js");
  expect(moduleRequest).toBeDefined();
  expect(moduleRequest.headers).toBeDefined();
});
