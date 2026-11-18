import directValue from "dir/value.js";

const modules = import.meta.glob("dir/*.js", {
  eager: true,
  import: "default",
});

it("continues to a prefix alias after skipping a trailing-slash exact alias", () => {
  expect(directValue).toBe("redirected");
  expect(modules).toEqual({ "/redirected/value.js": "redirected" });
});
