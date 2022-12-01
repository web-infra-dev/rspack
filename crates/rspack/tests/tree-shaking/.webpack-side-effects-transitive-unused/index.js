import { log } from "pmodule/tracker";
import { a, y } from "pmodule";

a.should.be.eql("a");
y.should.be.eql("y");
log.should.be.eql(["a.js", "b.js"]);
