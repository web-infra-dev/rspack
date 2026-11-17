it("should resolve worker additional data from the main registry", () => {
  expect(require("./a")).toEqual({
    main: true,
    buffer: "worker",
    map: "value",
  });
});
