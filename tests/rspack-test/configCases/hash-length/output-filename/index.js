it("should compile and run the test " + NAME, function () {});

it("should load additional chunks in " + NAME, () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
  require(['./chunk'], function () {
    done();
  });
}));
