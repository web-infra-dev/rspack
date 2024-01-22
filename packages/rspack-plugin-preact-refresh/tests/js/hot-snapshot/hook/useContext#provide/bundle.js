(() => { // webpackBootstrap
var __webpack_modules__ = ({
"../../../../../../node_modules/@prefresh/core/src/computeKey.js": (function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
"use strict";
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  computeKey: function() { return computeKey; }
});
/* harmony import */var _runtime_signaturesForType__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./runtime/signaturesForType */ "../../../../../../node_modules/@prefresh/core/src/runtime/signaturesForType.js");


/**
 *
 * This part has been vendored from "react-refresh"
 * https://github.com/facebook/react/blob/master/packages/react-refresh/src/ReactFreshRuntime.js#L83
 */
const computeKey = signature => {
  let fullKey = signature.key;
  let hooks;

  try {
    hooks = signature.getCustomHooks();
  } catch (err) {
    signature.forceReset = true;
    return fullKey;
  }

  for (let i = 0; i < hooks.length; i++) {
    const hook = hooks[i];
    if (typeof hook !== 'function') {
      signature.forceReset = true;
      return fullKey;
    }

    const nestedHookSignature = _runtime_signaturesForType__WEBPACK_IMPORTED_MODULE_0__.signaturesForType.get(hook);
    if (nestedHookSignature === undefined) continue;

    const nestedHookKey = computeKey(nestedHookSignature);
    if (nestedHookSignature.forceReset) signature.forceReset = true;

    fullKey += '\n---\n' + nestedHookKey;
  }

  return fullKey;
};


}),
"../../../../../../node_modules/@prefresh/core/src/constants.js": (function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
"use strict";
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  CATCH_ERROR_OPTION: function() { return CATCH_ERROR_OPTION; },
  COMPONENT_DIRTY: function() { return COMPONENT_DIRTY; },
  COMPONENT_HOOKS: function() { return COMPONENT_HOOKS; },
  EFFECTS_LIST: function() { return EFFECTS_LIST; },
  HOOKS_LIST: function() { return HOOKS_LIST; },
  HOOK_ARGS: function() { return HOOK_ARGS; },
  HOOK_CLEANUP: function() { return HOOK_CLEANUP; },
  HOOK_VALUE: function() { return HOOK_VALUE; },
  NAMESPACE: function() { return NAMESPACE; },
  RERENDER_COUNT: function() { return RERENDER_COUNT; },
  VNODE_CHILDREN: function() { return VNODE_CHILDREN; },
  VNODE_COMPONENT: function() { return VNODE_COMPONENT; },
  VNODE_DOM: function() { return VNODE_DOM; }
});
const VNODE_COMPONENT = '__c';
const NAMESPACE = '__PREFRESH__';
const COMPONENT_HOOKS = '__H';
const HOOKS_LIST = '__';
const EFFECTS_LIST = '__h';
const RERENDER_COUNT = '__r';
const CATCH_ERROR_OPTION = '__e';
const COMPONENT_DIRTY = '__d';
const VNODE_DOM = '__e';
const VNODE_CHILDREN = '__k';
const HOOK_VALUE = '__';
const HOOK_ARGS = '__H';
const HOOK_CLEANUP = '__c';


}),
"../../../../../../node_modules/@prefresh/core/src/index.js": (function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony import */var _runtime_catchError__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./runtime/catchError */ "../../../../../../node_modules/@prefresh/core/src/runtime/catchError.js");
/* harmony import */var _runtime_debounceRendering__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./runtime/debounceRendering */ "../../../../../../node_modules/@prefresh/core/src/runtime/debounceRendering.js");
/* harmony import */var _runtime_vnode__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./runtime/vnode */ "../../../../../../node_modules/@prefresh/core/src/runtime/vnode.js");
/* harmony import */var _runtime_unmount__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./runtime/unmount */ "../../../../../../node_modules/@prefresh/core/src/runtime/unmount.js");
/* harmony import */var preact__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! preact */ "../../../../../../node_modules/preact/dist/preact.js");
/* harmony import */var _constants__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! ./constants */ "../../../../../../node_modules/@prefresh/core/src/constants.js");
/* harmony import */var _computeKey__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! ./computeKey */ "../../../../../../node_modules/@prefresh/core/src/computeKey.js");
/* harmony import */var _runtime_vnodesForComponent__WEBPACK_IMPORTED_MODULE_7__ = __webpack_require__(/*! ./runtime/vnodesForComponent */ "../../../../../../node_modules/@prefresh/core/src/runtime/vnodesForComponent.js");
/* harmony import */var _runtime_signaturesForType__WEBPACK_IMPORTED_MODULE_8__ = __webpack_require__(/*! ./runtime/signaturesForType */ "../../../../../../node_modules/@prefresh/core/src/runtime/signaturesForType.js");
// Options for Preact.












let typesById = new Map();
let pendingUpdates = [];

function sign(type, key, forceReset, getCustomHooks, status) {
  if (type) {
    let signature = _runtime_signaturesForType__WEBPACK_IMPORTED_MODULE_8__.signaturesForType.get(type);
    if (status === 'begin') {
      _runtime_signaturesForType__WEBPACK_IMPORTED_MODULE_8__.signaturesForType.set(type, {
        type,
        key,
        forceReset,
        getCustomHooks: getCustomHooks || (() => []),
      });

      return 'needsHooks';
    } else if (status === 'needsHooks') {
      signature.fullKey = (0,_computeKey__WEBPACK_IMPORTED_MODULE_6__.computeKey)(signature);
    }
  }
}

function replaceComponent(OldType, NewType, resetHookState) {
  const vnodes = _runtime_vnodesForComponent__WEBPACK_IMPORTED_MODULE_7__.vnodesForComponent.get(OldType);
  if (!vnodes) return;

  // migrate the list to our new constructor reference
  _runtime_vnodesForComponent__WEBPACK_IMPORTED_MODULE_7__.vnodesForComponent["delete"](OldType);
  _runtime_vnodesForComponent__WEBPACK_IMPORTED_MODULE_7__.vnodesForComponent.set(NewType, vnodes);

  _runtime_vnodesForComponent__WEBPACK_IMPORTED_MODULE_7__.mappedVNodes.set(OldType, NewType);

  pendingUpdates = pendingUpdates.filter(p => p[0] !== OldType);

  vnodes.forEach(vnode => {
    if (!vnode.__c || !vnode.__c.__P) return;
    // update the type in-place to reference the new component
    vnode.type = NewType;

    if (vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT]) {
      vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT].constructor = vnode.type;

      try {
        if (vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT] instanceof OldType) {
          const oldInst = vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT];

          const newInst = new NewType(
            vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT].props,
            vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT].context
          );

          vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT] = newInst;
          // copy old properties onto the new instance.
          //   - Objects (including refs) in the new instance are updated with their old values
          //   - Missing or null properties are restored to their old values
          //   - Updated Functions are not reverted
          //   - Scalars are copied
          for (let i in oldInst) {
            const type = typeof oldInst[i];
            if (!(i in newInst)) {
              newInst[i] = oldInst[i];
            } else if (type !== 'function' && typeof newInst[i] === type) {
              if (
                type === 'object' &&
                newInst[i] != null &&
                newInst[i].constructor === oldInst[i].constructor
              ) {
                Object.assign(newInst[i], oldInst[i]);
              } else {
                newInst[i] = oldInst[i];
              }
            }
          }
        }
      } catch (e) {
        /* Functional component */
        vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT].constructor = NewType;
      }

      if (resetHookState) {
        if (
          vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT][_constants__WEBPACK_IMPORTED_MODULE_5__.COMPONENT_HOOKS] &&
          vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT][_constants__WEBPACK_IMPORTED_MODULE_5__.COMPONENT_HOOKS][_constants__WEBPACK_IMPORTED_MODULE_5__.HOOKS_LIST] &&
          vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT][_constants__WEBPACK_IMPORTED_MODULE_5__.COMPONENT_HOOKS][_constants__WEBPACK_IMPORTED_MODULE_5__.HOOKS_LIST].length
        ) {
          vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT][_constants__WEBPACK_IMPORTED_MODULE_5__.COMPONENT_HOOKS][_constants__WEBPACK_IMPORTED_MODULE_5__.HOOKS_LIST].forEach(
            possibleEffect => {
              if (
                possibleEffect[_constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_CLEANUP] &&
                typeof possibleEffect[_constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_CLEANUP] === 'function'
              ) {
                possibleEffect[_constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_CLEANUP]();
                possibleEffect[_constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_CLEANUP] = undefined;
              } else if (
                possibleEffect[_constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_ARGS] &&
                possibleEffect[_constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_VALUE] &&
                Object.keys(possibleEffect).length === 3
              ) {
                const cleanupKey = Object.keys(possibleEffect).find(
                  key => key !== _constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_ARGS && key !== _constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_VALUE
                );
                if (
                  cleanupKey &&
                  typeof possibleEffect[cleanupKey] == 'function'
                ) {
                  possibleEffect[cleanupKey]();
                  possibleEffect[cleanupKey] = undefined;
                }
              }
            }
          );
        }

        vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT][_constants__WEBPACK_IMPORTED_MODULE_5__.COMPONENT_HOOKS] = {
          [_constants__WEBPACK_IMPORTED_MODULE_5__.HOOKS_LIST]: [],
          [_constants__WEBPACK_IMPORTED_MODULE_5__.EFFECTS_LIST]: [],
        };
      } else if (
        vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT][_constants__WEBPACK_IMPORTED_MODULE_5__.COMPONENT_HOOKS] &&
        vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT][_constants__WEBPACK_IMPORTED_MODULE_5__.COMPONENT_HOOKS][_constants__WEBPACK_IMPORTED_MODULE_5__.HOOKS_LIST] &&
        vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT][_constants__WEBPACK_IMPORTED_MODULE_5__.COMPONENT_HOOKS][_constants__WEBPACK_IMPORTED_MODULE_5__.HOOKS_LIST].length
      ) {
        vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT][_constants__WEBPACK_IMPORTED_MODULE_5__.COMPONENT_HOOKS][_constants__WEBPACK_IMPORTED_MODULE_5__.HOOKS_LIST].forEach(
          possibleEffect => {
            if (
              possibleEffect[_constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_CLEANUP] &&
              typeof possibleEffect[_constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_CLEANUP] === 'function'
            ) {
              possibleEffect[_constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_CLEANUP]();
              possibleEffect[_constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_CLEANUP] = undefined;
            } else if (
              possibleEffect[_constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_ARGS] &&
              possibleEffect[_constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_VALUE] &&
              Object.keys(possibleEffect).length === 3
            ) {
              const cleanupKey = Object.keys(possibleEffect).find(
                key => key !== _constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_ARGS && key !== _constants__WEBPACK_IMPORTED_MODULE_5__.HOOK_VALUE
              );
              if (cleanupKey && typeof possibleEffect[cleanupKey] == 'function')
                possibleEffect[cleanupKey]();
              possibleEffect[cleanupKey] = undefined;
            }
          }
        );

        vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT][_constants__WEBPACK_IMPORTED_MODULE_5__.COMPONENT_HOOKS][_constants__WEBPACK_IMPORTED_MODULE_5__.HOOKS_LIST].forEach(hook => {
          if (hook.__H && Array.isArray(hook.__H)) {
            hook.__H = undefined;
          }
        });
      }

      preact__WEBPACK_IMPORTED_MODULE_4__.Component.prototype.forceUpdate.call(vnode[_constants__WEBPACK_IMPORTED_MODULE_5__.VNODE_COMPONENT]);
    }
  });
}

self[_constants__WEBPACK_IMPORTED_MODULE_5__.NAMESPACE] = {
  getSignature: type => _runtime_signaturesForType__WEBPACK_IMPORTED_MODULE_8__.signaturesForType.get(type),
  register: (type, id) => {
    if (typeof type !== 'function') return;

    if (typesById.has(id)) {
      const existing = typesById.get(id);
      if (existing !== type) {
        pendingUpdates.push([existing, type]);
        typesById.set(id, type);
      }
    } else {
      typesById.set(id, type);
    }

    if (!_runtime_signaturesForType__WEBPACK_IMPORTED_MODULE_8__.signaturesForType.has(type)) {
      _runtime_signaturesForType__WEBPACK_IMPORTED_MODULE_8__.signaturesForType.set(type, {
        getCustomHooks: () => [],
        type,
      });
    }
  },
  getPendingUpdates: () => pendingUpdates,
  flush: () => {
    pendingUpdates = [];
  },
  replaceComponent,
  sign,
  computeKey: _computeKey__WEBPACK_IMPORTED_MODULE_6__.computeKey,
};


}),
"../../../../../../node_modules/@prefresh/core/src/runtime/catchError.js": (function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony import */var preact__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! preact */ "../../../../../../node_modules/preact/dist/preact.js");
/* harmony import */var _constants__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ../constants */ "../../../../../../node_modules/@prefresh/core/src/constants.js");



const oldCatchError = preact__WEBPACK_IMPORTED_MODULE_0__.options[_constants__WEBPACK_IMPORTED_MODULE_1__.CATCH_ERROR_OPTION];
preact__WEBPACK_IMPORTED_MODULE_0__.options[_constants__WEBPACK_IMPORTED_MODULE_1__.CATCH_ERROR_OPTION] = (error, vnode, oldVNode) => {
  if (vnode[_constants__WEBPACK_IMPORTED_MODULE_1__.VNODE_COMPONENT] && vnode[_constants__WEBPACK_IMPORTED_MODULE_1__.VNODE_COMPONENT][_constants__WEBPACK_IMPORTED_MODULE_1__.COMPONENT_DIRTY]) {
    vnode[_constants__WEBPACK_IMPORTED_MODULE_1__.VNODE_COMPONENT][_constants__WEBPACK_IMPORTED_MODULE_1__.COMPONENT_DIRTY] = false;
  }

  if (oldCatchError) oldCatchError(error, vnode, oldVNode);
};


}),
"../../../../../../node_modules/@prefresh/core/src/runtime/debounceRendering.js": (function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony import */var preact__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! preact */ "../../../../../../node_modules/preact/dist/preact.js");
/* harmony import */var _constants__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ../constants */ "../../../../../../node_modules/@prefresh/core/src/constants.js");




