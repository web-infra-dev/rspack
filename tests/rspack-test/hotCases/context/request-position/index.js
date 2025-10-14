import { fn } from "./file";

it("should not panic when context request position change", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
  (async () => {
    const value = await fn();
    expect(value).toBe(1);
    module.hot.accept("./file", async () => {
      const value = await fn();
      expect(value.a).toBe(1);
      done();
    });
    NEXT(require("../../update")(done));
  })();
}));
