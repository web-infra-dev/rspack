import { answer } from "./data.cjs";

export { answer };

export const required = () => require("./data.cjs");

export const requiredAnswer = () => require("./data.cjs").answer;
