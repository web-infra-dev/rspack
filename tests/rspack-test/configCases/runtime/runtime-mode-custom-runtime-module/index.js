it("runs the module interceptor without enabling the optional startup helper", () => {
  expect(__webpack_require__.custom).toEqual({
    intercepted: true,
    onChunksLoaded: "undefined"
  });
});
