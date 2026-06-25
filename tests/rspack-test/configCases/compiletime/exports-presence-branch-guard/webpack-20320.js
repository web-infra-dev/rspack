import { b, c, d } from "./stub";

function fn() {}

"a" in c ? c.a() : 0;
"a" in c && "a" in b ? b.a(c.a) : 0;

if ("a" in d.c) {
  fn(d.c.a());
}
