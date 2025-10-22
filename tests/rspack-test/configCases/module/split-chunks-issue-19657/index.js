import m2 from "./testModule2.js"

it("should compile and evaluate fine", () => {
    expect(m2()).toBe("m11111111");
});

export default "index";
