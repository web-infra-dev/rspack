it("should keep nested commonjs require reexports stable", function() {
  const ns = require("./root");

  expect(ns).toEqual(
    nsObj({
      a: "a",
      b: "b"
    })
  );
});
