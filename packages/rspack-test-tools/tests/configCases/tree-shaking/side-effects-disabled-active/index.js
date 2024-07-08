import "./side-effect"
import { log } from "./tracker"

it("should not optimize side-effect", () => {
  expect(log).toEqual(["side-effect.js"])
})
