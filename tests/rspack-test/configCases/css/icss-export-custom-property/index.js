import * as asIs from "./exports.module.css";
import * as camelCase from "./exports.module.css?camel-case";
import * as camelCaseOnly from "./exports.module.css?camel-case-only";
import * as collision from "./collision.module.css";
import * as variables from "./variables.module.css";

// Non-link export types also provide a default CSS payload.
const namedExports = ({ default: _, ...exports }) => exports;

it("should replace repeated ICSS exports without creating custom-property aliases", () => {
  expect(namedExports(asIs)).toEqual({
    "--color": "blue",
    "--theme-color": "green",
    repeated: "second"
  });
  expect(namedExports(camelCase)).toEqual({
    "--color": "blue",
    color: "blue",
    "--theme-color": "green",
    themeColor: "green",
    repeated: "second"
  });
  expect(namedExports(camelCaseOnly)).toEqual({
    color: "blue",
    themeColor: "green",
    repeated: "second"
  });
});

it("should keep prefixed and unprefixed ICSS export names independent", () => {
  expect(namedExports(collision)).toEqual({ color: "green", "--color": "blue" });
});

it("should still export real CSS custom properties without the prefix", () => {
  expect(namedExports(variables)).toEqual({ normal: "normal", accent: "--accent" });
});
