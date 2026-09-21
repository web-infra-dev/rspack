import producer from "./producer-loader.mjs";

export default function () {
  throw new Error("A pitch result should skip the producer normal function");
};

export const pitch = producer;
