import "./a";

it("should create new JsModule when module changed", async () => {
    await NEXT_HMR();
});

module.hot.accept('./a');
