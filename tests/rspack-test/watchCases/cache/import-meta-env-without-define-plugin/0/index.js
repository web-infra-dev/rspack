import { value } from "./foo";

value;

it("should expose an empty env object", () => {
  expect(import.meta.env).toEqual({});
});
