(function () { 
      // runtime instance
      var runtime = new Object();
      self['__rspack_runtime__'] = runtime;
      
      // mount Modules
      (function () {
        runtime.installedModules = {};
      })();
    
      // mount Chunks
      (function () {
        runtime.installedChunks = {};
      })();
    
      // mount ModuleCache
      (function () {
        runtime.moduleCache = {};
      })();
      // mount PublicPath
      (function () {
        runtime.publicPath = '/';
      })();
      (function () {
        runtime.checkById = function (obj, prop) {
          return Object.prototype.hasOwnProperty.call(obj, prop);
        };
      })();
      // The require function
      function __rspack_require__(moduleId) {
        var cachedModule = this.moduleCache[moduleId];
        if (cachedModule !== undefined) {
          return cachedModule.exports;
        }
    
        // Create a new module (and put it into the cache)
        var module = (this.moduleCache[moduleId] = {
          // no module.id needed
          // no module.loaded needed
          exports: {},
        });
    
        this.installedModules[moduleId](
          module,
          module.exports,
          this.__rspack_require__.bind(this),
          this.__rspack_dynamic_require__.bind(this)
        );
    
        return module.exports;
      }
    
      // mount require function
      (function () {
        runtime.__rspack_require__ = __rspack_require__;
      })();
      // The register function
      function __rspack_register__(chunkIds, modules, callback) {
        if (
          chunkIds.some(
            function (id) {
              return this.installedChunks[id] !== 0;
            }.bind(this)
          )
        ) {
          for (moduleId in modules) {
            if (this.checkById(modules, moduleId)) {
              this.installedModules[moduleId] = modules[moduleId];
            }
          }
          if (callback) callback(this.__rspack_require__);
        }
        for (var i = 0; i < chunkIds.length; i++) {
          chunkId = chunkIds[i];
          if (this.checkById(this.installedChunks, chunkId) && this.installedChunks[chunkId]) {
            this.installedChunks[chunkId][0]();
          }
          this.installedChunks[chunkId] = 0;
        }
      }
    
      // mount register function
      (function () {
        runtime.__rspack_register__ = __rspack_register__;
      })();
      // mount Css Chunks
      (function () {
        runtime.installedCssChunks = {};
      })();
      
      (function () {
        runtime.chunkHashData = {
          js: {},
          css: {}
        };
      })();
      (function () {
        runtime.setChunkHashData = function (chunkId, hash, type) {
          return this.chunkHashData[type][chunkId] = hash;
        };
      })();
      function __rspack_dynamic_require__(chunkId) {
        return Promise.all(
          Object.keys(this)
            .filter(function (key) {
              return key.indexOf('rspack_load_dynamic') > 0;
            })
            .reduce(function (promises, key) {
              this[key](chunkId, promises);
              return promises;
            }.bind(this), [])
        );
      }
    
      // mount register dynamic require
      (function () {
        runtime.__rspack_dynamic_require__ = __rspack_dynamic_require__;
      })();
      (function () {
        runtime.__rspack_get_dynamic_chunk_url__ = function (chunkId, type) {
          return 'static/' + type + '/' + chunkId + '.' + this.chunkHashData[type][chunkId] + '.chunk.' + type;
        };
      })();
      
      (function () {
        runtime.__rspack_has_dynamic_chunk__ = function (chunkId, type) {
          return Boolean(this.chunkHashData && this.chunkHashData[type] && this.chunkHashData[type][chunkId]);
        };
      })();
      
      var inProgress = {};
      function load_script(url, done, key) {
        var dataWebpackPrefix = 'rspack-test:';
        if (inProgress[url]) {
          inProgress[url].push(done);
          return;
        }
        var script, needAttach;
        if (key !== undefined) {
          var scripts = document.getElementsByTagName('script');
          for (var i = 0; i < scripts.length; i++) {
            var s = scripts[i];
            if (s.getAttribute('src') == url || s.getAttribute('data-rspack') == dataWebpackPrefix + key) {
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
          script.setAttribute('data-rspack', dataWebpackPrefix + key);
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
        var timeout = setTimeout(onScriptComplete.bind(null, undefined, { type: 'timeout', target: script }), 120000);
        script.onerror = onScriptComplete.bind(null, script.onerror);
        script.onload = onScriptComplete.bind(null, script.onload);
        needAttach && document.head.appendChild(script);
      }
    
      function __rspack_load_dynamic_js__(chunkId, promises) {
        var runtime = this;
        var installedChunkData = this.checkById(this.installedChunks, chunkId) ? this.installedChunks[chunkId] : undefined;
        if (installedChunkData !== 0) {
          if (installedChunkData) {
            promises.push(installedChunkData[2]);
          } else {
            var promise = new Promise(function (resolve, reject) { installedChunkData = this.installedChunks[chunkId] = [resolve, reject]; }.bind(this));
            promises.push((installedChunkData[2] = promise));
            var url = this.publicPath + this.__rspack_get_dynamic_chunk_url__(chunkId, 'js');
            var error = new Error();
            var loadingEnded = function (event) {
              if (runtime.checkById(runtime.installedChunks, chunkId)) {
                installedChunkData = runtime.installedChunks[chunkId];
                if (installedChunkData !== 0) runtime.installedChunks[chunkId] = undefined;
                if (installedChunkData) {
                  var errorType = event && (event.type === 'load' ? 'missing' : event.type);
                  var realSrc = event && event.target && event.target.src;
                  error.message = 'Loading chunk ' + chunkId + ' failed.\n(' + errorType + ': ' + realSrc + ')';
                  error.name = 'ChunkLoadError';
                  error.type = errorType;
                  error.request = realSrc;
                  installedChunkData[1](error);
                }
              }
            };
            load_script(url, loadingEnded, 'chunk-' + chunkId);
          }
        }
      }
    
      // mount load dynamic js
      (function () {
        runtime.__rspack_load_dynamic_js__ = __rspack_load_dynamic_js__;
      })();
      function load_style(chunkId, href, fullhref, resolve, reject) {
        var existingLinkTags = document.getElementsByTagName('link');
        for (var i = 0; i < existingLinkTags.length; i++) {
          var tag = existingLinkTags[i];
          var dataHref = tag.getAttribute('data-href') || tag.getAttribute('href');
          if (tag.rel === 'stylesheet' && (dataHref === href || dataHref === fullhref)) return resolve();
        }
        var existingStyleTags = document.getElementsByTagName('style');
        for (var i = 0; i < existingStyleTags.length; i++) {
          var tag = existingStyleTags[i];
          var dataHref = tag.getAttribute('data-href');
          if (dataHref === href || dataHref === fullhref) return resolve();
        }
        var linkTag = document.createElement('link');
        linkTag.rel = 'stylesheet';
        linkTag.type = 'text/css';
        var onLinkComplete = function (event) {
          linkTag.onerror = linkTag.onload = null;
          if (event.type === 'load') {
            resolve();
          } else {
            var errorType = event && (event.type === 'load' ? 'missing' : event.type);
            var realHref = (event && event.target && event.target.href) || fullhref;
            var err = new Error('Loading CSS chunk ' + chunkId + ' failed.\n(' + realHref + ')');
            err.code = 'CSS_CHUNK_LOAD_FAILED';
            err.type = errorType;
            err.request = realHref;
            linkTag.parentNode.removeChild(linkTag);
            reject(err);
          }
        };
        linkTag.onerror = linkTag.onload = onLinkComplete;
        linkTag.href = fullhref;
        document.head.appendChild(linkTag);
        return linkTag;
      }
    
      function __rspack_load_dynamic_css__(chunkId, promises) {
        var installedChunkData = this.installedCssChunks[chunkId];
        if (installedChunkData) {
          promises.push(installedChunkData);
        } else if (installedChunkData !== 0 && this.__rspack_has_dynamic_chunk__(chunkId, 'css')) {
          var href = this.__rspack_get_dynamic_chunk_url__(chunkId, 'css');
          var fullhref = this.publicPath + href;
          promises.push(
            (installedChunkData = new Promise(function (resolve, reject) {
              load_style(chunkId, href, fullhref, resolve, reject);
            }).then(
              function () {
                installedChunkData = 0;
              },
              function (e) {
                delete installedChunkData;
                throw e;
              }
            ))
          );
        }
      }
    
      // mount load dynamic css
      (function () {
        runtime.__rspack_load_dynamic_css__ = __rspack_load_dynamic_css__;
      })(); })();