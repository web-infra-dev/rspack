import { broken } from "./broken.js";

export class Other {}

Other.doSomething = () => "other";

export function callBrokenFromOther() {
  return broken();
}
