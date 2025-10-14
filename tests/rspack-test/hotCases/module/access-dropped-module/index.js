import "./a";

it("should create new JsModule when module changed", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
    NEXT(
        require('../../update')(done, true, () => {
            done();
        }),
    );
}));

module.hot.accept('./a');
