export const fn = async function () {
  const name = "a";
  const wrap = v => v;
  return wrap((await import(`./lib/${name}.js`)).a);
}

---
export const fn = async function () {
  const name = "a";
  const wrap = v => v;
  return wrap((await import(`./lib/${name}.js`)));
}
