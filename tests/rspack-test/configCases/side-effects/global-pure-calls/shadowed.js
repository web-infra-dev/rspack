import { effects } from "./effects";

export * from "./value";

const Object = () => {
	effects.push("shadowed");
};

Object();
