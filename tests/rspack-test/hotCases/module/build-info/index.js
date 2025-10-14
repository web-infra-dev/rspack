import a from "./loader.js!./a";

it("should create new JsModule when module changed", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
    expect(a).toBe(1);
    NEXT(
        require('../../update')(done, true, () => {
            expect(a).toBe(2);
            done();
        }),
    );
}));

module.hot.accept('./loader.js!./a');
