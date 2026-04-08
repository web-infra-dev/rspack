import { pure } from "./dep";
import { track } from "./tracker";

export const value = track("impure") + /* #__NO_SIDE_EFFECTS__ */ pure();
