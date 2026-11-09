import { infinity, large, zero } from "./constants";

it("should use JavaScript number strings in constant conditions", () => {
  if (String(-0) !== "0") require("fail");
  if (`${-0}` !== "0") require("fail");
  if ("" + 1 / 0 !== "Infinity") require("fail");
  if ("".concat(-1 / 0) !== "-Infinity") require("fail");
  if ("" + 1e21 !== "1e+21") require("fail");
  if (1e-7 + "" !== "1e-7") require("fail");
});

it("should preserve number strings across module boundaries", () => {
  expect(infinity).toBe("Infinity");
  expect(zero).toBe("0");
  expect(large).toBe("1e+21");
});

it("should use JavaScript number strings in dynamic request suffixes", () => {
  const withNumber = name => require("./modules/" + name + -0 + ".js");
  const withStringAndNumber = name => require("./modules/" + name + "m" + -0 + ".js");
  expect(withNumber("item")).toBe("item0");
  expect(withStringAndNumber("ite")).toBe("item0");
});
