var P = "initial";
for (P in { expected: true });
var Q = "initial";
for (Q of ["expected"]);

export { P, Q };
