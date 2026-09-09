// Two blocks sharing one chunk name are merged into a single chunk group,
// while the third gets a group of its own.
import(/* webpackChunkName: "shared" */ "./foo");
import(/* webpackChunkName: "shared" */ "./bar");
import(/* webpackChunkName: "lonely" */ "./baz");
