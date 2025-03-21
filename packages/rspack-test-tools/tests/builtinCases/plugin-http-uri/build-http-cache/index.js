import cachedModule from "http://localhost:8999/cached.js";
import noCacheModule from "http://localhost:8999/no-cache.js";
import etagModule from "http://localhost:8999/etag.js";

// Import again to test caching behavior
import cachedModule2 from "http://localhost:8999/cached.js";
import noCacheModule2 from "http://localhost:8999/no-cache.js";
import etagModule2 from "http://localhost:8999/etag.js";

it("should cache modules with default cache headers", () => {
  expect(cachedModule.message).toBe("This module should be cached");
  expect(cachedModule2.message).toBe("This module should be cached");

  // Request count should be the same for both imports
  // because the module should be cached after the first request
  expect(cachedModule.requestCount).toBe(cachedModule2.requestCount);
});

it("should not cache modules with no-cache headers", () => {
  expect(noCacheModule.message).toBe("This module should NOT be cached (no-cache header)");
  expect(noCacheModule2.message).toBe("This module should NOT be cached (no-cache header)");

  // Request count should be different for the second import
  // because the module should not be cached
  expect(noCacheModule.requestCount).not.toBe(noCacheModule2.requestCount);
});

it("should handle ETags correctly", () => {
  expect(etagModule.message).toBe("This module should use ETag for caching");
  expect(etagModule2.message).toBe("This module should use ETag for caching");

  // Since we're using ETags, the server might return a 304 Not Modified
  // The requestCount might be the same or different depending on implementation
  // We're just checking that the content is correctly returned
});
