import data from "./data.cjs";

export const answer = data.answer;

export const lazy = () => import("./data.cjs").then(m => m.default);
