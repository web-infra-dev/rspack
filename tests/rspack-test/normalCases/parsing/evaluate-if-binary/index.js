it("should handle number compare", () => {
  if (4 > 5) require("fail");
  if (4 >= 5) require("fail");
  if (5 < 4) require("fail");
  if (5 <= 4) require("fail");
});

it("should handle string compare", () => {
  if ('10' > '9') require("fail");
  if ('10' >= '9') require("fail");
  if ('9' < '10') require("fail");
  if ('9' <= '10') require("fail");
});

it("should handle bit operation", () => {
  if ((1 & 3) !== 1) require("fail");
  if ((1 | 3) !== 3) require("fail");
  if ((1 ^ 2) !== 3) require("fail");
  if ((1 << 1) !== 2) require("fail");
  if ((2 >> 1) !== 1) require("fail");
  if ((2 >> 32) !== 2) require("fail");
  if ((2 << 32) !== 2) require("fail");
});

it("should handle number operation", () => {
  if (2 - 1 !== 1) require("fail");
  if (2 * 1 !== 2) require("fail");
  if (2 / 1 !== 2) require("fail");
  if (2 ** 2 !== 4) require("fail");
});

it("should handle string number operation", () => {
  if ('2' - '1' !== 1) require("fail");
  if ('2' * '1' !== 2) require("fail");
  if ('2' / '1' !== 2) require("fail");
  if ('2' ** '2' !== 4) require("fail");
});

it("should handle bool number operation", () => {
  if (2 - true !== 1) require("fail");
  if (2 * true !== 2) require("fail");
  if (2 / true !== 2) require("fail");
  if (2 ** false !== 1) require("fail");
});
