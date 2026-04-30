import { IS_DEV, IS_DEV1, IS_DEV2 } from "./env";
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
  ]);
});
