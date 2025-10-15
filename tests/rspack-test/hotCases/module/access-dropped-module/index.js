import "./a";

it("should create new JsModule when module changed", (done) => {
    NEXT(
        require('@rspack/test-tools/helper/legacy/update')(done, true, () => {
            done();
        }),
    );
});

module.hot.accept('./a');
