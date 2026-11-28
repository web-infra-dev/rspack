import { effects } from "./effects";

export * from "./value";

Array.from([1], () => effects.push("mapped"));
