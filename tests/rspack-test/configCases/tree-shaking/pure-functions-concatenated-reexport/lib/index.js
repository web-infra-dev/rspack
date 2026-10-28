import { createWidget, impure } from "./feature";
import { Base } from "./base";

export * from "./feature";
export * from "./item";

export const widget = createWidget(Base);
export const retained = impure();
