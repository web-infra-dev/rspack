import allowedModule from "http://test.rspack.rs/allowed-module.js";
import regexModule from "http://test.rspack.rs/regex-module.js";
// import restrictedModule from "http://localhost:8999/restricted-module.js";

it("should load a module from an allowed URI using string pattern", () => {
  expect(allowedModule).toBe("This module is from an allowed URI");
});

it("should load a module from an allowed URI using regex pattern", () => {
  expect(regexModule).toBe("This module is from a regex-matched URI");
});

// TODO: fix emit error to stats instead of bailing out the compilation
// it("should block a module from a non-allowed URI", () => {
//   expect(restrictedModule).toBe("Not found: restricted-module.js");
// });
