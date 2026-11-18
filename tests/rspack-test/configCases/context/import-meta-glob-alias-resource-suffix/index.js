const queryModules = import.meta.glob("query-assets/*.js", { eager: true, import: "default" });
const fragmentModules = import.meta.glob("fragment-assets/*.js", { eager: true, import: "default" });
const bothModules = import.meta.glob("assets/*.js", { eager: true, import: "default" });
const explicitModules = import.meta.glob("assets/*.js", {
  eager: true,
  import: "default",
  query: "?explicit",
});
const commonModules = import.meta.glob(["assets/*.js", "same-assets/*.js"], {
  eager: true,
  import: "default",
});
const excludedModules = import.meta.glob(["assets/*.js", "!fragment-assets/excluded.js"], {
  eager: true,
  import: "default",
});
const fragmentExplicitModules = import.meta.glob("fragment-assets/*.js", {
  eager: true,
  import: "default",
  query: "?explicit",
});
const mixedModules = import.meta.glob(["assets/*.js", "./local/*.js"], {
  eager: true,
  import: "default",
  query: "?explicit",
});

const distinctSuffixModules = import.meta.glob(
  ["raw-assets/*.js", "url-assets/*.js", "./local/*.js"],
  { eager: true, import: "default", query: "?explicit" },
);

it("preserves each alias suffix when positive patterns use different suffixes", () => {
  expect(distinctSuffixModules).toEqual({
    "/raw-dir/value.js": { query: "?raw", fragment: "#raw-fragment" },
    "/url-dir/value.js": { query: "?url", fragment: "#url-fragment" },
    "./local/value.js": { query: "?explicit", fragment: "" },
  });
});

it("preserves alias query and fragment on glob elements", () => {
  expect(queryModules["/dir/value.js"]).toEqual({ query: "?raw", fragment: "" });
  expect(fragmentModules["/dir/value.js"]).toEqual({ query: "", fragment: "#fragment" });
  expect(bothModules["/dir/value.js"]).toEqual({ query: "?raw", fragment: "#fragment" });
  expect(explicitModules["/dir/value.js"]).toEqual({ query: "?raw", fragment: "#fragment" });
  expect(commonModules["/dir/value.js"]).toEqual({ query: "?raw", fragment: "#fragment" });
  expect(excludedModules).toEqual({
    "/dir/value.js": { query: "?raw", fragment: "#fragment" },
  });
  expect(fragmentExplicitModules["/dir/value.js"]).toEqual({ query: "?explicit", fragment: "#fragment" });
  expect(mixedModules["./local/value.js"]).toEqual({ query: "?explicit", fragment: "" });
});
