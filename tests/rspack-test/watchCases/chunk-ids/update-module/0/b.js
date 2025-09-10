export default async function() {
  const b1 = await import("./b1.js");
  return b1.default;
}
