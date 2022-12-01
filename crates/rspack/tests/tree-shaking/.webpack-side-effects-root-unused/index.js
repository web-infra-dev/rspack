import { log } from "pmodule/tracker";
import { a, z, x } from "pmodule";

a.should.be.eql("a");
x.should.be.eql("x");
z.should.be.eql("z");
log.should.be.eql(["a.js", "b.js", "c.js"]);
