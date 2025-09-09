import "./cycle1/a";
export default 1;
---
import "./cycle2/a";
export default 2;
---
import "./common";
export default 3;
---
export default 4;
