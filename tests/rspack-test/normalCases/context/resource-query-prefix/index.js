it("should detect query strings in dynamic import in sub directories by concat as a static value", function () {
  var testFileName = "a";

  return Promise.all([
    import(`./sub/${testFileName}`).then(({ default: a }) => {
      expect(a()).toBe("a");
    }),
    import(`./sub/${testFileName}`).then(({ default: a }) => {
      expect(a()).toBe("a");
    }),
    import(`./sub/${testFileName}bc`).then(({ default: a }) => {
      expect(a()).toBe("abc");
    }),
    import(`./sub/${testFileName}?queryString`).then(({ default: a }) => {
      expect(a()).toBe("a?queryString");
    })
  ]);
});

it("should detect query strings in dynamic import in sub directories by template string as a static value", function () {
  var testFileName = "a";

  return Promise.all([
    import("./sub/".concat(testFileName)).then(({ default: a }) => {
      expect(a()).toBe("a");
    }),
    import("./sub/".concat(testFileName).concat("")).then(({ default: a }) => {
      expect(a()).toBe("a");
    }),
    import("./sub/".concat(testFileName).concat("bc")).then(({ default: a }) => {
      expect(a()).toBe("abc");
    }),
    import("./sub/".concat(testFileName).concat("?queryString")).then(({ default: a }) => {
      expect(a()).toBe("a?queryString");
    })
  ]);
});

it("should detect query strings in dynamic import in sub directories by add as a static value", function () {
  var testFileName = "a";

  return Promise.all([
    import("./sub/" + testFileName).then(({ default: a }) => {
      expect(a()).toBe("a");
    }),
    import("./sub/" + testFileName + "").then(({ default: a }) => {
      expect(a()).toBe("a");
    }),
    import("./sub/" + testFileName + "bc").then(({ default: a }) => {
      expect(a()).toBe("abc");
    }),
    import("./sub/" + testFileName + "?queryString").then(({ default: a }) => {
      expect(a()).toBe("a?queryString");
    })
  ]);
});