const defer =
  typeof Promise == 'function'
    ? Promise.prototype.then.bind(Promise.resolve())
    : setTimeout;

preact__WEBPACK_IMPORTED_MODULE_0__.options.debounceRendering = process => {
  defer(() => {
    try {
      process();
    } catch (e) {
      process[_constants__WEBPACK_IMPORTED_MODULE_1__.RERENDER_COUNT] = 0;
      throw e;
    }
  });
};


}),
"../../../../../../node_modules/@prefresh/core/src/runtime/signaturesForType.js": (function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
"use strict";
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  signaturesForType: function() { return signaturesForType; }
});
// Signatures for functional components and custom hooks.
const signaturesForType = new WeakMap();


}),
"../../../../../../node_modules/@prefresh/core/src/runtime/unmount.js": (function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony import */var preact__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! preact */ "../../../../../../node_modules/preact/dist/preact.js");
/* harmony import */var _vnodesForComponent__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./vnodesForComponent */ "../../../../../../node_modules/@prefresh/core/src/runtime/vnodesForComponent.js");



const oldUnmount = preact__WEBPACK_IMPORTED_MODULE_0__.options.unmount;
preact__WEBPACK_IMPORTED_MODULE_0__.options.unmount = vnode => {
  const type = (vnode || {}).type;
  if (typeof type === 'function' && _vnodesForComponent__WEBPACK_IMPORTED_MODULE_1__.vnodesForComponent.has(type)) {
    const vnodes = _vnodesForComponent__WEBPACK_IMPORTED_MODULE_1__.vnodesForComponent.get(type);
    if (vnodes) {
      const index = vnodes.indexOf(vnode);
      if (index !== -1) {
        vnodes.splice(index, 1);
      }
    }
  }

  if (oldUnmount) oldUnmount(vnode);
};


}),
"../../../../../../node_modules/@prefresh/core/src/runtime/vnode.js": (function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony import */var preact__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! preact */ "../../../../../../node_modules/preact/dist/preact.js");
/* harmony import */var _vnodesForComponent__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./vnodesForComponent */ "../../../../../../node_modules/@prefresh/core/src/runtime/vnodesForComponent.js");
/* harmony import */var _constants__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ../constants */ "../../../../../../node_modules/@prefresh/core/src/constants.js");




const getMappedVnode = type => {
  if (_vnodesForComponent__WEBPACK_IMPORTED_MODULE_1__.mappedVNodes.has(type)) {
    return getMappedVnode(_vnodesForComponent__WEBPACK_IMPORTED_MODULE_1__.mappedVNodes.get(type));
  }

  return type;
};

const oldVnode = preact__WEBPACK_IMPORTED_MODULE_0__.options.vnode;
preact__WEBPACK_IMPORTED_MODULE_0__.options.vnode = vnode => {
  if (vnode && typeof vnode.type === 'function') {
    const vnodes = _vnodesForComponent__WEBPACK_IMPORTED_MODULE_1__.vnodesForComponent.get(vnode.type);
    if (!vnodes) {
      _vnodesForComponent__WEBPACK_IMPORTED_MODULE_1__.vnodesForComponent.set(vnode.type, [vnode]);
    } else {
      vnodes.push(vnode);
    }

    const foundType = getMappedVnode(vnode.type);
    if (foundType !== vnode.type) {
      const vnodes = _vnodesForComponent__WEBPACK_IMPORTED_MODULE_1__.vnodesForComponent.get(foundType);
      if (!vnodes) {
        _vnodesForComponent__WEBPACK_IMPORTED_MODULE_1__.vnodesForComponent.set(foundType, [vnode]);
      } else {
        vnodes.push(vnode);
      }
    }

    vnode.type = foundType;
    if (
      vnode[_constants__WEBPACK_IMPORTED_MODULE_2__.VNODE_COMPONENT] &&
      'prototype' in vnode.type &&
      vnode.type.prototype.render
    ) {
      vnode[_constants__WEBPACK_IMPORTED_MODULE_2__.VNODE_COMPONENT].constructor = vnode.type;
    }
  }

  if (oldVnode) oldVnode(vnode);
};

