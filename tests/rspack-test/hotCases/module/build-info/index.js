import a from "./loader.js!./a";

it("should create new JsModule when module changed", async () => {
    expect(a).toBe(1);
    await NEXT_HMR();
    expect(a).toBe(2);
});

module.hot.accept('./loader.js!./a');
