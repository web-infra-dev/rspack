import { describeByWalk, createExampleCase } from "@rspack/test-tools";

describeByWalk(import.meta.filename, (name, src, dist) => {
  createExampleCase(name, src, dist);
}, {
  level: 1,
});
