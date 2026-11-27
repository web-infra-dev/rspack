import { hero } from "./style.module.css";

it("should allow local view transition names in pure mode", () => {
  expect(hero).toBe("demo-hero");
});