const oldDiffed = preact__WEBPACK_IMPORTED_MODULE_0__.options.diffed;
preact__WEBPACK_IMPORTED_MODULE_0__.options.diffed = vnode => {
  if (vnode && typeof vnode.type === 'function') {
    const vnodes = _vnodesForComponent__WEBPACK_IMPORTED_MODULE_1__.vnodesForComponent.get(vnode.type);
    if (vnodes) {
      const matchingDom = vnodes.filter(p => p.__c === vnode.__c);
      if (matchingDom.length > 1) {
        const i = vnodes.findIndex(p => p === matchingDom[0]);
        vnodes.splice(i, 1);
      }
    }
  }

  if (oldDiffed) oldDiffed(vnode);
};


}),
"../../../../../../node_modules/@prefresh/core/src/runtime/vnodesForComponent.js": (function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {
"use strict";
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  mappedVNodes: function() { return mappedVNodes; },
  vnodesForComponent: function() { return vnodesForComponent; }
});
// all vnodes referencing a given constructor
const vnodesForComponent = new WeakMap();
const mappedVNodes = new WeakMap();


}),
"../../../../../../node_modules/@prefresh/utils/dist/src/index.js": (function (__unused_webpack_module, exports) {
const compareSignatures = (prev, next) => {
  const prevSignature = self.__PREFRESH__.getSignature(prev) || {};
  const nextSignature = self.__PREFRESH__.getSignature(next) || {};

  if (
    prevSignature.key !== nextSignature.key ||
    self.__PREFRESH__.computeKey(prevSignature) !==
      self.__PREFRESH__.computeKey(nextSignature) ||
    nextSignature.forceReset
  ) {
    self.__PREFRESH__.replaceComponent(prev, next, true);
  } else {
    self.__PREFRESH__.replaceComponent(prev, next, false);
  }
};

const flush = () => {
  const pending = [...self.__PREFRESH__.getPendingUpdates()];
  self.__PREFRESH__.flush();

  if (pending.length > 0) {
    pending.forEach(([prev, next]) => {
      compareSignatures(prev, next);
    });
  }
};

const isComponent = exportValue => {
  if (typeof exportValue === 'function') {
    if (
      exportValue.prototype != null &&
      exportValue.prototype.isReactComponent
    ) {
      return true;
    }

    const name = exportValue.name || exportValue.displayName;
    return (
      typeof name === 'string' && name[0] && name[0] == name[0].toUpperCase()
    );
  }
  return false;
};

exports.flush=flush;
exports.isComponent=isComponent;

}),
"../../../../../../node_modules/preact/compat/client.js": (function (__unused_webpack_module, exports, __webpack_require__) {
const { render, hydrate, unmountComponentAtNode } = __webpack_require__(/*! preact/compat */ "../../../../../../node_modules/preact/compat/dist/compat.js");

function createRoot(container) {
	return {
		// eslint-disable-next-line
		render: function (children) {
			render(children, container);
		},
		// eslint-disable-next-line
		unmount: function () {
			unmountComponentAtNode(container);
		}
	};
}

exports.createRoot = createRoot;

exports.hydrateRoot = function (container, children) {
	hydrate(children, container);
	return createRoot(container);
};


}),
"../../../../../../node_modules/preact/compat/dist/compat.js": (function (__unused_webpack_module, exports, __webpack_require__) {
var n=__webpack_require__(/*! preact */ "../../../../../../node_modules/preact/dist/preact.js"),t=__webpack_require__(/*! preact/hooks */ "../../../../../../node_modules/preact/hooks/dist/hooks.js");function e(n,t){for(var e in t)n[e]=t[e];return n}function r(n,t){for(var e in n)if("__source"!==e&&!(e in t))return!0;for(var r in t)if("__source"!==r&&n[r]!==t[r])return!0;return!1}function u(n,t){this.props=n,this.context=t}function o(t,e){function u(n){var t=this.props.ref,u=t==n.ref;return!u&&t&&(t.call?t(null):t.current=null),e?!e(this.props,n)||!u:r(this.props,n)}function o(e){return this.shouldComponentUpdate=u,n.createElement(t,e)}return o.displayName="Memo("+(t.displayName||t.name)+")",o.prototype.isReactComponent=!0,o.__f=!0,o}(u.prototype=new n.Component).isPureReactComponent=!0,u.prototype.shouldComponentUpdate=function(n,t){return r(this.props,n)||r(this.state,t)};var i=n.options.__b;n.options.__b=function(n){n.type&&n.type.__f&&n.ref&&(n.props.ref=n.ref,n.ref=null),i&&i(n)};var c="undefined"!=typeof Symbol&&Symbol.for&&Symbol.for("react.forward_ref")||3911;function l(n){function t(t){var r=e({},t);return delete r.ref,n(r,t.ref||null)}return t.$$typeof=c,t.render=t,t.prototype.isReactComponent=t.__f=!0,t.displayName="ForwardRef("+(n.displayName||n.name)+")",t}var f=function(t,e){return null==t?null:n.toChildArray(n.toChildArray(t).map(e))},a={map:f,forEach:f,count:function(t){return t?n.toChildArray(t).length:0},only:function(t){var e=n.toChildArray(t);if(1!==e.length)throw"Children.only";return e[0]},toArray:n.toChildArray},s=n.options.__e;n.options.__e=function(n,t,e,r){if(n.then)for(var u,o=t;o=o.__;)if((u=o.__c)&&u.__c)return null==t.__e&&(t.__e=e.__e,t.__k=e.__k),u.__c(n,t);s(n,t,e,r)};var p=n.options.unmount;function h(n,t,r){return n&&(n.__c&&n.__c.__H&&(n.__c.__H.__.forEach(function(n){"function"==typeof n.__c&&n.__c()}),n.__c.__H=null),null!=(n=e({},n)).__c&&(n.__c.__P===r&&(n.__c.__P=t),n.__c=null),n.__k=n.__k&&n.__k.map(function(n){return h(n,t,r)})),n}function v(n,t,e){return n&&e&&(n.__v=null,n.__k=n.__k&&n.__k.map(function(n){return v(n,t,e)}),n.__c&&n.__c.__P===t&&(n.__e&&e.appendChild(n.__e),n.__c.__e=!0,n.__c.__P=e)),n}function d(){this.__u=0,this.t=null,this.__b=null}function m(n){var t=n.__.__c;return t&&t.__a&&t.__a(n)}function x(t){var e,r,u;function o(o){if(e||(e=t()).then(function(n){r=n.default||n},function(n){u=n}),u)throw u;if(!r)throw e;return n.createElement(r,o)}return o.displayName="Lazy",o.__f=!0,o}function b(){this.u=null,this.o=null}n.options.unmount=function(n){var t=n.__c;t&&t.__R&&t.__R(),t&&32&n.__u&&(n.type=null),p&&p(n)},(d.prototype=new n.Component).__c=function(n,t){var e=t.__c,r=this;null==r.t&&(r.t=[]),r.t.push(e);var u=m(r.__v),o=!1,i=function(){o||(o=!0,e.__R=null,u?u(c):c())};e.__R=i;var c=function(){if(!--r.__u){if(r.state.__a){var n=r.state.__a;r.__v.__k[0]=v(n,n.__c.__P,n.__c.__O)}var t;for(r.setState({__a:r.__b=null});t=r.t.pop();)t.forceUpdate()}};r.__u++||32&t.__u||r.setState({__a:r.__b=r.__v.__k[0]}),n.then(i,i)},d.prototype.componentWillUnmount=function(){this.t=[]},d.prototype.render=function(t,e){if(this.__b){if(this.__v.__k){var r=document.createElement("div"),u=this.__v.__k[0].__c;this.__v.__k[0]=h(this.__b,r,u.__O=u.__P)}this.__b=null}var o=e.__a&&n.createElement(n.Fragment,null,t.fallback);return o&&(o.__u&=-33),[n.createElement(n.Fragment,null,e.__a?null:t.children),o]};var y=function(n,t,e){if(++e[1]===e[0]&&n.o.delete(t),n.props.revealOrder&&("t"!==n.props.revealOrder[0]||!n.o.size))for(e=n.u;e;){for(;e.length>3;)e.pop()();if(e[1]<e[0])break;n.u=e=e[2]}};function _(n){return this.getChildContext=function(){return n.context},n.children}function g(t){var e=this,r=t.i;e.componentWillUnmount=function(){n.render(null,e.l),e.l=null,e.i=null},e.i&&e.i!==r&&e.componentWillUnmount(),e.l||(e.i=r,e.l={nodeType:1,parentNode:r,childNodes:[],appendChild:function(n){this.childNodes.push(n),e.i.appendChild(n)},insertBefore:function(n,t){this.childNodes.push(n),e.i.appendChild(n)},removeChild:function(n){this.childNodes.splice(this.childNodes.indexOf(n)>>>1,1),e.i.removeChild(n)}}),n.render(n.createElement(_,{context:e.context},t.__v),e.l)}function S(t,e){var r=n.createElement(g,{__v:t,i:e});return r.containerInfo=e,r}(b.prototype=new n.Component).__a=function(n){var t=this,e=m(t.__v),r=t.o.get(n);return r[0]++,function(u){var o=function(){t.props.revealOrder?(r.push(u),y(t,n,r)):u()};e?e(o):o()}},b.prototype.render=function(t){this.u=null,this.o=new Map;var e=n.toChildArray(t.children);t.revealOrder&&"b"===t.revealOrder[0]&&e.reverse();for(var r=e.length;r--;)this.o.set(e[r],this.u=[1,0,this.u]);return t.children},b.prototype.componentDidUpdate=b.prototype.componentDidMount=function(){var n=this;this.o.forEach(function(t,e){y(n,e,t)})};var E="undefined"!=typeof Symbol&&Symbol.for&&Symbol.for("react.element")||60103,C=/^(?:accent|alignment|arabic|baseline|cap|clip(?!PathU)|color|dominant|fill|flood|font|glyph(?!R)|horiz|image(!S)|letter|lighting|marker(?!H|W|U)|overline|paint|pointer|shape|stop|strikethrough|stroke|text(?!L)|transform|underline|unicode|units|v|vector|vert|word|writing|x(?!C))[A-Z]/,O=/^on(Ani|Tra|Tou|BeforeInp|Compo)/,R=/[A-Z0-9]/g,w="undefined"!=typeof document,j=function(n){return("undefined"!=typeof Symbol&&"symbol"==typeof Symbol()?/fil|che|rad/:/fil|che|ra/).test(n)};function I(t,e,r){return null==e.__k&&(e.textContent=""),n.render(t,e),"function"==typeof r&&r(),t?t.__c:null}function N(t,e,r){return n.hydrate(t,e),"function"==typeof r&&r(),t?t.__c:null}n.Component.prototype.isReactComponent={},["componentWillMount","componentWillReceiveProps","componentWillUpdate"].forEach(function(t){Object.defineProperty(n.Component.prototype,t,{configurable:!0,get:function(){return this["UNSAFE_"+t]},set:function(n){Object.defineProperty(this,t,{configurable:!0,writable:!0,value:n})}})});var k=n.options.event;function M(){}function T(){return this.cancelBubble}function A(){return this.defaultPrevented}n.options.event=function(n){return k&&(n=k(n)),n.persist=M,n.isPropagationStopped=T,n.isDefaultPrevented=A,n.nativeEvent=n};var D,L={enumerable:!1,configurable:!0,get:function(){return this.class}},F=n.options.vnode;n.options.vnode=function(t){"string"==typeof t.type&&function(t){var e=t.props,r=t.type,u={};for(var o in e){var i=e[o];if(!("value"===o&&"defaultValue"in e&&null==i||w&&"children"===o&&"noscript"===r||"class"===o||"className"===o)){var c=o.toLowerCase();"defaultValue"===o&&"value"in e&&null==e.value?o="value":"download"===o&&!0===i?i="":"translate"===c&&"no"===i?i=!1:"ondoubleclick"===c?o="ondblclick":"onchange"!==c||"input"!==r&&"textarea"!==r||j(e.type)?"onfocus"===c?o="onfocusin":"onblur"===c?o="onfocusout":O.test(o)?o=c:-1===r.indexOf("-")&&C.test(o)?o=o.replace(R,"-$&").toLowerCase():null===i&&(i=void 0):c=o="oninput","oninput"===c&&u[o=c]&&(o="oninputCapture"),u[o]=i}}"select"==r&&u.multiple&&Array.isArray(u.value)&&(u.value=n.toChildArray(e.children).forEach(function(n){n.props.selected=-1!=u.value.indexOf(n.props.value)})),"select"==r&&null!=u.defaultValue&&(u.value=n.toChildArray(e.children).forEach(function(n){n.props.selected=u.multiple?-1!=u.defaultValue.indexOf(n.props.value):u.defaultValue==n.props.value})),e.class&&!e.className?(u.class=e.class,Object.defineProperty(u,"className",L)):(e.className&&!e.class||e.class&&e.className)&&(u.class=u.className=e.className),t.props=u}(t),t.$$typeof=E,F&&F(t)};var U=n.options.__r;n.options.__r=function(n){U&&U(n),D=n.__c};var V=n.options.diffed;n.options.diffed=function(n){V&&V(n);var t=n.props,e=n.__e;null!=e&&"textarea"===n.type&&"value"in t&&t.value!==e.value&&(e.value=null==t.value?"":t.value),D=null};var W={ReactCurrentDispatcher:{current:{readContext:function(n){return D.__n[n.__c].props.value},useCallback:t.useCallback,useContext:t.useContext,useDebugValue:t.useDebugValue,useDeferredValue:Q,useEffect:t.useEffect,useId:t.useId,useImperativeHandle:t.useImperativeHandle,useInsertionEffect:nn,useLayoutEffect:t.useLayoutEffect,useMemo:t.useMemo,useReducer:t.useReducer,useRef:t.useRef,useState:t.useState,useSyncExternalStore:en,useTransition:X}}};function P(t){return n.createElement.bind(null,t)}function z(n){return!!n&&n.$$typeof===E}function B(t){return z(t)&&t.type===n.Fragment}function H(n){return!!n&&!!n.displayName&&("string"==typeof n.displayName||n.displayName instanceof String)&&n.displayName.startsWith("Memo(")}function q(t){return z(t)?n.cloneElement.apply(null,arguments):t}function Z(t){return!!t.__k&&(n.render(null,t),!0)}function Y(n){return n&&(n.base||1===n.nodeType&&n)||null}var $=function(n,t){return n(t)},G=function(n,t){return n(t)},J=n.Fragment;function K(n){n()}function Q(n){return n}function X(){return[!1,K]}var nn=t.useLayoutEffect,tn=z;function en(n,e){var r=e(),u=t.useState({p:{__:r,h:e}}),o=u[0].p,i=u[1];return t.useLayoutEffect(function(){o.__=r,o.h=e,rn(o)&&i({p:o})},[n,r,e]),t.useEffect(function(){return rn(o)&&i({p:o}),n(function(){rn(o)&&i({p:o})})},[n]),r}function rn(n){var t,e,r=n.h,u=n.__;try{var o=r();return!((t=u)===(e=o)&&(0!==t||1/t==1/e)||t!=t&&e!=e)}catch(n){return!0}}var un={useState:t.useState,useId:t.useId,useReducer:t.useReducer,useEffect:t.useEffect,useLayoutEffect:t.useLayoutEffect,useInsertionEffect:nn,useTransition:X,useDeferredValue:Q,useSyncExternalStore:en,startTransition:K,useRef:t.useRef,useImperativeHandle:t.useImperativeHandle,useMemo:t.useMemo,useCallback:t.useCallback,useContext:t.useContext,useDebugValue:t.useDebugValue,version:"17.0.2",Children:a,render:I,hydrate:N,unmountComponentAtNode:Z,createPortal:S,createElement:n.createElement,createContext:n.createContext,createFactory:P,cloneElement:q,createRef:n.createRef,Fragment:n.Fragment,isValidElement:z,isElement:tn,isFragment:B,isMemo:H,findDOMNode:Y,Component:n.Component,PureComponent:u,memo:o,forwardRef:l,flushSync:G,unstable_batchedUpdates:$,StrictMode:J,Suspense:d,SuspenseList:b,lazy:x,__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED:W};Object.defineProperty(exports, "Component", ({enumerable:!0,get:function(){return n.Component}})),Object.defineProperty(exports, "Fragment", ({enumerable:!0,get:function(){return n.Fragment}})),Object.defineProperty(exports, "createContext", ({enumerable:!0,get:function(){return n.createContext}})),Object.defineProperty(exports, "createElement", ({enumerable:!0,get:function(){return n.createElement}})),Object.defineProperty(exports, "createRef", ({enumerable:!0,get:function(){return n.createRef}})),exports.Children=a,exports.PureComponent=u,exports.StrictMode=J,exports.Suspense=d,exports.SuspenseList=b,exports.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED=W,exports.cloneElement=q,exports.createFactory=P,exports.createPortal=S,exports["default"]=un,exports.findDOMNode=Y,exports.flushSync=G,exports.forwardRef=l,exports.hydrate=N,exports.isElement=tn,exports.isFragment=B,exports.isMemo=H,exports.isValidElement=z,exports.lazy=x,exports.memo=o,exports.render=I,exports.startTransition=K,exports.unmountComponentAtNode=Z,exports.unstable_batchedUpdates=$,exports.useDeferredValue=Q,exports.useInsertionEffect=nn,exports.useSyncExternalStore=en,exports.useTransition=X,exports.version="17.0.2",Object.keys(t).forEach(function(n){"default"===n||exports.hasOwnProperty(n)||Object.defineProperty(exports,n,{enumerable:!0,get:function(){return t[n]}})});
//# sourceMappingURL=compat.js.map


}),
"../../../../../../node_modules/preact/compat/jsx-dev-runtime.js": (function (module, __unused_webpack_exports, __webpack_require__) {
__webpack_require__(/*! preact/compat */ "../../../../../../node_modules/preact/compat/dist/compat.js");

module.exports = __webpack_require__(/*! preact/jsx-runtime */ "../../../../../../node_modules/preact/jsx-runtime/dist/jsxRuntime.js");


}),
"../../../../../../node_modules/preact/dist/preact.js": (function (__unused_webpack_module, exports) {
var n,l,t,u,i,o,r,e,f,c,s,a,h={},p=[],v=/acit|ex(?:s|g|n|p|$)|rph|grid|ows|mnc|ntw|ine[ch]|zoo|^ord|itera/i,y=Array.isArray;function d(n,l){for(var t in l)n[t]=l[t];return n}function w(n){var l=n.parentNode;l&&l.removeChild(n)}function _(l,t,u){var i,o,r,e={};for(r in t)"key"==r?i=t[r]:"ref"==r?o=t[r]:e[r]=t[r];if(arguments.length>2&&(e.children=arguments.length>3?n.call(arguments,2):u),"function"==typeof l&&null!=l.defaultProps)for(r in l.defaultProps)void 0===e[r]&&(e[r]=l.defaultProps[r]);return g(l,e,i,o,null)}function g(n,u,i,o,r){var e={type:n,props:u,key:i,ref:o,__k:null,__:null,__b:0,__e:null,__d:void 0,__c:null,constructor:void 0,__v:null==r?++t:r,__i:-1,__u:0};return null==r&&null!=l.vnode&&l.vnode(e),e}function x(n){return n.children}function k(n,l){this.props=n,this.context=l}function m(n,l){if(null==l)return n.__?m(n.__,n.__i+1):null;for(var t;l<n.__k.length;l++)if(null!=(t=n.__k[l])&&null!=t.__e)return t.__e;return"function"==typeof n.type?m(n):null}function b(n){var l,t;if(null!=(n=n.__)&&null!=n.__c){for(n.__e=n.__c.base=null,l=0;l<n.__k.length;l++)if(null!=(t=n.__k[l])&&null!=t.__e){n.__e=n.__c.base=t.__e;break}return b(n)}}function C(n){(!n.__d&&(n.__d=!0)&&i.push(n)&&!M.__r++||o!==l.debounceRendering)&&((o=l.debounceRendering)||r)(M)}function M(){var n,t,u,o,r,f,c,s;for(i.sort(e);n=i.shift();)n.__d&&(t=i.length,o=void 0,f=(r=(u=n).__v).__e,c=[],s=[],u.__P&&((o=d({},r)).__v=r.__v+1,l.vnode&&l.vnode(o),A(u.__P,o,r,u.__n,u.__P.namespaceURI,32&r.__u?[f]:null,c,null==f?m(r):f,!!(32&r.__u),s),o.__v=r.__v,o.__.__k[o.__i]=o,F(c,o,s),o.__e!=f&&b(o)),i.length>t&&i.sort(e));M.__r=0}function P(n,l,t,u,i,o,r,e,f,c,s){var a,v,y,d,w,_=u&&u.__k||p,g=l.length;for(t.__d=f,S(t,l,_),f=t.__d,a=0;a<g;a++)null!=(y=t.__k[a])&&"boolean"!=typeof y&&"function"!=typeof y&&(v=-1===y.__i?h:_[y.__i]||h,y.__i=a,A(n,y,v,i,o,r,e,f,c,s),d=y.__e,y.ref&&v.ref!=y.ref&&(v.ref&&j(v.ref,null,y),s.push(y.ref,y.__c||d,y)),null==w&&null!=d&&(w=d),65536&y.__u||v.__k===y.__k?(f&&!f.isConnected&&(f=m(v)),f=$(y,f,n)):"function"==typeof y.type&&void 0!==y.__d?f=y.__d:d&&(f=d.nextSibling),y.__d=void 0,y.__u&=-196609);t.__d=f,t.__e=w}function S(n,l,t){var u,i,o,r,e,f=l.length,c=t.length,s=c,a=0;for(n.__k=[],u=0;u<f;u++)r=u+a,null!=(i=n.__k[u]=null==(i=l[u])||"boolean"==typeof i||"function"==typeof i?null:"string"==typeof i||"number"==typeof i||"bigint"==typeof i||i.constructor==String?g(null,i,null,null,null):y(i)?g(x,{children:i},null,null,null):void 0===i.constructor&&i.__b>0?g(i.type,i.props,i.key,i.ref?i.ref:null,i.__v):i)?(i.__=n,i.__b=n.__b+1,e=I(i,t,r,s),i.__i=e,o=null,-1!==e&&(s--,(o=t[e])&&(o.__u|=131072)),null==o||null===o.__v?(-1==e&&a--,"function"!=typeof i.type&&(i.__u|=65536)):e!==r&&(e===r+1?a++:e>r?s>f-r?a+=e-r:a--:e<r?e==r-1&&(a=e-r):a=0,e!==u+a&&(i.__u|=65536))):(o=t[r])&&null==o.key&&o.__e&&0==(131072&o.__u)&&(o.__e==n.__d&&(n.__d=m(o)),z(o,o,!1),t[r]=null,s--);if(s)for(u=0;u<c;u++)null!=(o=t[u])&&0==(131072&o.__u)&&(o.__e==n.__d&&(n.__d=m(o)),z(o,o))}function $(n,l,t){var u,i;if("function"==typeof n.type){for(u=n.__k,i=0;u&&i<u.length;i++)u[i]&&(u[i].__=n,l=$(u[i],l,t));return l}n.__e!=l&&(t.insertBefore(n.__e,l||null),l=n.__e);do{l=l&&l.nextSibling}while(null!=l&&8===l.nodeType);return l}function I(n,l,t,u){var i=n.key,o=n.type,r=t-1,e=t+1,f=l[t];if(null===f||f&&i==f.key&&o===f.type&&0==(131072&f.__u))return t;if(u>(null!=f&&0==(131072&f.__u)?1:0))for(;r>=0||e<l.length;){if(r>=0){if((f=l[r])&&0==(131072&f.__u)&&i==f.key&&o===f.type)return r;r--}if(e<l.length){if((f=l[e])&&0==(131072&f.__u)&&i==f.key&&o===f.type)return e;e++}}return-1}function H(n,l,t){"-"===l[0]?n.setProperty(l,null==t?"":t):n[l]=null==t?"":"number"!=typeof t||v.test(l)?t:t+"px"}function L(n,l,t,u,i){var o;n:if("style"===l)if("string"==typeof t)n.style.cssText=t;else{if("string"==typeof u&&(n.style.cssText=u=""),u)for(l in u)t&&l in t||H(n.style,l,"");if(t)for(l in t)u&&t[l]===u[l]||H(n.style,l,t[l])}else if("o"===l[0]&&"n"===l[1])o=l!==(l=l.replace(/(PointerCapture)$|Capture$/i,"$1")),l=l.toLowerCase()in n||"onFocusOut"===l||"onFocusIn"===l?l.toLowerCase().slice(2):l.slice(2),n.l||(n.l={}),n.l[l+o]=t,t?u?t.t=u.t:(t.t=f,n.addEventListener(l,o?s:c,o)):n.removeEventListener(l,o?s:c,o);else{if("http://www.w3.org/2000/svg"==i)l=l.replace(/xlink(H|:h)/,"h").replace(/sName$/,"s");else if("width"!=l&&"height"!=l&&"href"!=l&&"list"!=l&&"form"!=l&&"tabIndex"!=l&&"download"!=l&&"rowSpan"!=l&&"colSpan"!=l&&"role"!=l&&l in n)try{n[l]=null==t?"":t;break n}catch(n){}"function"==typeof t||(null==t||!1===t&&"-"!==l[4]?n.removeAttribute(l):n.setAttribute(l,t))}}function T(n){return function(t){if(this.l){var u=this.l[t.type+n];if(null==t.u)t.u=f++;else if(t.u<u.t)return;return u(l.event?l.event(t):t)}}}function A(n,t,u,i,o,r,e,f,c,s){var a,h,p,v,w,_,g,m,b,C,M,S,$,I,H,L=t.type;if(void 0!==t.constructor)return null;128&u.__u&&(c=!!(32&u.__u),r=[f=t.__e=u.__e]),(a=l.__b)&&a(t);n:if("function"==typeof L)try{if(m=t.props,b=(a=L.contextType)&&i[a.__c],C=a?b?b.props.value:a.__:i,u.__c?g=(h=t.__c=u.__c).__=h.__E:("prototype"in L&&L.prototype.render?t.__c=h=new L(m,C):(t.__c=h=new k(m,C),h.constructor=L,h.render=N),b&&b.sub(h),h.props=m,h.state||(h.state={}),h.context=C,h.__n=i,p=h.__d=!0,h.__h=[],h._sb=[]),null==h.__s&&(h.__s=h.state),null!=L.getDerivedStateFromProps&&(h.__s==h.state&&(h.__s=d({},h.__s)),d(h.__s,L.getDerivedStateFromProps(m,h.__s))),v=h.props,w=h.state,h.__v=t,p)null==L.getDerivedStateFromProps&&null!=h.componentWillMount&&h.componentWillMount(),null!=h.componentDidMount&&h.__h.push(h.componentDidMount);else{if(null==L.getDerivedStateFromProps&&m!==v&&null!=h.componentWillReceiveProps&&h.componentWillReceiveProps(m,C),!h.__e&&(null!=h.shouldComponentUpdate&&!1===h.shouldComponentUpdate(m,h.__s,C)||t.__v===u.__v)){for(t.__v!==u.__v&&(h.props=m,h.state=h.__s,h.__d=!1),t.__e=u.__e,t.__k=u.__k,t.__k.forEach(function(n){n&&(n.__=t)}),M=0;M<h._sb.length;M++)h.__h.push(h._sb[M]);h._sb=[],h.__h.length&&e.push(h);break n}null!=h.componentWillUpdate&&h.componentWillUpdate(m,h.__s,C),null!=h.componentDidUpdate&&h.__h.push(function(){h.componentDidUpdate(v,w,_)})}if(h.context=C,h.props=m,h.__P=n,h.__e=!1,S=l.__r,$=0,"prototype"in L&&L.prototype.render){for(h.state=h.__s,h.__d=!1,S&&S(t),a=h.render(h.props,h.state,h.context),I=0;I<h._sb.length;I++)h.__h.push(h._sb[I]);h._sb=[]}else do{h.__d=!1,S&&S(t),a=h.render(h.props,h.state,h.context),h.state=h.__s}while(h.__d&&++$<25);h.state=h.__s,null!=h.getChildContext&&(i=d(d({},i),h.getChildContext())),p||null==h.getSnapshotBeforeUpdate||(_=h.getSnapshotBeforeUpdate(v,w)),P(n,y(H=null!=a&&a.type===x&&null==a.key?a.props.children:a)?H:[H],t,u,i,o,r,e,f,c,s),h.base=t.__e,t.__u&=-161,h.__h.length&&e.push(h),g&&(h.__E=h.__=null)}catch(n){t.__v=null,c||null!=r?(t.__e=f,t.__u|=c?160:32,r[r.indexOf(f)]=null):(t.__e=u.__e,t.__k=u.__k),l.__e(n,t,u)}else null==r&&t.__v===u.__v?(t.__k=u.__k,t.__e=u.__e):t.__e=O(u.__e,t,u,i,o,r,e,c,s);(a=l.diffed)&&a(t)}function F(n,t,u){t.__d=void 0;for(var i=0;i<u.length;i++)j(u[i],u[++i],u[++i]);l.__c&&l.__c(t,n),n.some(function(t){try{n=t.__h,t.__h=[],n.some(function(n){n.call(t)})}catch(n){l.__e(n,t.__v)}})}function O(l,t,u,i,o,r,e,f,c){var s,a,p,v,d,_,g,x=u.props,k=t.props,b=t.type;if("svg"===b?o="http://www.w3.org/2000/svg":"math"===b?o="http://www.w3.org/1998/Math/MathML":o||(o="http://www.w3.org/1999/xhtml"),null!=r)for(s=0;s<r.length;s++)if((d=r[s])&&"setAttribute"in d==!!b&&(b?d.localName===b:3===d.nodeType)){l=d,r[s]=null;break}if(null==l){if(null===b)return document.createTextNode(k);l=document.createElementNS(o,b,k.is&&k),r=null,f=!1}if(null===b)x===k||f&&l.data===k||(l.data=k);else{if(r=r&&n.call(l.childNodes),x=u.props||h,!f&&null!=r)for(x={},s=0;s<l.attributes.length;s++)x[(d=l.attributes[s]).name]=d.value;for(s in x)if(d=x[s],"children"==s);else if("dangerouslySetInnerHTML"==s)p=d;else if("key"!==s&&!(s in k)){if("value"==s&&"defaultValue"in k||"checked"==s&&"defaultChecked"in k)continue;L(l,s,null,d,o)}for(s in k)d=k[s],"children"==s?v=d:"dangerouslySetInnerHTML"==s?a=d:"value"==s?_=d:"checked"==s?g=d:"key"===s||f&&"function"!=typeof d||x[s]===d||L(l,s,d,x[s],o);if(a)f||p&&(a.__html===p.__html||a.__html===l.innerHTML)||(l.innerHTML=a.__html),t.__k=[];else if(p&&(l.innerHTML=""),P(l,y(v)?v:[v],t,u,i,"foreignObject"===b?"http://www.w3.org/1999/xhtml":o,r,e,r?r[0]:u.__k&&m(u,0),f,c),null!=r)for(s=r.length;s--;)null!=r[s]&&w(r[s]);f||(s="value",void 0!==_&&(_!==l[s]||"progress"===b&&!_||"option"===b&&_!==x[s])&&L(l,s,_,x[s],o),s="checked",void 0!==g&&g!==l[s]&&L(l,s,g,x[s],o))}return l}function j(n,t,u){try{"function"==typeof n?n(t):n.current=t}catch(n){l.__e(n,u)}}function z(n,t,u){var i,o;if(l.unmount&&l.unmount(n),(i=n.ref)&&(i.current&&i.current!==n.__e||j(i,null,t)),null!=(i=n.__c)){if(i.componentWillUnmount)try{i.componentWillUnmount()}catch(n){l.__e(n,t)}i.base=i.__P=null}if(i=n.__k)for(o=0;o<i.length;o++)i[o]&&z(i[o],t,u||"function"!=typeof n.type);u||null==n.__e||w(n.__e),n.__c=n.__=n.__e=n.__d=void 0}function N(n,l,t){return this.constructor(n,t)}function V(t,u,i){var o,r,e,f;l.__&&l.__(t,u),r=(o="function"==typeof i)?null:i&&i.__k||u.__k,e=[],f=[],A(u,t=(!o&&i||u).__k=_(x,null,[t]),r||h,h,u.namespaceURI,!o&&i?[i]:r?null:u.firstChild?n.call(u.childNodes):null,e,!o&&i?i:r?r.__e:u.firstChild,o,f),F(e,t,f)}n=p.slice,l={__e:function(n,l,t,u){for(var i,o,r;l=l.__;)if((i=l.__c)&&!i.__)try{if((o=i.constructor)&&null!=o.getDerivedStateFromError&&(i.setState(o.getDerivedStateFromError(n)),r=i.__d),null!=i.componentDidCatch&&(i.componentDidCatch(n,u||{}),r=i.__d),r)return i.__E=i}catch(l){n=l}throw n}},t=0,u=function(n){return null!=n&&null==n.constructor},k.prototype.setState=function(n,l){var t;t=null!=this.__s&&this.__s!==this.state?this.__s:this.__s=d({},this.state),"function"==typeof n&&(n=n(d({},t),this.props)),n&&d(t,n),null!=n&&this.__v&&(l&&this._sb.push(l),C(this))},k.prototype.forceUpdate=function(n){this.__v&&(this.__e=!0,n&&this.__h.push(n),C(this))},k.prototype.render=x,i=[],r="function"==typeof Promise?Promise.prototype.then.bind(Promise.resolve()):setTimeout,e=function(n,l){return n.__v.__b-l.__v.__b},M.__r=0,f=0,c=T(!1),s=T(!0),a=0,exports.Component=k,exports.Fragment=x,exports.cloneElement=function(l,t,u){var i,o,r,e,f=d({},l.props);for(r in l.type&&l.type.defaultProps&&(e=l.type.defaultProps),t)"key"==r?i=t[r]:"ref"==r?o=t[r]:f[r]=void 0===t[r]&&void 0!==e?e[r]:t[r];return arguments.length>2&&(f.children=arguments.length>3?n.call(arguments,2):u),g(l.type,f,i||l.key,o||l.ref,null)},exports.createContext=function(n,l){var t={__c:l="__cC"+a++,__:n,Consumer:function(n,l){return n.children(l)},Provider:function(n){var t,u;return this.getChildContext||(t=[],(u={})[l]=this,this.getChildContext=function(){return u},this.shouldComponentUpdate=function(n){this.props.value!==n.value&&t.some(function(n){n.__e=!0,C(n)})},this.sub=function(n){t.push(n);var l=n.componentWillUnmount;n.componentWillUnmount=function(){t.splice(t.indexOf(n),1),l&&l.call(n)}}),n.children}};return t.Provider.__=t.Consumer.contextType=t},exports.createElement=_,exports.createRef=function(){return{current:null}},exports.h=_,exports.hydrate=function n(l,t){V(l,t,n)},exports.isValidElement=u,exports.options=l,exports.render=V,exports.toChildArray=function n(l,t){return t=t||[],null==l||"boolean"==typeof l||(y(l)?l.some(function(l){n(l,t)}):t.push(l)),t};
//# sourceMappingURL=preact.js.map


}),
"../../../../../../node_modules/preact/hooks/dist/hooks.js": (function (__unused_webpack_module, exports, __webpack_require__) {
var n,t,r,u,o=__webpack_require__(/*! preact */ "../../../../../../node_modules/preact/dist/preact.js"),i=0,f=[],c=[],e=o.options,a=e.__b,v=e.__r,s=e.diffed,l=e.__c,p=e.unmount,x=e.__;function d(n,r){e.__h&&e.__h(t,n,i||r),i=0;var u=t.__H||(t.__H={__:[],__h:[]});return n>=u.__.length&&u.__.push({__V:c}),u.__[n]}function m(n){return i=1,h(b,n)}function h(r,u,o){var i=d(n++,2);if(i.t=r,!i.__c&&(i.__=[o?o(u):b(void 0,u),function(n){var t=i.__N?i.__N[0]:i.__[0],r=i.t(t,n);t!==r&&(i.__N=[r,i.__[1]],i.__c.setState({}))}],i.__c=t,!t.u)){var f=function(n,t,r){if(!i.__c.__H)return!0;var u=i.__c.__H.__.filter(function(n){return!!n.__c});if(u.every(function(n){return!n.__N}))return!c||c.call(this,n,t,r);var o=!1;return u.forEach(function(n){if(n.__N){var t=n.__[0];n.__=n.__N,n.__N=void 0,t!==n.__[0]&&(o=!0)}}),!(!o&&i.__c.props===n)&&(!c||c.call(this,n,t,r))};t.u=!0;var c=t.shouldComponentUpdate,e=t.componentWillUpdate;t.componentWillUpdate=function(n,t,r){if(this.__e){var u=c;c=void 0,f(n,t,r),c=u}e&&e.call(this,n,t,r)},t.shouldComponentUpdate=f}return i.__N||i.__}function y(r,u){var o=d(n++,4);!e.__s&&V(o.__H,u)&&(o.__=r,o.o=u,t.__h.push(o))}function _(t,r){var u=d(n++,7);return V(u.__H,r)?(u.__V=t(),u.o=r,u.__h=t,u.__V):u.__}function q(){for(var n;n=f.shift();)if(n.__P&&n.__H)try{n.__H.__h.forEach(T),n.__H.__h.forEach(P),n.__H.__h=[]}catch(t){n.__H.__h=[],e.__e(t,n.__v)}}e.__b=function(n){t=null,a&&a(n)},e.__=function(n,t){n&&t.__k&&t.__k.__m&&(n.__m=t.__k.__m),x&&x(n,t)},e.__r=function(u){v&&v(u),n=0;var o=(t=u.__c).__H;o&&(r===t?(o.__h=[],t.__h=[],o.__.forEach(function(n){n.__N&&(n.__=n.__N),n.__V=c,n.__N=n.o=void 0})):(o.__h.forEach(T),o.__h.forEach(P),o.__h=[],n=0)),r=t},e.diffed=function(n){s&&s(n);var o=n.__c;o&&o.__H&&(o.__H.__h.length&&(1!==f.push(o)&&u===e.requestAnimationFrame||((u=e.requestAnimationFrame)||F)(q)),o.__H.__.forEach(function(n){n.o&&(n.__H=n.o),n.__V!==c&&(n.__=n.__V),n.o=void 0,n.__V=c})),r=t=null},e.__c=function(n,t){t.some(function(n){try{n.__h.forEach(T),n.__h=n.__h.filter(function(n){return!n.__||P(n)})}catch(r){t.some(function(n){n.__h&&(n.__h=[])}),t=[],e.__e(r,n.__v)}}),l&&l(n,t)},e.unmount=function(n){p&&p(n);var t,r=n.__c;r&&r.__H&&(r.__H.__.forEach(function(n){try{T(n)}catch(n){t=n}}),r.__H=void 0,t&&e.__e(t,r.__v))};var A="function"==typeof requestAnimationFrame;function F(n){var t,r=function(){clearTimeout(u),A&&cancelAnimationFrame(t),setTimeout(n)},u=setTimeout(r,100);A&&(t=requestAnimationFrame(r))}function T(n){var r=t,u=n.__c;"function"==typeof u&&(n.__c=void 0,u()),t=r}function P(n){var r=t;n.__c=n.__(),t=r}function V(n,t){return!n||n.length!==t.length||t.some(function(t,r){return t!==n[r]})}function b(n,t){return"function"==typeof t?t(n):t}exports.useCallback=function(n,t){return i=8,_(function(){return n},t)},exports.useContext=function(r){var u=t.context[r.__c],o=d(n++,9);return o.c=r,u?(null==o.__&&(o.__=!0,u.sub(t)),u.props.value):r.__},exports.useDebugValue=function(n,t){e.useDebugValue&&e.useDebugValue(t?t(n):n)},exports.useEffect=function(r,u){var o=d(n++,3);!e.__s&&V(o.__H,u)&&(o.__=r,o.o=u,t.__H.__h.push(o))},exports.useErrorBoundary=function(r){var u=d(n++,10),o=m();return u.__=r,t.componentDidCatch||(t.componentDidCatch=function(n,t){u.__&&u.__(n,t),o[1](n)}),[o[0],function(){o[1](void 0)}]},exports.useId=function(){var r=d(n++,11);if(!r.__){for(var u=t.__v;null!==u&&!u.__m&&null!==u.__;)u=u.__;var o=u.__m||(u.__m=[0,0]);r.__="P"+o[0]+"-"+o[1]++}return r.__},exports.useImperativeHandle=function(n,t,r){i=6,y(function(){return"function"==typeof n?(n(t()),function(){return n(null)}):n?(n.current=t(),function(){return n.current=null}):void 0},null==r?r:r.concat(n))},exports.useLayoutEffect=y,exports.useMemo=_,exports.useReducer=h,exports.useRef=function(n){return i=5,_(function(){return{current:n}},[])},exports.useState=m;
//# sourceMappingURL=hooks.js.map


}),
"../../../../../../node_modules/preact/jsx-runtime/dist/jsxRuntime.js": (function (__unused_webpack_module, exports, __webpack_require__) {
var r=__webpack_require__(/*! preact */ "../../../../../../node_modules/preact/dist/preact.js"),e=/["&<]/;function t(r){if(0===r.length||!1===e.test(r))return r;for(var t=0,n=0,o="",f="";n<r.length;n++){switch(r.charCodeAt(n)){case 34:f="&quot;";break;case 38:f="&amp;";break;case 60:f="&lt;";break;default:continue}n!==t&&(o+=r.slice(t,n)),o+=f,t=n+1}return n!==t&&(o+=r.slice(t,n)),o}var n=/acit|ex(?:s|g|n|p|$)|rph|grid|ows|mnc|ntw|ine[ch]|zoo|^ord|itera/i,o=0,f=Array.isArray;function i(e,t,n,f,i,u){t||(t={});var a,c,p=t;if("ref"in p)for(c in p={},t)"ref"==c?a=t[c]:p[c]=t[c];var l={type:e,props:p,key:n,ref:a,__k:null,__:null,__b:0,__e:null,__d:void 0,__c:null,constructor:void 0,__v:--o,__i:-1,__u:0,__source:i,__self:u};if("function"==typeof e&&(a=e.defaultProps))for(c in a)void 0===p[c]&&(p[c]=a[c]);return r.options.vnode&&r.options.vnode(l),l}var u={},a=/[A-Z]/g;Object.defineProperty(exports, "Fragment", ({enumerable:!0,get:function(){return r.Fragment}})),exports.jsx=i,exports.jsxAttr=function(e,o){if(r.options.attr){var f=r.options.attr(e,o);if("string"==typeof f)return f}if("ref"===e||"key"===e)return"";if("style"===e&&"object"==typeof o){var i="";for(var c in o){var p=o[c];if(null!=p&&""!==p){var l="-"==c[0]?c:u[c]||(u[c]=c.replace(a,"-$&").toLowerCase()),_=";";"number"!=typeof p||l.startsWith("--")||n.test(l)||(_="px;"),i=i+l+":"+p+_}}return e+'="'+i+'"'}return null==o||!1===o||"function"==typeof o||"object"==typeof o?"":!0===o?e:e+'="'+t(o)+'"'},exports.jsxDEV=i,exports.jsxEscape=function r(e){if(null==e||"boolean"==typeof e||"function"==typeof e)return null;if("object"==typeof e){if(void 0===e.constructor)return e;if(f(e)){for(var n=0;n<e.length;n++)e[n]=r(e[n]);return e}}return t(""+e)},exports.jsxTemplate=function(e){var t=i(r.Fragment,{tpl:e,exprs:[].slice.call(arguments,1)});return t.key=t.__v,t},exports.jsxs=i;
//# sourceMappingURL=jsxRuntime.js.map


}),
"./app.jsx": (function (module, __webpack_exports__, __webpack_require__) {
"use strict";
__webpack_require__.r(__webpack_exports__);
__webpack_require__.d(__webpack_exports__, {
  App: function() { return App; }
});
/* harmony import */var react_jsx_dev_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-dev-runtime */ "../../../../../../node_modules/preact/compat/jsx-dev-runtime.js");
/* harmony import */var preact_compat__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! preact/compat */ "../../../../../../node_modules/preact/compat/dist/compat.js");
/* harmony import */var preact_hooks__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! preact/hooks */ "../../../../../../node_modules/preact/hooks/dist/hooks.js");
/* module decorator */ module = __webpack_require__.hmd(module);
/* provided dependency */ var __prefresh_utils__ = __webpack_require__(/*! ../../../../client/prefresh.js */ "../../../../client/prefresh.js");

