import { makeConfig, register } from "./dep";

// A nested pure (side-effects-free) call assigned to a local binding. Nothing
// reads its value directly, so on its own it would be severed and dropped.
const config = makeConfig();

// A retained deferred-IMPURE expression whose VALUE is unused, but whose side
// effect reads `config`. This is the transitive deferred-pure case: the inner
// graph keeps `register(config)` because `register` is impure, and must in turn
// keep the local pure binding `config` it depends on — with its correct value —
// so the side effect does not run against a severed `undefined`.
export const sideEffectOnly = register(config);

// Keeps the module evaluated.
export const keep = "kept";
