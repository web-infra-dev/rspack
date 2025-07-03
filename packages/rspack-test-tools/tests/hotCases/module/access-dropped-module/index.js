import "./a";

it("should create new JsModule when module changed", (done) => {
    NEXT(
        require('../../update')(done, true, () => {
            done();
        }),
    );
});

module.hot.accept('./a');