var _s = $RefreshSig$();


var Theme = (0,preact_compat__WEBPACK_IMPORTED_MODULE_1__.createContext)('light');
function Inner() {
    _s();
    var theme = (0,preact_hooks__WEBPACK_IMPORTED_MODULE_2__.useContext)(Theme);
    return /*#__PURE__*/ (0,react_jsx_dev_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxDEV)("div", {
        children: /*#__PURE__*/ (0,react_jsx_dev_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxDEV)("span", {
            children: [
                "after: ",
                theme
            ]
        }, void 0, true, {
            fileName: "/Users/bytedance/rspack-dev/rspack/packages/rspack-plugin-preact-refresh/tests/hotCases/hook/useContext#provide/app.jsx",
            lineNumber: 8,
            columnNumber: 16
        }, this)
    }, void 0, false, {
        fileName: "/Users/bytedance/rspack-dev/rspack/packages/rspack-plugin-preact-refresh/tests/hotCases/hook/useContext#provide/app.jsx",
        lineNumber: 8,
        columnNumber: 11
    }, this);
}
_s(Inner, "+C1P7ukOg/azcV4AZ819oyezFOE=");
_c = Inner;
function App() {
    return /*#__PURE__*/ (0,react_jsx_dev_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxDEV)(Theme.Provider, {
        value: "light",
        children: /*#__PURE__*/ (0,react_jsx_dev_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxDEV)(Inner, {}, void 0, false, {
            fileName: "/Users/bytedance/rspack-dev/rspack/packages/rspack-plugin-preact-refresh/tests/hotCases/hook/useContext#provide/app.jsx",
            lineNumber: 14,
            columnNumber: 7
        }, this)
    }, void 0, false, {
        fileName: "/Users/bytedance/rspack-dev/rspack/packages/rspack-plugin-preact-refresh/tests/hotCases/hook/useContext#provide/app.jsx",
        lineNumber: 13,
        columnNumber: 5
    }, this);
}
_c1 = App;
var _c, _c1;
$RefreshReg$(_c, "Inner");
$RefreshReg$(_c1, "App");

