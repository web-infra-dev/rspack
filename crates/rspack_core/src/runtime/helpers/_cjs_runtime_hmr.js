class Hot {
  constructor(id) {
    this.id = id;
    this.accepts = [];
  }
  accept(ids, callback) {
    if (ids === undefined) {
      this.accepts.push({
        ids: this.id,
        accept: undefined,
      });
    } else if (typeof ids === 'function') {
      this.accepts.push({
        ids: this.id,
        accept: ids,
      });
    } else {
      this.accepts.push({
        ids,
        accept: callback,
      });
    }
  }
  dispose(callback) {
    this.accepts.push({
      id: this.id,
      dispose: callback,
    });
  }
}

globalThis.rs.Hot = globalThis.rs.Hot || Hot;

function __invalidate__(dirtyId) {
  const modules = rs.m;
  rs.require(dirtyId);

  const module = modules[dirtyId];
  const hmrBoundaries = new Set(); // 所有的hmr boundary,即所有的收到冒泡影响的含有accept的模块
  const hotMetaList = new Set(); // hmrBoundary 模块关联的accept回调
  const removeModules = new Set(); // 在冒泡规则中收到影响的所有模块

  collectModules(dirtyId);

  console.log('hmr:', hmrBoundaries, hotMetaList, removeModules);

  /**
   * 卸载所有过时的模块，等待重新触发副作用
   */
  for (const mod of removeModules.values()) {
    modules[mod].loaded = false;
  }
  /**
   * 重新执行boundary的所有模块
   */
  for (const id of hmrBoundaries.values()) {
    rs.require(id);
  }
  /**
   * 触发meta列表
   */
  for (const hot of hotMetaList.values()) {
    if (typeof hot.ids === 'string') {
      if (hot.accept) {
        const mod = modules[hot.ids];
        hot.accept(mod.exports);
      }
    } else {
      hot.accept(hot.ids.map((id) => modules[id].exports));
    }
  }
  /**
   * id: 当前模块id
   *
   */
  function collectModules(id) {
    removeModules.add(id);
    const module = modules[id];
    /**
     * 计算自身的accept
     */
    const selfAccepts = getAccepts(id, module);
    // 如果存在自身的accept，执行自身的accept,冒泡终止
    if (selfAccepts.length > 0) {
      hmrBoundaries.add(id);
      for (const accept of selfAccepts) {
        hotMetaList.add(accept);
      }
    } else {
      // 向上冒泡
      for (const m of module.parents.values()) {
        /**
         * 断开原有的父子关系
         */
        // m.children.delete(module);
        // module.parents.delete(m);
        const childAccepts = getAccepts(id, m);
        // 如果在父模块accept了子模块，那么停止冒泡
        if (childAccepts.length > 0) {
          for (const accept of childAccepts) {
            // 子模块为boundary
            hmrBoundaries.add(module.id);
            hotMetaList.add(accept);
          }
        } else {
          cllectModules(m.id);
        }
      }
    }
  }
  /**
   * id: 当前修改的模块id
   * module: 待检查的module
   */
  function getAccepts(id, module) {
    return module.hot.accepts.filter(({ ids }) => {
      if (typeof ids === 'string') {
        return ids === id;
      } else {
        return ids.includes(id);
      }
    });
  }
}

const socketUrl = `ws://${location.host}/`;
const socket = new WebSocket(socketUrl, 'web-server');
function reload() {
  setTimeout(() => {
    window.location.reload();
  });
}
socket.onmessage = function (event) {
  const data = JSON.parse(event.data);
  (0, eval)(data.code);
  // reload();
};

globalThis.rs.invalidate = globalThis.rs.invalidate || __invalidate__;
