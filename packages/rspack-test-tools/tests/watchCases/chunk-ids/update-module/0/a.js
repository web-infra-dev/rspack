export default async function() {
  const a1 = await import("./a1.js");
  return a1.default;
}
