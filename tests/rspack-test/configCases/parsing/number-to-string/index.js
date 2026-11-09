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

it("should join array elements when folding String()", () => {
  if (String([-0]) === "[0]") require("fail");
  if (String([]) !== "") require("fail");
  if (String([null, [-0, 1e21], undefined]) !== ",0,1e+21,") require("fail");

  // Leave sparse arrays and values that are only known at runtime to JavaScript.
  expect(String([, -0]) === ",0" ? "match" : "wrong").toBe("match");
  const isEmpty = undefined => String([undefined]) === "" ? true : false;
  expect(isEmpty("x")).toBe(false);
  expect(isEmpty(undefined)).toBe(true);
});

it("should compare known template prefixes and suffixes at Unicode boundaries", () => {
  function check(value) {
    if (`${-0}${value}` === `é${value}`) require("fail");
    if (`${value}é` === `${value}${-0}`) require("fail");
  }
  check("x");

  // Compatible prefixes alone cannot decide equality.
  const matches = (a, b) => `é${a}` === `é0${b}` ? true : false;
  expect(matches("0x", "x")).toBe(true);
  expect(matches("x", "x")).toBe(false);
});

it("should use JavaScript number strings in dynamic request suffixes", () => {
  const withNumber = name => require("./modules/" + name + -0 + ".js");
  const withStringAndNumber = name => require("./modules/" + name + "m" + -0 + ".js");
  expect(withNumber("item")).toBe("item0");
  expect(withStringAndNumber("ite")).toBe("item0");
});
