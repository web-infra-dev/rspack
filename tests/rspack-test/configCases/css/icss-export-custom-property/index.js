import * as asIs from "./exports.module.css";
import * as camelCase from "./exports.module.css?camel-case";
import * as camelCaseOnly from "./exports.module.css?camel-case-only";
import * as collision from "./collision.module.css";
// Copied from webpack/test/configCases/css/css-modules.
import * as variables from "./var-function-export.modules.css";

// Non-link export types also provide a default CSS payload.
const namedExports = ({ default: _, ...exports }) => exports;

it("should replace repeated ICSS exports without creating custom-property aliases", () => {
  expect(namedExports(asIs)).toEqual({
    "--color": "blue",
    "--theme-color": "green"
  });
  expect(namedExports(camelCase)).toEqual({
    "--color": "blue",
    color: "blue",
    "--theme-color": "green",
    themeColor: "green"
  });
  expect(namedExports(camelCaseOnly)).toEqual({
    color: "blue",
    themeColor: "green"
  });
});

it("should keep prefixed and unprefixed ICSS export names independent", () => {
  expect(namedExports(collision)).toEqual({ color: "green", "--color": "blue" });
});

it("should still export real CSS custom properties without the prefix", () => {
  expect(namedExports(variables)).toEqual({
    "my-var-u1": "--my-var-u1",
    "my-var-u2": "--my-var-u2",
    "not-override-class": "--not-override-class",
    "1": "--_1",
    "--a": "--a",
    "main-bg-color": "--main-bg-color",
    "my-var-u1-cls": "my-var-u1-cls"
  });
});
