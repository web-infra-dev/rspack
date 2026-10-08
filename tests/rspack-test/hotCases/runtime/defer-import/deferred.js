import { evaluations } from "./state";

evaluations.push("deferred 1");
export let value = 1;
export const increment = () => value++;
module.hot.accept();
---
import { evaluations } from "./state";

evaluations.push("deferred 1");
export let value = 1;
export const increment = () => value++;
module.hot.accept();
---
import { evaluations } from "./state";

evaluations.push("deferred 10");
export let value = 10;
export const increment = () => value++;
module.hot.accept();
