import * as namespace from "./lib.js";
import * as splitNamespace from "./split.js";

it("defines a live ESM namespace", () => {
  expect(Reflect.get(namespace, "__esModule")).toBe(true);
  expect(Object.prototype.toString.call(namespace)).toBe("[object Module]");

  const descriptor = Object.getOwnPropertyDescriptor(namespace, "value");
  expect(descriptor.enumerable).toBe(true);
  expect(typeof descriptor.get).toBe("function");
  expect(descriptor.set).toBeUndefined();

  expect(namespace.value).toBe(1);
  expect(namespace.readValue()).toBe(1);
  namespace.setValue(2);
  expect(namespace.value).toBe(2);
  expect(namespace.readValue()).toBe(2);

  expect(splitNamespace.splitValue).toBe(42);
});
