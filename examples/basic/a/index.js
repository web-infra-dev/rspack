import {bbb} from './bbb'
export * from './aaa'

var Index = function() {}
Index.prototype.bbb = bbb;

export default Index;