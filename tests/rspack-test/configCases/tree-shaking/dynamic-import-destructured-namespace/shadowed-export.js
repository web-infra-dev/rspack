export const alias = "unrelated";
export async function read() {
  const {z: alias} = await import(/* webpackChunkName: "shadowed" */ "./shadowed-barrel");
  return alias.used();
}
