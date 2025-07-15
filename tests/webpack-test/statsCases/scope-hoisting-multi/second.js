import c from "./common";
import v from "./vendor";

import(/* webpackChunkName: "lazy_second" */ "./lazy_second");
import(/* webpackChunkName: "lazy_shared" */ "./lazy_shared");

export default v + c;