/**
 * The following code is modified based on
 * //https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/packages/webpack/src/loader/runtime.js
 *
 * MIT Licensed
 * Author JoviDeCroock
 * Copyright (c) 2021-Present Preact Team
 * https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/LICENSE
 */

const isPrefreshComponent = __prefresh_utils__.shouldBind(module);

// `@vanilla-extract/webpack` does some custom preprocessing where
// `module.hot` is partially replaced. This leads to our injected
// code being executed although it shouldn't be:
//
// Intermediate result:
//
//   if (true) { // <- inlined by intermediate compile step
//     const previousHotModuleExports = module.hot.data && ...
//                    // Crash happens here ---^
//
// It crashes at that line because some intermediate compiler isn't
// running in hot mode, but the overall guard condition was compiled
// down to being truthy. By moving `module.hot` outside of the
// condition of the if-statement, it will be left as is.
const moduleHot = module.hot;

if (moduleHot) {
  const currentExports = __prefresh_utils__.getExports(module);
  const previousHotModuleExports =
    moduleHot.data && moduleHot.data.moduleExports;

  __prefresh_utils__.registerExports(currentExports, module.id);

  if (isPrefreshComponent) {
    if (previousHotModuleExports) {
      try {
        __prefresh_utils__.flush();
        if (
          typeof __prefresh_errors__ !== 'undefined' &&
          __prefresh_errors__ &&
          __prefresh_errors__.clearRuntimeErrors
        ) {
          __prefresh_errors__.clearRuntimeErrors();
        }
      } catch (e) {
        // Only available in newer webpack versions.
        if (moduleHot.invalidate) {
          moduleHot.invalidate();
        } else {
          self.location.reload();
        }
      }
    }

    moduleHot.dispose(data => {
      data.moduleExports = __prefresh_utils__.getExports(module);
    });

    moduleHot.accept(function errorRecovery() {
      if (
        typeof __prefresh_errors__ !== 'undefined' &&
        __prefresh_errors__ &&
        __prefresh_errors__.handleRuntimeError
      ) {
        __prefresh_errors__.handleRuntimeError(error);
      }

      __webpack_require__.c[module.id].hot.accept(errorRecovery);
    });
  }
}

}),
"./index.jsx": (function (module, __webpack_exports__, __webpack_require__) {
"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony import */var react_jsx_dev_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-dev-runtime */ "../../../../../../node_modules/preact/compat/jsx-dev-runtime.js");
/* harmony import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ "../../../../../../node_modules/preact/compat/dist/compat.js");
/* harmony import */var react_dom_client__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! react-dom/client */ "../../../../../../node_modules/preact/compat/client.js");
/* harmony import */var _app__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./app */ "./app.jsx");
/* harmony import */var _update_js__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ../../update.js */ "../../update.js");
/* harmony import */var _update_js__WEBPACK_IMPORTED_MODULE_4___default = /*#__PURE__*/__webpack_require__.n(_update_js__WEBPACK_IMPORTED_MODULE_4__);
/* module decorator */ module = __webpack_require__.hmd(module);
/* provided dependency */ var __prefresh_utils__ = __webpack_require__(/*! ../../../../client/prefresh.js */ "../../../../client/prefresh.js");





var container = document.createElement('div');
container.id = "root";
document.body.appendChild(container);
var root = react_dom_client__WEBPACK_IMPORTED_MODULE_2__.createRoot(container);
root.render(/*#__PURE__*/ (0,react_jsx_dev_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxDEV)("div", {
    children: /*#__PURE__*/ (0,react_jsx_dev_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxDEV)(_app__WEBPACK_IMPORTED_MODULE_3__.App, {}, void 0, false, {
        fileName: "/Users/bytedance/rspack-dev/rspack/packages/rspack-plugin-preact-refresh/tests/hotCases/hook/useContext#provide/index.jsx",
        lineNumber: 12,
        columnNumber: 5
    }, undefined)
}, void 0, false, {
    fileName: "/Users/bytedance/rspack-dev/rspack/packages/rspack-plugin-preact-refresh/tests/hotCases/hook/useContext#provide/index.jsx",
    lineNumber: 11,
    columnNumber: 3
}, undefined));
it("should keep state", function(done) {
    expect(container.querySelector('span').textContent).toBe("before: dark");
    NEXT(_update_js__WEBPACK_IMPORTED_MODULE_4___default()(done, true, function() {
        expect(container.querySelector('span').textContent).toBe("after: light");
        done();
    }));
});
if (true) {
    module.hot.accept(/*! ./app */ "./app.jsx", function(){
/* harmony import */_app__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./app */ "./app.jsx");

});
}

/**
 * The following code is modified based on
 * //https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/packages/webpack/src/loader/runtime.js
 *
 * MIT Licensed
 * Author JoviDeCroock
 * Copyright (c) 2021-Present Preact Team
 * https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/LICENSE
 */

const isPrefreshComponent = __prefresh_utils__.shouldBind(module);

// `@vanilla-extract/webpack` does some custom preprocessing where
// `module.hot` is partially replaced. This leads to our injected
// code being executed although it shouldn't be:
//
// Intermediate result:
//
//   if (true) { // <- inlined by intermediate compile step
//     const previousHotModuleExports = module.hot.data && ...
//                    // Crash happens here ---^
//
// It crashes at that line because some intermediate compiler isn't
// running in hot mode, but the overall guard condition was compiled
// down to being truthy. By moving `module.hot` outside of the
// condition of the if-statement, it will be left as is.
const moduleHot = module.hot;

