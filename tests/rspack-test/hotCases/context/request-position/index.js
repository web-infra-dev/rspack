import { fn } from "./file";

it("should not panic when context request position change", async () => {
  await (async () => {
    let value = await fn();
    expect(value).toBe(1);
    await NEXT_HMR();
    value = await fn();
    expect(value.a).toBe(1);
  })();
});

module.hot.accept("./file");