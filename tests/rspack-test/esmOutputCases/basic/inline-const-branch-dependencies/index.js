import { IS_DEV, IS_DEV1, IS_DEV2 } from "./env";
import * as envNs from "./env";
import { alt } from "./alt";
import { foo } from "./foo";
import { bar } from "./bar";

const values = [];

if (IS_DEV) {
  console.log(foo);
  values.push(foo());
} else {
  console.log(bar);
  values.push(bar());
}

if (IS_DEV && (IS_DEV1 || IS_DEV2)) {
  console.log(foo);
  values.push(foo());
} else {
  console.log(bar);
  values.push(bar());
}

if ((IS_DEV && IS_DEV1) || (IS_DEV && IS_DEV2)) {
  console.log(foo);
  values.push(foo());
} else {
  console.log(bar);
  values.push(bar());
}

if ((IS_DEV || IS_DEV1) && IS_DEV2) {
  console.log(foo);
  values.push(foo());
} else {
  console.log(bar);
  values.push(bar());
}

if (!(!IS_DEV || (!IS_DEV1 && !IS_DEV2))) {
  console.log(foo);
  values.push(foo());
} else {
  console.log(bar);
  values.push(bar());
}

if (IS_DEV1) {
  console.log(bar);
  values.push(bar());
} else {
  console.log(alt);
  values.push(alt());
}

if (IS_DEV && IS_DEV1) {
  console.log(bar);
  values.push(bar());
} else {
  console.log(alt);
  values.push(alt());
}

if (IS_DEV1 || !IS_DEV) {
  console.log(bar);
  values.push(bar());
} else {
  console.log(alt);
  values.push(alt());
}

if (IS_DEV && (IS_DEV1 || false)) {
  console.log(bar);
  values.push(bar());
} else {
  console.log(alt);
  values.push(alt());
}

if (!IS_DEV || (IS_DEV1 && IS_DEV2)) {
  console.log(bar);
  values.push(bar());
} else {
  console.log(alt);
  values.push(alt());
}

if (IS_DEV && IS_DEV1 || IS_DEV2) {
  console.log(foo);
  values.push(foo());
} else {
  console.log(bar);
  values.push(bar());
}

if (IS_DEV1 || IS_DEV && false) {
  console.log(bar);
  values.push(bar());
} else {
  console.log(alt);
  values.push(alt());
}

if ((IS_DEV1 || IS_DEV2) && (IS_DEV && false || IS_DEV2)) {
  console.log(foo);
  values.push(foo());
} else {
  console.log(bar);
  values.push(bar());
}

values.push((IS_DEV ? foo : bar)());
values.push((IS_DEV1 ? bar : alt)());

if (envNs.IS_DEV) {
  console.log(foo);
  values.push(foo());
} else {
  console.log(bar);
  values.push(bar());
}

if (envNs.IS_DEV1) {
  console.log(bar);
  values.push(bar());
} else {
  console.log(alt);
  values.push(alt());
}

if (IS_DEV) {
  if (IS_DEV1) {
    console.log(bar);
    values.push(bar());
  } else {
    console.log(foo);
    values.push(foo());
  }
} else {
  console.log(bar);
  values.push(bar());
}

if (IS_DEV1) {
  console.log(bar);
  values.push(bar());
} else if (IS_DEV2) {
  console.log(foo);
  values.push(foo());
} else {
  console.log(bar);
  values.push(bar());
}

if (IS_DEV1) {
  if (IS_DEV) {
    console.log(bar);
    values.push(bar());
  }
} else {
  if (IS_DEV2) {
    console.log(alt);
    values.push(alt());
  } else {
    console.log(bar);
    values.push(bar());
  }
}

if (IS_DEV1) {
  import("./dynamic-unused");
} else {
  values.push("dynamic-alt");
}

it("should drop inactive ESM branch dependencies guarded by inlined booleans", () => {
  expect(values).toEqual([
    "foo",
    "foo",
    "foo",
    "foo",
    "foo",
    "alt",
    "alt",
    "alt",
    "alt",
    "alt",
    "foo",
    "alt",
    "foo",
    "foo",
    "alt",
    "foo",
    "alt",
    "foo",
    "foo",
    "alt",
    "dynamic-alt",
  ]);
});