if (moduleHot) {
  const currentExports = __prefresh_utils__.getExports(module);
  const previousHotModuleExports =
    moduleHot.data && moduleHot.data.moduleExports;

  __prefresh_utils__.registerExports(currentExports, module.id);

  if (isPrefreshComponent) {
    if (previousHotModuleExports) {
      try {
        __prefresh_utils__.flush();
        if (
          typeof __prefresh_errors__ !== 'undefined' &&
          __prefresh_errors__ &&
          __prefresh_errors__.clearRuntimeErrors
        ) {
          __prefresh_errors__.clearRuntimeErrors();
        }
      } catch (e) {
        // Only available in newer webpack versions.
        if (moduleHot.invalidate) {
          moduleHot.invalidate();
        } else {
          self.location.reload();
        }
      }
    }

    moduleHot.dispose(data => {
      data.moduleExports = __prefresh_utils__.getExports(module);
    });

    moduleHot.accept(function errorRecovery() {
      if (
        typeof __prefresh_errors__ !== 'undefined' &&
        __prefresh_errors__ &&
        __prefresh_errors__.handleRuntimeError
      ) {
        __prefresh_errors__.handleRuntimeError(error);
      }

      __webpack_require__.c[module.id].hot.accept(errorRecovery);
    });
  }
}

}),
"../../update.js": (function (module, __unused_webpack_exports, __webpack_require__) {
/* module decorator */ module = __webpack_require__.nmd(module);
/* provided dependency */ var __prefresh_utils__ = __webpack_require__(/*! ../../../../client/prefresh.js */ "../../../../client/prefresh.js");
module.exports = function(done, options, callback) {
    return function(err, stats) {
        if (err) return done(err);
        module.hot.check(options || true).then(function(updatedModules) {
            if (!updatedModules) return done(new Error("No update available"));
            if (callback) callback(stats);
        }).catch(function(err) {
            done(err);
        });
    };
};

/**
 * The following code is modified based on
 * //https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/packages/webpack/src/loader/runtime.js
 *
 * MIT Licensed
 * Author JoviDeCroock
 * Copyright (c) 2021-Present Preact Team
 * https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/LICENSE
 */

const isPrefreshComponent = __prefresh_utils__.shouldBind(module);

// `@vanilla-extract/webpack` does some custom preprocessing where
// `module.hot` is partially replaced. This leads to our injected
// code being executed although it shouldn't be:
//
// Intermediate result:
//
//   if (true) { // <- inlined by intermediate compile step
//     const previousHotModuleExports = module.hot.data && ...
//                    // Crash happens here ---^
//
// It crashes at that line because some intermediate compiler isn't
// running in hot mode, but the overall guard condition was compiled
// down to being truthy. By moving `module.hot` outside of the
// condition of the if-statement, it will be left as is.
const moduleHot = module.hot;

if (moduleHot) {
  const currentExports = __prefresh_utils__.getExports(module);
  const previousHotModuleExports =
    moduleHot.data && moduleHot.data.moduleExports;

  __prefresh_utils__.registerExports(currentExports, module.id);

  if (isPrefreshComponent) {
    if (previousHotModuleExports) {
      try {
        __prefresh_utils__.flush();
        if (
          typeof __prefresh_errors__ !== 'undefined' &&
          __prefresh_errors__ &&
          __prefresh_errors__.clearRuntimeErrors
        ) {
          __prefresh_errors__.clearRuntimeErrors();
        }
      } catch (e) {
        // Only available in newer webpack versions.
        if (moduleHot.invalidate) {
          moduleHot.invalidate();
        } else {
          self.location.reload();
        }
      }
    }

    moduleHot.dispose(data => {
      data.moduleExports = __prefresh_utils__.getExports(module);
    });

    moduleHot.accept(function errorRecovery() {
      if (
        typeof __prefresh_errors__ !== 'undefined' &&
        __prefresh_errors__ &&
        __prefresh_errors__.handleRuntimeError
      ) {
        __prefresh_errors__.handleRuntimeError(error);
      }

      __webpack_require__.c[module.id].hot.accept(errorRecovery);
    });
  }
}

}),
"../../../../client/prefresh.js": (function (module, __unused_webpack_exports, __webpack_require__) {
/**
 * The following code is modified based on
 * https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/packages/webpack/src/utils/prefresh.js
 *
 * MIT Licensed
 * Author JoviDeCroock
 * Copyright (c) 2021-Present Preact Team
 * https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/LICENSE
 */ var _require = __webpack_require__(/*! @prefresh/utils */ "../../../../../../node_modules/@prefresh/utils/dist/src/index.js"), isComponent = _require.isComponent, flush = _require.flush;
// eslint-disable-next-line
var getExports = function(m) {
    return m.exports || m.__proto__.exports;
};
function isSafeExport(key) {
    return key === "__esModule" || key === "__N_SSG" || key === "__N_SSP" || key === "config";
}
function registerExports(moduleExports, moduleId) {
    self["__PREFRESH__"].register(moduleExports, moduleId + " %exports%");
    if (moduleExports == null || typeof moduleExports !== "object") return;
    for(var key in moduleExports){
        if (isSafeExport(key)) continue;
        var exportValue = moduleExports[key];
        var typeID = moduleId + " %exports% " + key;
        self["__PREFRESH__"].register(exportValue, typeID);
    }
}
var shouldBind = function(m) {
    var isCitizen = false;
    var moduleExports = getExports(m);
    if (isComponent(moduleExports)) {
        isCitizen = true;
    }
    if (moduleExports === undefined || moduleExports === null || typeof moduleExports !== "object") {
        isCitizen = isCitizen || false;
    } else {
        for(var key in moduleExports){
            if (key === "__esModule") continue;
            var exportValue = moduleExports[key];
            if (isComponent(exportValue)) {
                isCitizen = isCitizen || true;
            }
        }
    }
    return isCitizen;
};
module.exports = Object.freeze({
    getExports: getExports,
    shouldBind: shouldBind,
    flush: flush,
    registerExports: registerExports
});


}),

});
/************************************************************************/
// The module cache
var __webpack_module_cache__ = {};

// The require function
function __webpack_require__(moduleId) {

// Check if module is in cache
var cachedModule = __webpack_module_cache__[moduleId];
if (cachedModule !== undefined) {
if (cachedModule.error !== undefined) throw cachedModule.error;
return cachedModule.exports;
}
// Create a new module (and put it into the cache)
var module = (__webpack_module_cache__[moduleId] = {
id: moduleId,
loaded: false,
exports: {}
});
// Execute the module function
try {

var execOptions = { id: moduleId, module: module, factory: __webpack_modules__[moduleId], require: __webpack_require__ };
__webpack_require__.i.forEach(function(handler) { handler(execOptions); });
module = execOptions.module;
if (!execOptions.factory) {
  console.error("undefined factory", moduleId)
}
execOptions.factory.call(module.exports, module, module.exports, execOptions.require);

} catch (e) {
module.error = e;
throw e;
}
// Flag the module as loaded
module.loaded = true;
// Return the exports of the module
return module.exports;

}

// expose the modules object (__webpack_modules__)
__webpack_require__.m = __webpack_modules__;

// expose the module cache
__webpack_require__.c = __webpack_module_cache__;

// expose the module execution interceptor
__webpack_require__.i = [];

