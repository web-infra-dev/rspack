export default function () {
  throw new Error("A lower-priority loader must not run");
}
