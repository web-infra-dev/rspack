import { price as named } from "./named.js";
import { price as namespace } from "./namespace.js";
import { price as defaultImport } from "./default.js";
import commonjs from "./commonjs.cjs";
import folded from "./folded.cjs";
import { price as dynamic } from "./dynamic.js";
import { factory } from "./factory.js";
import { isNativeBuiltin } from "./benign.js";
import { value } from "./ordinary.js";
import "./resolve-only.js";

export async function readAll() {
  const escapedFactory = factory;
  return [
    named(),
    namespace(),
    defaultImport(),
    commonjs.price(),
    folded.price(),
    await dynamic(),
    escapedFactory(__FIXTURE_PARENT__)("./provider.cjs").price,
    isNativeBuiltin(),
    value(),
  ];
}