/************************************************************************/
// webpack/runtime/compat_get_default_export
(() => {
// getDefaultExport function for compatibility with non-harmony modules
__webpack_require__.n = function (module) {
	var getter = module && module.__esModule ?
		function () { return module['default']; } :
		function () { return module; };
	__webpack_require__.d(getter, { a: getter });
	return getter;
};




})();
// webpack/runtime/define_property_getters
(() => {
__webpack_require__.d = function(exports, definition) {
	for(var key in definition) {
        if(__webpack_require__.o(definition, key) && !__webpack_require__.o(exports, key)) {
            Object.defineProperty(exports, key, { enumerable: true, get: definition[key] });
        }
    }
};
})();
// webpack/runtime/get css chunk filename
(() => {
// This function allow to reference chunks
        __webpack_require__.k = function (chunkId) {
          // return url for filenames not based on template
          
          // return url for filenames based on template
          return "" + chunkId + ".css";
        };
      
})();
// webpack/runtime/get_chunk_update_filename
(() => {
__webpack_require__.hu = function (chunkId) {
            return '' + chunkId + '.' + __webpack_require__.h() + '.hot-update.js';
         };
        
})();
// webpack/runtime/get_full_hash
(() => {
__webpack_require__.h = function () {
	return "57105987294e2bff8e90";
};

})();
// webpack/runtime/get_main_filename/update manifest
(() => {
__webpack_require__.hmrF = function () {
            return "main." + __webpack_require__.h() + ".hot-update.json";
         };
        
})();
// webpack/runtime/harmony_module_decorator
(() => {
__webpack_require__.hmd = function (module) {
    module = Object.create(module);
    if (!module.children) module.children = [];
    Object.defineProperty(module, 'exports', {
        enumerable: true,
        set: function () {
            throw new Error('ES Modules may not assign module.exports or exports.*, Use ESM export syntax, instead: ' + module.id);
        }
    });
    return module;
};
})();
// webpack/runtime/has_own_property
(() => {
__webpack_require__.o = function (obj, prop) {
	return Object.prototype.hasOwnProperty.call(obj, prop);
};

})();
// webpack/runtime/hot_module_replacement
(() => {
var currentModuleData = {};
var installedModules = __webpack_require__.c;

// module and require creation
var currentChildModule;
var currentParents = [];

// status
var registeredStatusHandlers = [];
var currentStatus = "idle";

// while downloading
var blockingPromises = 0;
var blockingPromisesWaiting = [];

// The update info
var currentUpdateApplyHandlers;
var queuedInvalidatedModules;

__webpack_require__.hmrD = currentModuleData;
__webpack_require__.i.push(function (options) {
	var module = options.module;
	var require = createRequire(options.require, options.id);
	module.hot = createModuleHotObject(options.id, module);
	module.parents = currentParents;
	module.children = [];
	currentParents = [];
	options.require = require;
});

__webpack_require__.hmrC = {};
__webpack_require__.hmrI = {};

function createRequire(require, moduleId) {
	var me = installedModules[moduleId];
	if (!me) return require;
	var fn = function (request) {
		if (me.hot.active) {
			if (installedModules[request]) {
				var parents = installedModules[request].parents;
				if (parents.indexOf(moduleId) === -1) {
					parents.push(moduleId);
				}
			} else {
				currentParents = [moduleId];
				currentChildModule = request;
			}
			if (me.children.indexOf(request) === -1) {
				me.children.push(request);
			}
		} else {
			console.warn(
				"[HMR] unexpected require(" +
				request +
				") from disposed module " +
				moduleId
			);
			currentParents = [];
		}
		return require(request);
	};
	var createPropertyDescriptor = function (name) {
		return {
			configurable: true,
			enumerable: true,
			get: function () {
				return require[name];
			},
			set: function (value) {
				require[name] = value;
			}
		};
	};
	for (var name in require) {
		if (Object.prototype.hasOwnProperty.call(require, name) && name !== "e") {
			Object.defineProperty(fn, name, createPropertyDescriptor(name));
		}
	}

	fn.e = function (chunkId, fetchPriority) {
		return trackBlockingPromise(require.e(chunkId, fetchPriority));
	};

	return fn;
}

function createModuleHotObject(moduleId, me) {
	var _main = currentChildModule !== moduleId;
	var hot = {
		_acceptedDependencies: {},
		_acceptedErrorHandlers: {},
		_declinedDependencies: {},
		_selfAccepted: false,
		_selfDeclined: false,
		_selfInvalidated: false,
		_disposeHandlers: [],
		_main: _main,
		_requireSelf: function () {
			currentParents = me.parents.slice();
			currentChildModule = _main ? undefined : moduleId;
			__webpack_require__(moduleId);
		},
		active: true,
		accept: function (dep, callback, errorHandler) {
			if (dep === undefined) hot._selfAccepted = true;
			else if (typeof dep === "function") hot._selfAccepted = dep;
			else if (typeof dep === "object" && dep !== null) {
				for (var i = 0; i < dep.length; i++) {
					hot._acceptedDependencies[dep[i]] = callback || function () { };
					hot._acceptedErrorHandlers[dep[i]] = errorHandler;
				}
			} else {
				hot._acceptedDependencies[dep] = callback || function () { };
				hot._acceptedErrorHandlers[dep] = errorHandler;
			}
		},
		decline: function (dep) {
			if (dep === undefined) hot._selfDeclined = true;
			else if (typeof dep === "object" && dep !== null)
				for (var i = 0; i < dep.length; i++)
					hot._declinedDependencies[dep[i]] = true;
			else hot._declinedDependencies[dep] = true;
		},
		dispose: function (callback) {
			hot._disposeHandlers.push(callback);
		},
		addDisposeHandler: function (callback) {
			hot._disposeHandlers.push(callback);
		},
		removeDisposeHandler: function (callback) {
			var idx = hot._disposeHandlers.indexOf(callback);
			if (idx >= 0) hot._disposeHandlers.splice(idx, 1);
		},
		invalidate: function () {
			this._selfInvalidated = true;
			switch (currentStatus) {
				case "idle":
					currentUpdateApplyHandlers = [];
					Object.keys(__webpack_require__.hmrI).forEach(function (key) {
						__webpack_require__.hmrI[key](moduleId, currentUpdateApplyHandlers);
					});
					setStatus("ready");
					break;
				case "ready":
					Object.keys(__webpack_require__.hmrI).forEach(function (key) {
						__webpack_require__.hmrI[key](moduleId, currentUpdateApplyHandlers);
					});
					break;
				case "prepare":
				case "check":
				case "dispose":
				case "apply":
					(queuedInvalidatedModules = queuedInvalidatedModules || []).push(
						moduleId
					);
					break;
				default:
					break;
			}
		},
		check: hotCheck,
		apply: hotApply,
		status: function (l) {
			if (!l) return currentStatus;
			registeredStatusHandlers.push(l);
		},
		addStatusHandler: function (l) {
			registeredStatusHandlers.push(l);
		},
		removeStatusHandler: function (l) {
			var idx = registeredStatusHandlers.indexOf(l);
			if (idx >= 0) registeredStatusHandlers.splice(idx, 1);
		},
		data: currentModuleData[moduleId]
	};
	currentChildModule = undefined;
	return hot;
}

function setStatus(newStatus) {
	currentStatus = newStatus;
      self.__HMR_UPDATED_RUNTIME__.statusPath.push(newStatus);
      
	var results = [];
	for (var i = 0; i < registeredStatusHandlers.length; i++)
		results[i] = registeredStatusHandlers[i].call(null, newStatus);

	return Promise.all(results);
}

function unblock() {
	if (--blockingPromises === 0) {
		setStatus("ready").then(function () {
			if (blockingPromises === 0) {
				var list = blockingPromisesWaiting;
				blockingPromisesWaiting = [];
				for (var i = 0; i < list.length; i++) {
					list[i]();
				}
			}
		});
	}
}

function trackBlockingPromise(promise) {
	switch (currentStatus) {
		case "ready":
			setStatus("prepare");
		case "prepare":
			blockingPromises++;
			promise.then(unblock, unblock);
			return promise;
		default:
			return promise;
	}
}

function waitForBlockingPromises(fn) {
	if (blockingPromises === 0) return fn();
	return new Promise(function (resolve) {
		blockingPromisesWaiting.push(function () {
			resolve(fn());
		});
	});
}

function hotCheck(applyOnUpdate) {
	if (currentStatus !== "idle") {
		throw new Error("check() is only allowed in idle status");
	}
      self.__HMR_UPDATED_RUNTIME__ = {
        javascript: {
          outdatedModules: [],
          outdatedDependencies: [],

          acceptedModules: [],
          updatedModules: [],
          updatedRuntime: [],
          disposedModules: [],
        },
        statusPath: []
      };
      
	return setStatus("check")
		.then(__webpack_require__.hmrM)
		.then(function (update) {
			if (!update) {
				return setStatus(applyInvalidatedModules() ? "ready" : "idle").then(
					function () {
						return null;
					}
				);
			}

			return setStatus("prepare").then(function () {
				var updatedModules = [];
				currentUpdateApplyHandlers = [];

				return Promise.all(
					Object.keys(__webpack_require__.hmrC).reduce(function (
						promises,
						key
					) {
						__webpack_require__.hmrC[key](
							update.c,
							update.r,
							update.m,
							promises,
							currentUpdateApplyHandlers,
							updatedModules
						);
						return promises;
					},
						[])
				).then(function () {
					return waitForBlockingPromises(function () {
						if (applyOnUpdate) {
							return internalApply(applyOnUpdate);
						} else {
							return setStatus("ready").then(function () {
								return updatedModules;
							});
						}
					});
				});
			});
		});
}

function hotApply(options) {
	if (currentStatus !== "ready") {
		return Promise.resolve().then(function () {
			throw new Error(
				"apply() is only allowed in ready status (state: " + currentStatus + ")"
			);
		});
	}
	return internalApply(options);
}

function internalApply(options) {
	options = options || {};
	applyInvalidatedModules();
	var results = currentUpdateApplyHandlers.map(function (handler) {
		return handler(options);
	});
	currentUpdateApplyHandlers = undefined;
	var errors = results
		.map(function (r) {
			return r.error;
		})
		.filter(Boolean);

	if (errors.length > 0) {
		return setStatus("abort").then(function () {
			throw errors[0];
		});
	}

	var disposePromise = setStatus("dispose");

	results.forEach(function (result) {
		if (result.dispose) result.dispose();
	});

	var applyPromise = setStatus("apply");

	var error;
	var reportError = function (err) {
		if (!error) error = err;
	};

	var outdatedModules = [];
	results.forEach(function (result) {
		if (result.apply) {
			var modules = result.apply(reportError);
			if (modules) {
				for (var i = 0; i < modules.length; i++) {
					outdatedModules.push(modules[i]);
				}
			}
		}
	});

	return Promise.all([disposePromise, applyPromise]).then(function () {
		if (error) {
			return setStatus("fail").then(function () {
				throw error;
			});
		}

		if (queuedInvalidatedModules) {
			return internalApply(options).then(function (list) {
				outdatedModules.forEach(function (moduleId) {
					if (list.indexOf(moduleId) < 0) list.push(moduleId);
				});
				return list;
			});
		}

		return setStatus("idle").then(function () {
			return outdatedModules;
		});
	});
}

function applyInvalidatedModules() {
	if (queuedInvalidatedModules) {
		if (!currentUpdateApplyHandlers) currentUpdateApplyHandlers = [];
		Object.keys(__webpack_require__.hmrI).forEach(function (key) {
			queuedInvalidatedModules.forEach(function (moduleId) {
				__webpack_require__.hmrI[key](moduleId, currentUpdateApplyHandlers);
			});
		});
		queuedInvalidatedModules = undefined;
		return true;
	}
}

/**
 * The following code is modified based on
 * https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/packages/webpack/src/utils/Runtime.js
 *
 * MIT Licensed
 * Author JoviDeCroock
 * Copyright (c) 2021-Present Preact Team
 * https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/LICENSE
 */

__webpack_require__.i.push(function (options) {
	var originalFactory = options.factory;
	options.factory = function (moduleObject, moduleExports, webpackRequire) {
		var prevRefreshReg = self.$RefreshReg$;
		var prevRefreshSig = self.$RefreshSig$;
		self.$RefreshSig$ = function () {
			var status = "begin";
			var savedType;

			return function (type, key, forceReset, getCustomHooks) {
				if (!savedType) savedType = type;
				status = self.__PREFRESH__.sign(
					type || savedType,
					key,
					forceReset,
					getCustomHooks,
					status
				);
				return type;
			};
		};
		var reg = function (currentModuleId) {
			self.$RefreshReg$ = function (type, id) {
				self.__PREFRESH__.register(type, currentModuleId + " " + id);
			};
		};
		reg();
		try {
			originalFactory.call(this, moduleObject, moduleExports, webpackRequire);
		} finally {
			self.$RefreshReg$ = prevRefreshReg;
			self.$RefreshSig$ = prevRefreshSig;
		}
	};
});

})();
// webpack/runtime/load_script
(() => {
var inProgress = {};


// loadScript function to load a script via script tag
__webpack_require__.l = function (url, done, key, chunkId) {
	if (inProgress[url]) {
		inProgress[url].push(done);
		return;
	}
	var script, needAttach;
	if (key !== undefined) {
		var scripts = document.getElementsByTagName("script");
		for (var i = 0; i < scripts.length; i++) {
			var s = scripts[i];
			if (s.getAttribute("src") == url) {
				script = s;
				break;
			}
		}
	}
	if (!script) {
		needAttach = true;
		script = document.createElement('script');
		
		script.charset = 'utf-8';
		script.timeout = 120;
		if (__webpack_require__.nc) {
			script.setAttribute("nonce", __webpack_require__.nc);
		}
		
		script.src = url;

		
	}
	inProgress[url] = [done];
	var onScriptComplete = function (prev, event) {
		script.onerror = script.onload = null;
		clearTimeout(timeout);
		var doneFns = inProgress[url];
		delete inProgress[url];
		script.parentNode && script.parentNode.removeChild(script);
		doneFns &&
			doneFns.forEach(function (fn) {
				return fn(event);
			});
		if (prev) return prev(event);
	};
	var timeout = setTimeout(
		onScriptComplete.bind(null, undefined, {
			type: 'timeout',
			target: script
		}),
		120000
	);
	script.onerror = onScriptComplete.bind(null, script.onerror);
	script.onload = onScriptComplete.bind(null, script.onload);
	needAttach && document.head.appendChild(script);
};

})();
// webpack/runtime/make_namespace_object
(() => {
// define __esModule on exports
__webpack_require__.r = function(exports) {
	if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
		Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
	}
	Object.defineProperty(exports, '__esModule', { value: true });
};

})();
// webpack/runtime/node_module_decorator
(() => {
__webpack_require__.nmd = function (module) {
    module.paths = [];
    if (!module.children) module.children = [];
    return module;
};
})();
// webpack/runtime/public_path
(() => {
__webpack_require__.p = "https://test.cases/path/";

})();
// webpack/runtime/css_loading
(() => {
var installedChunks = {"main": 0,};
var uniqueName = "";
// loadCssChunkData is unnecessary
var loadingAttribute = "data-webpack-loading";
var loadStylesheet = function (chunkId, url, done, hmr) {
	var link,
		needAttach,
		key = "chunk-" + chunkId;
	if (!hmr) {
		var links = document.getElementsByTagName("link");
		for (var i = 0; i < links.length; i++) {
			var l = links[i];
			var href = l.getAttribute("href") || l.href;
			if (href && !href.startsWith(__webpack_require__.p)) {
				href =
					__webpack_require__.p + (href.startsWith("/") ? href.slice(1) : href);
			}
			if (
				l.rel == "stylesheet" &&
				((href && href.startsWith(url)) ||
					l.getAttribute("data-webpack") == uniqueName + ":" + key)
			) {
				link = l;
				break;
			}
		}
		if (!done) return link;
	}
	if (!link) {
		needAttach = true;
		link = document.createElement("link");
		link.setAttribute("data-webpack", uniqueName + ":" + key);
		link.setAttribute(loadingAttribute, 1);
		link.rel = "stylesheet";
		link.href = url;

		
	}
	var onLinkComplete = function (prev, event) {
		link.onerror = link.onload = null;
		link.removeAttribute(loadingAttribute);
		clearTimeout(timeout);
		if (event && event.type != "load") link.parentNode.removeChild(link);
		done(event);
		if (prev) return prev(event);
	};
	if (link.getAttribute(loadingAttribute)) {
		var timeout = setTimeout(
			onLinkComplete.bind(null, undefined, { type: "timeout", target: link }),
			120000
		);
		link.onerror = onLinkComplete.bind(null, link.onerror);
		link.onload = onLinkComplete.bind(null, link.onload);
	} else onLinkComplete(undefined, { type: "load", target: link });
	hmr
		? hmr.parentNode.insertBefore(link, hmr)
		: needAttach && document.head.appendChild(link);
	return link;
};
var oldTags = [];
var newTags = [];
var applyHandler = function (options) {
	return {
		dispose: function () {},
		apply: function () {
			var moduleIds = [];
			newTags.forEach(function (info) {
				info[1].sheet.disabled = false;
			});
			while (oldTags.length) {
				var oldTag = oldTags.pop();
				if (oldTag.parentNode) oldTag.parentNode.removeChild(oldTag);
			}
			while (newTags.length) {
				var info = newTags.pop();
				// var chunkModuleIds = loadCssChunkData(__webpack_require__.m, info[1], info[0]);
				// chunkModuleIds.forEach(function(id) {
				//     moduleIds.push(id)
				// });
			}
			return moduleIds;
		}
	};
};
var cssTextKey = function (link) {
	return Array.from(link.sheet.cssRules, function (r) {
		return r.cssText
	}).join();
};
__webpack_require__.hmrC.css = function (
	chunkIds,
	removedChunks,
	removedModules,
	promises,
	applyHandlers,
	updatedModulesList
) {
	applyHandlers.push(applyHandler);
	chunkIds.forEach(function (chunkId) {
		var filename = __webpack_require__.k(chunkId);
		var url = __webpack_require__.p + filename;
		var oldTag = loadStylesheet(chunkId, url);
		if (!oldTag) return;
		promises.push(
			new Promise(function (resolve, reject) {
				var link = loadStylesheet(
					chunkId,
					url + (url.indexOf("?") < 0 ? "?" : "&") + "hmr=" + Date.now(),
					function (event) {
						if (event.type !== "load") {
							var error = new Error();
							var errorType = event && event.type;
							var realSrc = event && event.target && event.target.src;
							error.message =
								"Loading css hot update chunk " +
								chunkId +
								" failed.\n(" +
								errorType +
								": " +
								realSrc +
								")";
							error.name = "ChunkLoadError";
							error.type = errorType;
							error.request = realSrc;
							reject(error);
						} else {
							try {
								if (cssTextKey(oldTag) == cssTextKey(link)) {
									if (link.parentNode) link.parentNode.removeChild(link);
									return resolve();
								}
							} catch (e) {}
							// var factories = {};
							// loadCssChunkData(factories, link, chunkId);
							// Object.keys(factories).forEach(function(id) {
							//     (updatedModulesList.push(id));
							// });
							link.sheet.disabled = true;
							oldTags.push(oldTag);
							newTags.push([chunkId, link]);
							resolve();
						}
					},
					oldTag
				);
			})
		);
	});
};

})();
// webpack/runtime/jsonp_chunk_loading
(() => {

      // object to store loaded and loading chunks
      // undefined = chunk not loaded, null = chunk preloaded/prefetched
      // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
      var installedChunks = __webpack_require__.hmrS_jsonp = __webpack_require__.hmrS_jsonp || {"main": 0,};
      var currentUpdatedModulesList;
var waitingUpdateResolves = {};
function loadUpdateChunk(chunkId, updatedModulesList) {
	currentUpdatedModulesList = updatedModulesList;
	return new Promise(function (resolve, reject) {
		waitingUpdateResolves[chunkId] = resolve;
		// start update chunk loading
		var url = __webpack_require__.p + __webpack_require__.hu(chunkId);
		// create error before stack unwound to get useful stacktrace later
		var error = new Error();
		var loadingEnded = function (event) {
			if (waitingUpdateResolves[chunkId]) {
				waitingUpdateResolves[chunkId] = undefined;
				var errorType =
					event && (event.type === 'load' ? 'missing' : event.type);
				var realSrc = event && event.target && event.target.src;
				error.message =
					'Loading hot update chunk ' +
					chunkId +
					' failed.\n(' +
					errorType +
					': ' +
					realSrc +
					')';
				error.name = 'ChunkLoadError';
				error.type = errorType;
				error.request = realSrc;
				reject(error);
			}
		};
		__webpack_require__.l(url, loadingEnded);
	});
}

self["webpackHotUpdate"] = function (chunkId, moreModules, runtime) {
	for (var moduleId in moreModules) {
		if (__webpack_require__.o(moreModules, moduleId)) {
			currentUpdate[moduleId] = moreModules[moduleId];
			if (currentUpdatedModulesList) currentUpdatedModulesList.push(moduleId);
		}
	}
	if (runtime) currentUpdateRuntime.push(runtime);
	if (waitingUpdateResolves[chunkId]) {
		waitingUpdateResolves[chunkId]();
		waitingUpdateResolves[chunkId] = undefined;
	}
};
var currentUpdateChunks;
var currentUpdate;
var currentUpdateRemovedChunks;
var currentUpdateRuntime;
function applyHandler(options) {
	if (__webpack_require__.f) delete __webpack_require__.f.jsonpHmr;
	currentUpdateChunks = undefined;
	function getAffectedModuleEffects(updateModuleId) {
		var outdatedModules = [updateModuleId];
		var outdatedDependencies = {};
		var queue = outdatedModules.map(function (id) {
			return {
				chain: [id],
				id: id
			};
		});
		while (queue.length > 0) {
			var queueItem = queue.pop();
			var moduleId = queueItem.id;
			var chain = queueItem.chain;
			var module = __webpack_require__.c[moduleId];
			if (
				!module ||
				(module.hot._selfAccepted && !module.hot._selfInvalidated)
			) {
				continue;
			}

			if (module.hot._selfDeclined) {
				return {
					type: "self-declined",
					chain: chain,
					moduleId: moduleId
				};
			}

			if (module.hot._main) {
				return {
					type: "unaccepted",
					chain: chain,
					moduleId: moduleId
				};
			}

			for (var i = 0; i < module.parents.length; i++) {
				var parentId = module.parents[i];
				var parent = __webpack_require__.c[parentId];
				if (!parent) {
					continue;
				}
				if (parent.hot._declinedDependencies[moduleId]) {
					return {
						type: "declined",
						chain: chain.concat([parentId]),
						moduleId: moduleId,
						parentId: parentId
					};
				}
				if (outdatedModules.indexOf(parentId) !== -1) {
					continue;
				}
				if (parent.hot._acceptedDependencies[moduleId]) {
					if (!outdatedDependencies[parentId]) {
						outdatedDependencies[parentId] = [];
					}
					addAllToSet(outdatedDependencies[parentId], [moduleId]);
					continue;
				}
				delete outdatedDependencies[parentId];
				outdatedModules.push(parentId);
				queue.push({
					chain: chain.concat([parentId]),
					id: parentId
				});
			}
		}

		return {
			type: "accepted",
			moduleId: updateModuleId,
			outdatedModules: outdatedModules,
			outdatedDependencies: outdatedDependencies
		};
	}

	function addAllToSet(a, b) {
		for (var i = 0; i < b.length; i++) {
			var item = b[i];
			if (a.indexOf(item) === -1) a.push(item);
		}
	}

	var outdatedDependencies = {};
	var outdatedModules = [];
	var appliedUpdate = {};

	var warnUnexpectedRequire = function warnUnexpectedRequire(module) {
		console.warn(
			"[HMR] unexpected require(" + module.id + ") to disposed module"
		);
	};

	for (var moduleId in currentUpdate) {
		if (__webpack_require__.o(currentUpdate, moduleId)) {
			var newModuleFactory = currentUpdate[moduleId];
			var result;
			if (newModuleFactory) {
				result = getAffectedModuleEffects(moduleId);
			} else {
				result = {
					type: "disposed",
					moduleId: moduleId
				};
			}
			var abortError = false;
			var doApply = false;
			var doDispose = false;
			var chainInfo = "";
			if (result.chain) {
				chainInfo = "\nUpdate propagation: " + result.chain.join(" -> ");
			}
			switch (result.type) {
				case "self-declined":
					if (options.onDeclined) options.onDeclined(result);
					if (!options.ignoreDeclined)
						abortError = new Error(
							"Aborted because of self decline: " + result.moduleId + chainInfo
						);
					break;
				case "declined":
					if (options.onDeclined) options.onDeclined(result);
					if (!options.ignoreDeclined)
						abortError = new Error(
							"Aborted because of declined dependency: " +
								result.moduleId +
								" in " +
								result.parentId +
								chainInfo
						);
					break;
				case "unaccepted":
					if (options.onUnaccepted) options.onUnaccepted(result);
					if (!options.ignoreUnaccepted)
						abortError = new Error(
							"Aborted because " + moduleId + " is not accepted" + chainInfo
						);
					break;
				case "accepted":
					if (options.onAccepted) options.onAccepted(result);
					doApply = true;
					break;
				case "disposed":
					if (options.onDisposed) options.onDisposed(result);
					doDispose = true;
					break;
				default:
					throw new Error("Unexception type " + result.type);
			}
			if (abortError) {
				return {
					error: abortError
				};
			}
			if (doApply) {
				appliedUpdate[moduleId] = newModuleFactory;
				addAllToSet(outdatedModules, result.outdatedModules);
				for (moduleId in result.outdatedDependencies) {
					if (__webpack_require__.o(result.outdatedDependencies, moduleId)) {
						if (!outdatedDependencies[moduleId])
							outdatedDependencies[moduleId] = [];
						addAllToSet(
							outdatedDependencies[moduleId],
							result.outdatedDependencies[moduleId]
						);
					}
				}
			}
			if (doDispose) {
				addAllToSet(outdatedModules, [result.moduleId]);
				appliedUpdate[moduleId] = warnUnexpectedRequire;
			}
		}
	}
	currentUpdate = undefined;

	var outdatedSelfAcceptedModules = [];
	for (var j = 0; j < outdatedModules.length; j++) {
		var outdatedModuleId = outdatedModules[j];
		var module = __webpack_require__.c[outdatedModuleId];
		if (
			module &&
			(module.hot._selfAccepted || module.hot._main) &&
			// removed self-accepted modules should not be required
			appliedUpdate[outdatedModuleId] !== warnUnexpectedRequire &&
			// when called invalidate self-accepting is not possible
			!module.hot._selfInvalidated
		) {
			outdatedSelfAcceptedModules.push({
				module: outdatedModuleId,
				require: module.hot._requireSelf,
				errorHandler: module.hot._selfAccepted
			});
		}
	}
      self.__HMR_UPDATED_RUNTIME__.javascript.outdatedModules = outdatedModules;
	    self.__HMR_UPDATED_RUNTIME__.javascript.outdatedDependencies = outdatedDependencies;
      

	var moduleOutdatedDependencies;
	return {
		dispose: function () {
			currentUpdateRemovedChunks.forEach(function (chunkId) {
				delete installedChunks[chunkId];
			});
			currentUpdateRemovedChunks = undefined;

			var idx;
			var queue = outdatedModules.slice();
			while (queue.length > 0) {
				var moduleId = queue.pop();
				var module = __webpack_require__.c[moduleId];
				if (!module) continue;

				var data = {};

				// Call dispose handlers
				var disposeHandlers = module.hot._disposeHandlers;
      if (disposeHandlers.length > 0) {
        self.__HMR_UPDATED_RUNTIME__.javascript.disposedModules.push(moduleId);
      }
      
				for (j = 0; j < disposeHandlers.length; j++) {
					disposeHandlers[j].call(null, data);
				}
				__webpack_require__.hmrD[moduleId] = data;

				module.hot.active = false;

				delete __webpack_require__.c[moduleId];

				delete outdatedDependencies[moduleId];

				for (j = 0; j < module.children.length; j++) {
					var child = __webpack_require__.c[module.children[j]];
					if (!child) continue;
					idx = child.parents.indexOf(moduleId);
					if (idx >= 0) {
						child.parents.splice(idx, 1);
					}
				}
			}

			var dependency;
			for (var outdatedModuleId in outdatedDependencies) {
				if (__webpack_require__.o(outdatedDependencies, outdatedModuleId)) {
					module = __webpack_require__.c[outdatedModuleId];
					if (module) {
						moduleOutdatedDependencies = outdatedDependencies[outdatedModuleId];
						for (j = 0; j < moduleOutdatedDependencies.length; j++) {
							dependency = moduleOutdatedDependencies[j];
							idx = module.children.indexOf(dependency);
							if (idx >= 0) module.children.splice(idx, 1);
						}
					}
				}
			}
		},
		apply: function (reportError) {
			// insert new code
			for (var updateModuleId in appliedUpdate) {
				if (__webpack_require__.o(appliedUpdate, updateModuleId)) {
					__webpack_require__.m[updateModuleId] = appliedUpdate[updateModuleId];
      self.__HMR_UPDATED_RUNTIME__.javascript.updatedModules.push(updateModuleId);
      
				}
			}

			// run new runtime modules
			for (var i = 0; i < currentUpdateRuntime.length; i++) {
				
      currentUpdateRuntime[i](new Proxy(__webpack_require__, {
        set(target, prop, value, receiver) {
          self.__HMR_UPDATED_RUNTIME__.javascript.updatedRuntime.push(`__webpack_require__.${prop}`);
          return Reflect.set(target, prop, value, receiver);
        }
      }));
      
			}

			// call accept handlers
			for (var outdatedModuleId in outdatedDependencies) {
				if (__webpack_require__.o(outdatedDependencies, outdatedModuleId)) {
					var module = __webpack_require__.c[outdatedModuleId];
					if (module) {
						moduleOutdatedDependencies = outdatedDependencies[outdatedModuleId];
						var callbacks = [];
						var errorHandlers = [];
						var dependenciesForCallbacks = [];
						for (var j = 0; j < moduleOutdatedDependencies.length; j++) {
							var dependency = moduleOutdatedDependencies[j];
							var acceptCallback = module.hot._acceptedDependencies[dependency];
							var errorHandler = module.hot._acceptedErrorHandlers[dependency];
							if (acceptCallback) {
								if (callbacks.indexOf(acceptCallback) !== -1) continue;
								callbacks.push(acceptCallback);
								errorHandlers.push(errorHandler);
      self.__HMR_UPDATED_RUNTIME__.javascript.acceptedModules.push(dependency);
      
								dependenciesForCallbacks.push(dependency);
							}
						}
						for (var k = 0; k < callbacks.length; k++) {
							try {
								callbacks[k].call(null, moduleOutdatedDependencies);
							} catch (err) {
								if (typeof errorHandlers[k] === "function") {
									try {
										errorHandlers[k](err, {
											moduleId: outdatedModuleId,
											dependencyId: dependenciesForCallbacks[k]
										});
									} catch (err2) {
										if (options.onErrored) {
											options.onErrored({
												type: "accept-error-handler-errored",
												moduleId: outdatedModuleId,
												dependencyId: dependenciesForCallbacks[k],
												error: err2,
												originalError: err
											});
										}
										if (!options.ignoreErrored) {
											reportError(err2);
											reportError(err);
										}
									}
								} else {
									if (options.onErrored) {
										options.onErrored({
											type: "accept-errored",
											moduleId: outdatedModuleId,
											dependencyId: dependenciesForCallbacks[k],
											error: err
										});
									}
									if (!options.ignoreErrored) {
										reportError(err);
									}
								}
							}
						}
					}
				}
			}

			// Load self accepted modules
			for (var o = 0; o < outdatedSelfAcceptedModules.length; o++) {
				var item = outdatedSelfAcceptedModules[o];
				var moduleId = item.module;
				try {
					item.require(moduleId);
				} catch (err) {
					if (typeof item.errorHandler === "function") {
						try {
							item.errorHandler(err, {
								moduleId: moduleId,
								module: __webpack_require__.c[moduleId]
							});
						} catch (err2) {
							if (options.onErrored) {
								options.onErrored({
									type: "self-accept-error-handler-errored",
									moduleId: moduleId,
									error: err2,
									originalError: err
								});
							}
							if (!options.ignoreErrored) {
								reportError(err2);
								reportError(err);
							}
						}
					} else {
						if (options.onErrored) {
							options.onErrored({
								type: "self-accept-errored",
								moduleId: moduleId,
								error: err
							});
						}
						if (!options.ignoreErrored) {
							reportError(err);
						}
					}
				}
			}

			return outdatedModules;
		}
	};
}

__webpack_require__.hmrI.jsonp = function (moduleId, applyHandlers) {
	if (!currentUpdate) {
		currentUpdate = {};
		currentUpdateRuntime = [];
		currentUpdateRemovedChunks = [];
		applyHandlers.push(applyHandler);
	}
	if (!__webpack_require__.o(currentUpdate, moduleId)) {
		currentUpdate[moduleId] = __webpack_require__.m[moduleId];
	}
};

__webpack_require__.hmrC.jsonp = function (
	chunkIds,
	removedChunks,
	removedModules,
	promises,
	applyHandlers,
	updatedModulesList
) {
	applyHandlers.push(applyHandler);
	currentUpdateChunks = {};
	currentUpdateRemovedChunks = removedChunks;
	currentUpdate = removedModules.reduce(function (obj, key) {
		obj[key] = false;
		return obj;
	}, {});
	currentUpdateRuntime = [];
	chunkIds.forEach(function (chunkId) {
		if (
			__webpack_require__.o(installedChunks, chunkId) &&
			installedChunks[chunkId] !== undefined
		) {
			promises.push(loadUpdateChunk(chunkId, updatedModulesList));
			currentUpdateChunks[chunkId] = true;
		} else {
			currentUpdateChunks[chunkId] = false;
		}
	});
	if (__webpack_require__.f) {
		__webpack_require__.f.jsonpHmr = function (chunkId, promises) {
			if (
				currentUpdateChunks &&
				__webpack_require__.o(currentUpdateChunks, chunkId) &&
				!currentUpdateChunks[chunkId]
			) {
				promises.push(loadUpdateChunk(chunkId));
				currentUpdateChunks[chunkId] = true;
			}
		};
	}
};
__webpack_require__.hmrM = function () {
	if (typeof fetch === "undefined")
		throw new Error("No browser support: need fetch API");
	return fetch(__webpack_require__.p + __webpack_require__.hmrF()).then(
		function (response) {
			if (response.status === 404) return; // no update available
			if (!response.ok)
				throw new Error(
					"Failed to fetch update manifest " + response.statusText
				);
			return response.json();
		}
	);
};

})();
/************************************************************************/
// module cache are used so entry inlining is disabled
// startup
// Load entry module and return exports
__webpack_require__("../../../../../../node_modules/@prefresh/core/src/index.js");
var __webpack_exports__ = __webpack_require__("./index.jsx");
module.exports = __webpack_exports__;
})()
;