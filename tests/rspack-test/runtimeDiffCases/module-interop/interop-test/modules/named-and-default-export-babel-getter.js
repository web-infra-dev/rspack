"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true,
});

// prettier-ignore
Object.defineProperty(exports, "named", {
  enumerable: true,
  get: function () {
    return obj.named;
  }
});

// prettier-ignore
Object.defineProperty(exports, "default", {
  enumerable: true,
  get: function () {
    return obj.default;
  }
});

const obj = {
  named: "named",
  default: "default",
};
