import * as ns from "./stub";
import * as nsAlias from "./stub";

function fn() {}

if ("a" in ns) {
  fn(ns.a);
  ns.a();
}
if ("alias" in ns) {
  fn(nsAlias.alias);
}
if ("b" in ns && "c" in ns) {
  fn(ns.b, ns.c);
  ns.b();
  ns.c();
}
if (0 || "a" in ns) {
  fn(ns.a);

  if ("b" in ns && "c" in ns) {
    fn(ns.a, ns.b, ns.c);
  }

  if (null ?? "d" in ns) {
    fn(ns.a, ns.d);
  }
}

if (!("a" in ns)) {
  fn(ns.a);
  fn(ns.b);
}

if (!!("a" in ns)) {
  fn(ns.a);
}

"a" in ns ? fn(ns.a) : 0;
"b" in ns && "c" in ns ? (fn(ns.b, ns.c), ns.b(), ns.c()) : 0;

0 || "a" in ns
  ? fn(ns.a)
  : "b" in ns && "c" in ns
    ? "d" in ns
      ? fn(ns.b, ns.c, ns.d)
      : (null ?? "e" in ns)
        ? fn(ns.b, ns.c, ns.e)
        : fn(ns.b, ns.c)
    : 0;

!!("a" in ns) ? ns.a() : 0;

if (ns.a) {
  console.log(ns.a);
}
if (ns.a !== undefined) {
  console.log(ns.a);
}
