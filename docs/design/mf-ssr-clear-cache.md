# MF SSR ClearCache 设计方案

## 背景问题

MF 在 SSR 场景中运行在 Node 进程内。remote 第一次加载后，旧版本会同时留在多层缓存里：

- MF runtime 的 remote 实例缓存、remote entry exports、manifest 和 snapshot cache。
- bundler runtime 写回的 remote module factory、remote module exports 和 remote loading 标记。
- 消费者模块已经执行后的 `__webpack_require__.c` 缓存。
- MF Node runtime plugin 注入的 chunk 加载缓存。
- SDK Node loader 在 `loadScriptNode` 中维护的 remote entry 全局导出和 ESM module cache。
- Node 自身的 CommonJS `require.cache` 或 ESM 模块缓存。
- 浏览器侧的全局 remote entry、加载中记录和 HTTP 相关缓存。

当业务已经收到 remote 更新信号时，只删除生产者 remote 的缓存不够。比如 `pageA` 静态消费了 `remoteA`，`pageA` 第一次 SSR 时已经执行并进入消费者缓存；下一次渲染如果直接复用 `pageA` 的 exports，通常不会重新执行 `pageA`，也就不会重新触发 `loadRemoteA`。因此 `clearCache` 必须同时处理 remote 缓存和受影响消费者缓存。

目标是提供一个 remote 粒度的 `clearCache`，让业务在收到更新信号后只清理缓存、不重启服务，就能让后续请求重新加载当前注册指向的 remote，同时避免因为粗暴全局清理带来明显内存增长、运行态错乱或 shared 单例问题。

如果 remote 更新意味着 entry 地址也要替换，应先通过 `registerRemotes(..., { force: true })` 替换注册，再复用 `clearCache` 的旧缓存失效能力。第一版不保证新 remote 已经加载成功。

## 目标

第一版只做安全最小能力：

- `clearCache` 由 MF runtime 提供公共入口。
- 业务只面向 MF runtime，不直接操作 bundler runtime、SDK Node loader 或 Node runtime plugin。
- 默认按 remote 名字清理，不做全局重置。
- `clearCache({ name })` 保留 `host.options.remotes` 中的 remote 注册。
- remote 地址替换由 `registerRemotes(..., { force: true })` 更新注册，并复用旧缓存失效能力。
- SSR 默认追溯并清理受影响的消费者模块缓存。
- 浏览器默认更保守，只清 remote 相关可控缓存，不做广泛消费者祖先清理。
- 默认保留已加载 shared，只清未加载、失败或仅注册的 shared。
- `clearCache` 是异步流程，必须等待目标 remote 当前正在进行的加载完成或结束后再清理。
- `clearCache` 只负责失效旧缓存，不提前加载、验证或切换到新 remote；下一次请求按正常路径重新加载当前注册的 remote。

第一版不做：

- 不取消已经在执行中的 SSR 请求。
- 不强制回收业务代码已经持有的旧 remote 或旧消费者引用。
- 不保证清除 Node 原生 ESM 同 URL 模块缓存。
- 不加入强制删除 loaded shared 的公开选项。
- 不提前请求 remote entry，不做 remote 健康检查，不准备新的 remote factory。
- 不复用 HMR 的 accept、decline、dispose callback 语义。
- 不把“移除 remote 注册”作为 `clearCache` 的一部分。

## 缓存分层

### MF runtime

MF runtime 是 `clearCache` 的唯一公共入口，负责定义语义、串联各层清理 adapter，并提供最终一致性保护。

它拥有或能定位这些状态：

- `host.options.remotes` 中注册的 remote。
- `host.moduleCache` 中 remote 对应的 `Module`。
- remote entry exports。
- remote entry 加载中的 `globalLoading` 记录。
- snapshot 和 manifest loading 记录。
- remote 自身作为 MF instance 留在全局 `__FEDERATION__.__INSTANCES__` 中的记录。
- shared scope map 中由该 remote 贡献的 shared。

MF runtime 应该抽取现有私有清理逻辑中可复用的缓存清理步骤，但不能直接把 `removeRemote` 升级成 `clearCache`。现有 `removeRemote` 同时承担移除注册、清缓存和清 shared 的职责，其中 shared 清理行为可能删除已加载 shared，和本方案的 safe shared 策略冲突。

第一版需要拆成两种语义：

- `clearCache({ name })`：只清缓存，保留 remote 注册。
- `registerRemotes(..., { force: true })`：替换 remote 注册，并复用同一套旧缓存清理能力；第一版不提前加载新 remote。

### bundler runtime

bundler runtime 不直接暴露给业务，只作为 MF runtime 的内部清理参与者。

它需要清理两类状态：

- remote module 状态：remote module factory、remote module exports、remote loading promise 或成功失败标记。
- 消费者 module 状态：已经执行过的消费者 module exports。

两类状态不能混在一起处理：

- remote module 的 factory 是运行时写入的，更新 remote 时不能只删除 factory，而是要恢复成可重新加载状态。
- 消费者 module 的 factory 是本地 bundle 产物，不能删除；只删除执行结果缓存，让下一次请求重新执行本地消费者代码。

这样下一次加载同一个消费者时，会重新执行消费者 factory，并重新触发 remote module 的加载流程，而不是直接返回旧 exports。

remote module 的可重新加载状态需要满足：

- remote loading 标记回到未加载状态。
- remote module 的旧 exports 被删除。
- 不留下会被消费者同步 `require` 到的缺失 factory 状态。
- 旧 factory 写回状态被失效，但下一次正常加载仍能重新进入 remote loading 流程。
- SSR 集成层不需要调用新的 remote reload API，只需要按原方式重新进入 SSR 入口。

这里的关键约束是：清理旧 factory 写回状态时，不能让 `webpackRequire.m[remoteModuleId]` 变成会被消费者同步 `require` 到的缺失状态。实现可以保留或恢复一个能触发 remote loading 的 remote module factory，也可以重置相关 chunk / remote ensure 状态，确保消费者入口重新执行时先走 ensure，再同步消费 remote module。禁止出现“消费者重新执行时同步 require remote module，但 factory 已被删除且加载流程没有重新触发”的状态。

SSR 集成层仍然按原方式重新进入 SSR 入口。如果服务层绕过入口，直接复用已经缓存的 render handler、路由 handler 或消费者引用，runtime 无法替它重新触发 remote loading，集成层需要自己清这些外部缓存。

`clearCache` 不要求 bundler runtime 在清理时重新准备 remote factory。新 remote 是否可用，由下一次正常 remote loading 决定。

### SDK node loader

MF 在 Node 侧加载生产者入口时会走 `loadScriptNode`。

这层也需要成为 clear 流程的一部分：

- 普通 script remoteEntry 不会进入 Node `require.cache`，但执行结果会写到 `globalThis[remoteEntryKey]`，需要按 remote 的 `globalName` 或 `__FEDERATION_${name}:custom__` 删除。
- ESM remoteEntry 会进入 SDK 内部 `esmModuleCache`，需要 SDK 提供按 root URL 清理的内部方法。
- 如果要清理 root module 递归 import 过的子 URL，SDK Node loader 必须在 `loadModule` 时记录 root URL 到 child URL 的依赖关系；没有依赖记录时，第一版只承诺清理已知 root URL。
- `sdkImportCache` 主要缓存 `path`、`vm`、`node-fetch`、`node:module` 这类 Node 依赖，不能全量清理；只有 URL-like 且能关联到目标 remote 的 key 才允许删除。

这部分不应该由 MF runtime 直接操作私有 Map，而应由 SDK Node loader 提供内部 clear adapter。

### MF node runtime plugin

MF Node runtime plugin 负责 SSR Node 侧的 chunk 加载缓存。

它需要清理或串联：

- Rspack 生成的 Node chunk loading runtime 暴露出来的目标 chunk 加载状态。
- MF Node runtime plugin 中与目标 remote 或 remote chunk 相关的动态 import 缓存记录。
- 注入 `.f.readFileVm` 和 `.f.require` 时维护的 `installedChunks`。
- 通过文件系统加载过的 remote chunk 对应的 CommonJS `require.cache`。
- 通过远程 URL 拉取并执行过的 chunk 加载状态。

这些状态不全在 MF Node runtime plugin 内部。Rspack 生成的 Node chunk loading runtime 也需要提供受控清理入口，暴露目标 chunk 的加载状态、chunk 到 remote 的关系和 generation 检查点。MF Node runtime plugin 负责串联这些能力，而不是猜闭包里的 `installedChunks`。

`nodeRuntimeImportCache` 是 MF Node runtime plugin 里的动态 import 缓存记录，在 `module-federation/core` 的 `packages/node/src/runtimePlugin.ts` 中维护，不是 Rspack runtime 自带缓存。当前 Rspack 仓库没有这个缓存表或 remote URL chunk 状态的直接接入口，所以第一版只声明清理由 Rspack 生成的 Node chunk runtime 和 CommonJS `require.cache` 可控状态。remote URL chunk 和等价动态 import 缓存需要等 MF Node runtime plugin 暴露明确 adapter 后再接入。

### Node 原生缓存

CommonJS 可以按文件路径清理 `require.cache`。

原生 ESM 没有可靠的同 URL 删除缓存能力。对 ESM remote，必须使用版本化 URL，例如带 hash、版本号或 query。`clearCache` 可以清掉 MF 层、bundler 层和 SDK 可控状态，但同一个原生 ESM URL 是否重新执行不应该作为承诺。

### 浏览器缓存

浏览器支持 `clearCache`，但范围限定为运行时可控状态：

- remote entry 全局变量。
- remote entry 加载中的记录。
- MF runtime remote module cache。
- bundler runtime 的 remote module cache。

浏览器 HTTP 缓存、Service Worker 缓存、原生 ESM 同 URL模块缓存不由 `clearCache` 保证清除。remote 更新应使用版本化 URL。

## Public API

第一版公共 API 保持最小：

```ts
await clearCache({ name: 'remoteA' });
```

建议类型：

```ts
type ClearCacheOptions = {
  name: string;
};

type ClearCacheResult = {
  name: string;
  cleared: true;
};
```

`name` 使用 remote 的注册名。MF runtime 内部可以兼容 alias、remote key、global name，但公共文档里先要求传注册名，避免同一个 remote 有多个名字时出现歧义。

`clearCache` 成功时 resolve `ClearCacheResult`，只表示旧缓存已被失效，后续加载会重新走当前注册的 remote。它不表示新版 remote 已经加载、验证或可用。下一次 remote 加载失败，应归类为正常 remote loading 阶段的错误，不是 `clearCache` 失败。

只有 clear 过程自身无法完成时才 Promise reject，例如内部索引缺失、adapter 清理失败或 barrier 状态无法恢复。等待旧加载超时不应导致 `clearCache` 失败，而应把旧加载标记为 stale 并继续清理。

如果 clear 过程自身失败，`clearCache` 必须停止后续清理并抛出错误。实现需要在清理前保存旧目标和旧状态快照，确保失败时仍有足够信息恢复到可继续服务的旧状态。

第一版不暴露 `force`、`dryRun`、`includeShared` 这类高级选项。后续如果确实需要，可以在不破坏默认安全行为的前提下扩展。

## bundler runtime 与 remote 的建连

clearCache 需要明确知道“哪个 remote 对应哪些本地 module id”。这部分应该由 bundler 编译期和运行时共同建立。

Rspack 当前已经有 remote loading metadata：

- `chunkMapping`：chunk id 到 remote module id 的关系。
- `moduleIdToRemoteDataMapping`：remote module id 到 remote data 的关系。
- `remoteData.remoteName`：当前写入的是 remote key。
- `remoteData.externalModuleId`：加载 remote entry/container 的 external module id。

在此基础上需要补充反向索引：

- `remoteKeyToRemoteModuleIds`：remote key 到 remote module id。
- `remoteKeyToExternalModuleIds`：remote key 到 external module id。
- `remoteModuleIdToConsumerModuleIds`：remote module id 到直接消费者 module id。
- `consumerModuleIdToParentModuleIds`：消费者 module 到上层同步消费者 module。
- `remoteKeyToChunkIds`：remote key 到相关 chunk id。

这些索引可以在 `RemoteRuntimeModule` 生成 runtime metadata 时从 module graph 和 chunk graph 里产出。运行时不应该依赖 HMR runtime 的 `module.parents`，因为生产环境可能没有启用 HMR。

MF runtime 清理时先把 `name` 规范化成内部目标：

```ts
type ClearCacheTarget = {
  name: string;
  remoteKey: string;
  entry: string;
  globalName?: string;
};
```

MF runtime 用 `name` 清自己的 remote 实例和 module cache；bundler runtime 用 `remoteKey` 清编译产物里的 remote module 和消费者 module；SDK Node loader 用 `entry` 和 `globalName` 清 remoteEntry 相关缓存。

## 消费者失效策略

这里的思路和 HMR 类似：从发生变化的 remote module 出发，沿反向依赖关系找到需要失效的最小上层消费者集合。

但 clearCache 不复用 HMR 的 accept/decline 语义。它不是要在当前页面或当前请求里热替换模块，而是让后续请求重新执行必要的消费者链路。

### 为什么必须清消费者

对于静态消费：

```ts
import RemoteButton from 'remoteA/Button';

export function pageA() {
  return render(RemoteButton);
}
```

如果 `pageA` 已经执行过，它的 exports 会留在消费者缓存中。后续 SSR 再次渲染时，如果直接复用 `pageA` 的 exports，就不会重新执行上面的 import，也不会重新触发 remote loading。

因此只清 remote entry、remote module factory 或 remote exports，仍可能返回旧消费者闭包里的旧 remote 引用。

### SSR 默认策略

SSR 默认启用消费者追溯：

1. 以目标 remote 的 remote module id 作为 seed。
2. 将 remote module 恢复成可重新加载状态，清理旧 exports 和 loading 标记。
3. 删除 external module 的 exports，让 remote entry/container 下次重新加载。
4. 找到直接消费者 module id，删除这些消费者的 exports。
5. 对 active 且同步的 incoming dependency 继续向上追溯，直到遇到入口、异步边界或未执行的 module。
6. 删除追溯到的已执行消费者 exports。

静态父依赖需要继续追溯，是因为父模块也可能在执行时捕获了子模块 exports。异步边界默认不继续穿透，因为下一次动态 import 会重新进入已被清理的子模块。

这套策略追求的是“最小正确失效集”：不清全局 module cache，但要清到足以让下一次 SSR 请求重新执行 remote 消费链路。

SSR 集成层还必须处理自己持有的入口引用。如果服务层缓存了入口导出的 render handler、路由 handler 或 page module 引用，runtime 只能清内部 module cache，不能替换这些外部变量。集成层需要在 `clearCache` 后清掉自己的 handler 缓存，或每次请求从 runtime 重新获取入口。

### 追溯边界

消费者追溯规则必须由编译期索引表达清楚，不能留给运行时猜测。

继续向上追溯的边：

- dependency 在当前 runtime 条件下是 active。
- dependency 属于同步模块依赖。
- parent module 已经执行并存在于 module cache 中。

停止追溯的边界：

- dynamic import 或其它 async block。
- 未执行的 module。
- external module。
- runtime module。
- 已经访问过的 module。

如果追溯到已执行入口模块，也清入口模块的 module cache。循环依赖通过 visited set 防止重复访问。

### 浏览器默认策略

浏览器默认不做广泛消费者祖先清理。

原因是浏览器页面有长生命周期状态，广泛删除消费者 exports 可能让事件处理、组件状态、客户端路由和单例对象处于不一致状态。

浏览器第一版只保证：

- 让 remote module 回到可重新加载状态，并清旧 exports 和 loading 标记。
- 清 external module exports。
- 清 MF runtime 可控的 remote cache。
- 清 script/global remote entry。

如果业务希望浏览器也追溯消费者，应作为后续显式能力设计，而不是第一版默认行为。

## installedChunks 安全边界

`installedChunks` 不能全量清理，也不能在 chunk 正在加载时直接删除。

危险场景：

1. 旧 remote chunk 正在加载。
2. `clearCache` 删除了 `installedChunks[chunkId]`。
3. 新请求又触发同一个 chunk 加载。
4. 旧加载稍后完成，把旧 modules 写回 `__webpack_require__.m`，并把 `installedChunks[chunkId]` 标记为 loaded。

因此 Node runtime plugin 的 adapter 必须满足：

- Node runtime plugin 和 Rspack Node chunk loading runtime 的 remote 相关状态都必须记录 remote generation。
- `clearCache` 推进 generation 后，旧 chunk、动态 import 和 cache 清理只能命中旧 generation。
- 不允许只按 remote name 或 chunk id 粗暴清理，避免影响非目标业务 chunk 或下一次正常加载产生的新状态。
- chunk install 前检查 remote generation，过期加载结果不能写回 module factory 或 `installedChunks`。
- 清理 loaded chunk 时只删除目标 remote 的旧 generation chunk，不影响后续正常加载的新状态或本地业务 chunk。

bundler runtime 不直接操作 Node plugin 闭包里的 `installedChunks`。它只清自己持有的 remote module、external module 和消费者 module 缓存；`installedChunks` 由 Node runtime plugin 自己清。

## 清理流程

`clearCache({ name })` 的推荐顺序：

1. MF runtime 在 `try/finally` 中为目标 remote 设置 clear barrier，让新的同 remote 加载等待，防止它插入旧状态清理过程；`finally` 必须释放 barrier。
2. 根据 `name` 找到当前 remote，规范化出 `name`、`remoteKey`、`entry` 和 `globalName`。
3. 保存旧状态快照，包括 MF runtime cache、SDK global entry、bundler remote factory 写回状态、loading 标记、当前 generation、remote module、external module、消费者 module 和 chunk 索引。
4. 等待目标 remote 当前正在进行的 entry、module、chunk 加载 settle，但等待必须有内部上限；超时后把这次旧加载标记为 stale。
5. 推进目标 remote generation，防止旧异步结果后续写回。
6. 清理旧 remote 缓存，包括 MF runtime `moduleCache`、remote entry exports、global loading、snapshot、manifest loading、remote 全局变量和旧 MF instance。
7. 通知 bundler runtime 失效旧 remote module 执行结果、旧 loading 状态、旧 factory 写回状态、external module exports 和受影响 SSR 消费者 exports。
8. 通知 SDK Node loader 清理 `loadScriptNode` 相关的旧 global entry 和 ESM module cache。
9. 如果当前环境安装了 MF Node runtime plugin，通知它清理目标 remote 的旧 generation chunk、动态 import 和 CommonJS cache。
10. 按 safe shared 策略清理该 remote 贡献的 shared。
11. 保留 `host.options.remotes` 中的 remote 注册。
12. 无论成功、失败或超时，都在 `finally` 中释放 clear barrier，不能永久阻塞同名 remote。

`clearCache` 不提前加载 remote entry，不执行 remote `get`，不验证新 remote 是否可用。barrier 释放后，后续请求按正常路径重新加载当前注册的 remote。

如果业务只是更新同一个 entry URL 背后的内容，`clearCache({ name })` 只在下一次请求确实能拿到新内容的前提下足够。浏览器缓存、Service Worker、HTTP 代理缓存、Node fetch cache 和 ESM 同 URL 模块缓存都可能让同 URL 继续返回旧内容。生产更新仍推荐版本化 URL。  
如果业务需要替换 entry 地址，必须使用 `registerRemotes([{ name, entry: newEntry }], { force: true })`，由 register 流程替换注册并清理旧缓存。

对于正在执行中的 SSR 请求，`clearCache` 不强行中断。旧请求可以继续使用已经拿到的引用，新请求重新进入加载流程。

## stale load 保护

`clearCache` 必须处理异步竞争：

- 旧 remote 正在加载时，业务调用了 `clearCache`。
- 清理完成后，旧 remote 的加载 promise 才 resolve。
- 如果没有保护，旧 promise 可能把旧 factory、旧 remote entry、旧 consumer exports 或旧 shared 再写回缓存。

推荐做法是给每个 remote 维护一个内部 generation。

- 开始加载 remote entry、remote module、remote chunk 或 shared 时读取当前 generation。
- `clearCache` 开始清理旧状态时推进 generation。
- 异步加载完成后，写缓存前检查 generation 是否仍一致。
- 如果不一致，丢弃这次旧结果，不再写回 runtime cache。

这个保护应该覆盖 MF runtime、bundler runtime、SDK Node loader 和 Node runtime plugin 的写缓存位置。

等待当前加载不能无限阻塞。实现上需要一个内部超时上限；超时后旧加载被标记为 stale，后续即使成功返回，也不能写回任何 runtime cache。第一版不把 timeout 暴露成公共 API。

stale 只表示旧加载结果不能再写回 runtime cache，不代表取消已经开始的旧请求。已经持有并等待这个 promise 的旧 SSR 请求仍然可以拿到原本的成功结果或错误；`clearCache` 只阻止它污染后续请求使用的缓存状态。

## shared 策略

第一版采用 `safe` 策略，并作为默认且唯一公开行为。

### 可以清理

满足以下条件的 shared 可以从 share scope 中删除：

- `from` 指向被清理的 remote。
- 没有 `lib`。
- 没有 `loaded`。

这类 shared 没有完成加载，还没有把可复用实例交给 host 或其它 remote，清理风险低。

### 正在加载

如果 shared 正在加载：

- 不强行取消。
- 等待当前加载 settle，或给它打上与 remote generation 相关的失效标记。
- 如果旧 loading 后续成功返回，但 generation 已失效，不再把结果登记为可复用 shared。

这样可以避免旧 shared 在清理后重新污染缓存。

### 已加载

已加载 shared 默认保留。

原因是 `lib` 一旦存在，可能已经被 host、其它 remote、当前 SSR 请求或业务全局对象持有。强行删除注册表不能回收这些引用，反而可能让后续请求加载另一份实例，造成同一进程内多份 React、状态库或其它单例并存。

即使 `useIn` 暂时为空，第一版也不删除 loaded shared。`useIn` 可以作为未来更精细 refcount 策略的参考，但不作为第一版删除依据。

## 浏览器行为

浏览器中 `clearCache({ name })` 应支持 script/global remote，但只清 runtime 可控的旧状态：

- 清理旧 remote entry 全局变量。
- 清理旧 loading promise。
- 清理旧 MF runtime `moduleCache`。
- 让 bundler runtime 中的 remote module 回到可重新加载状态，并清理旧 exports 和 loading 标记。
- 清理旧 external module exports。

`clearCache` 不准备新的 script/global remote entry。下一次请求再按当前注册插入或加载 remote entry。浏览器第一版仍不做广泛消费者追溯，避免破坏长生命周期页面状态。

对于 ESM remote：

- `clearCache` 只清 runtime 可控状态。
- 同一个 ESM URL 是否重新执行不做保证。
- 推荐 remote entry 必须带版本化 URL。

对于 HTTP 缓存和 Service Worker 缓存：

- `clearCache` 不直接操作。
- 如果业务使用 Service Worker，应由业务自己的更新策略负责失效。

## Node SSR 行为

Node SSR 是这次设计的重点。

MF Node runtime plugin 应提供内部清理 adapter，让 MF runtime 可以通知它清理目标 remote。

当前 Rspack 实现需要覆盖：

- Rspack Node chunk loading runtime 暴露出来的目标 chunk loading 状态。
- `.f.readFileVm` 维护的 `installedChunks`。
- `.f.require` 维护的 `installedChunks`。
- filesystem chunk 的 CommonJS `require.cache`。

后续等 MF Node runtime plugin 暴露明确 adapter 后，再覆盖：

- `nodeRuntimeImportCache` 或等价动态 import 缓存记录。
- remote URL chunk 的加载状态。

CommonJS remote entry 或 filesystem chunk 可以按路径清理 `require.cache`。

原生 ESM remote entry 不保证同 URL 清理。文档和测试都应明确：如果 remote 使用 ESM，在更新时必须换 URL。

## 与 registerRemotes 的关系

`registerRemotes(remotes, { force: true })` 可以继续支持覆盖同名 remote，但第一版不要求它先加载或验证新 remote。

推荐流程：

1. 在 `try/finally` 中为目标 remote 设置 clear barrier，让新的同名 remote 加载等待；`finally` 必须释放 barrier。
2. 保存旧 remote 注册和旧 runtime 快照，生成 `oldTarget`。
3. 用新的 remote 配置替换 `host.options.remotes` 中的注册。
4. 把 `oldTarget` 传给内部 clear adapter，清理旧 remote 相关可控缓存和受影响消费者缓存。
5. 如果旧缓存清理失败，停止后续流程，恢复旧 remote 注册和可恢复的旧 runtime 状态，并让 `registerRemotes(..., { force: true })` Promise reject。
6. 新 remote 是否可用由后续正常 remote loading 决定。

`oldTarget` 至少需要包含旧 `name`、旧 `remoteKey`、旧 `entry`、旧 `globalName`、旧 generation、SDK loader key，以及旧 remote module、external module、consumer module、chunk 索引。替换注册后不能再按 `name` 调公共 `clearCache({ name })` 来清旧缓存，因为公共入口会按当前注册解析目标，此时看到的是新 remote，可能漏清旧 entry、旧 globalName、旧 SDK cache、旧 global entry 和旧 chunk 状态。

force register 不能在旧缓存清理失败后继续保持新注册并返回成功。第一版的失败语义是失败即回滚并抛错，避免调用方误以为注册替换和旧缓存失效已经完成。

这样手动 `clearCache` 和 force register 复用同一套底层清理能力，但语义不同：`clearCache` 保留注册，force register 替换注册。若未来需要“新 remote 加载成功后才切换”的强原子能力，应单独设计 `reloadRemote` 或更强的 force register 语义，不混入第一版 `clearCache`。

## 测试方案

### SSR remote 更新

构造 remote v1 和 v2：

1. SSR 首次加载 remote v1。
2. 调用 `clearCache({ name })`。
3. 保持 remote 注册不变，让当前 entry URL 返回 remote v2。
4. 再次加载同一 remote module。
5. 断言返回 v2 内容。

### force register 注册替换

构造 remote v1 和 v2 使用不同 entry URL：

1. SSR 首次加载 remote v1。
2. 调用 `registerRemotes([{ name, entry: v2Entry }], { force: true })`。
3. 断言注册已替换为 v2 entry。
4. 断言 `registerRemotes(..., { force: true })` 不提前请求 v2 remote entry。
5. 断言旧 SDK cache、旧 global entry、旧 chunk 状态使用 v1 的 `oldTarget` 清理，而不是按当前 v2 注册解析。
6. 断言 force register 的旧缓存清理不能通过公共 `clearCache({ name })` 实现。
7. 再次加载同一 remote module，断言此时才请求 v2 entry 并返回 v2 内容。
8. 如果 v2 加载失败，断言错误发生在 remote loading 阶段，不是 register 阶段。

### SSR 消费者缓存

构造 `pageA` 静态消费 `remoteA/Button`：

1. SSR 首次渲染 `pageA`，确认 `pageA` 和 `remoteA/Button` 都已进入缓存。
2. 更新 `remoteA`。
3. 调用 `clearCache({ name: "remoteA" })`。
4. 再次渲染 `pageA`。
5. 断言 `pageA` 的必要消费者链路重新执行，并返回新版 remote 内容。

### 消费者追溯边界

覆盖消费者反向追溯的边界：

- active 同步依赖：继续向上清理已执行 parent。
- dynamic import 或 async block：停止追溯。
- 未执行 module：停止追溯。
- external module 和 runtime module：停止追溯。
- 循环依赖：visited set 防止重复访问。
- 已执行入口模块：清入口 module cache，并验证 SSR handler cache 需要集成层配合清理。

### SSR handler cache

构造 SSR 服务层缓存入口 handler 的场景：

1. 首次请求从入口模块取出 render handler，并把 handler 保存在服务层变量里。
2. 更新 remote 并调用 `clearCache({ name })`。
3. 不清服务层 handler cache 时，再次请求仍可能使用旧引用，测试应记录为集成层未配合。
4. 清服务层 handler cache 或每次从 runtime 重新取入口后，再次请求应返回新版 remote 内容。

### remote reload after clear

覆盖 remote module 清理后的正常再次加载：

1. 首次加载 remote module，确认 bundler runtime 已写入 remote factory 和 exports。
2. 调用 `clearCache({ name })`。
3. 断言 `clearCache` 不提前请求 remote entry，也不提前执行 remote `get`。
4. 再次执行消费者模块。
5. 断言消费者链路重新执行，并重新进入正常 remote loading 流程。
6. 如果下一次 remote loading 失败，断言错误发生在 remote loading 阶段，不是 `clearCache` 阶段。
7. 断言不会因为 remote factory 缺失而同步报错。
8. 断言如果消费者同步 require remote module，runtime 已经先触发 chunk / remote ensure，或者保留了能触发 remote loading 的 remote module factory。

### pending load 与 stale 写回

覆盖正在加载的旧 remote：

- pending old load：`clearCache` 等待旧加载 settle，新的同名加载在 clear barrier 期间等待。
- timeout old load：等待超过内部上限后继续 clear，并把旧加载标记为 stale。
- stale old load：旧加载后续返回也不能写回 module factory、remote entry、shared 或 `installedChunks`，但已经在等待这个旧加载的旧 SSR 请求仍然可以拿到原本的成功结果或错误。
- next load：barrier 释放后，新请求按当前注册正常加载 remote。

### 重复更新与内存

连续执行 20 次 remote 更新：

- 每次更新后加载新 remote。
- 断言 MF runtime 的目标 remote cache 没有持续增长。
- 断言 bundler runtime 中目标 remote 的旧 module factory 和 exports 被替换。
- 断言消费者 module exports 没有持续堆积。
- 记录进程内存趋势，确认没有明显线性增长。

### shared safe 策略

覆盖三类 shared：

- 只注册未加载：`clearCache` 后被删除。
- 正在加载：`clearCache` 后旧 loading 完成也不再写回。
- 已加载：`clearCache` 后仍保留并可被后续请求复用。

### loadScriptNode

Node SSR 下分别覆盖：

- script remoteEntry：确认 `globalThis[remoteEntryKey]` 被清理。
- ESM remoteEntry root-only：没有依赖记录时，只确认已知 root URL 被清理。
- ESM remoteEntry dependency graph：第一版不记录 root 到 child URL 的依赖关系；后续如果 SDK Node loader 暴露依赖记录，再确认 root 和 descendants 都被清理。
- `sdkImportCache`：确认 `path`、`vm`、`node-fetch`、`node:module` 这类 Node 依赖不被误删。
- MF Node runtime plugin：当前 Rspack 仓库没有 `nodeRuntimeImportCache` 或 remote URL chunk 状态接入口；第一版只记录为不保证清理。

### 浏览器 script remote

在浏览器环境加载 script/global remote：

1. 加载 v1。
2. 调用 `clearCache({ name })`。
3. 断言 `clearCache` 不提前插入或加载新的 remote entry。
4. 如果 v2 使用同一个注册 entry，只有在下一次请求确实返回新内容时才断言拿到 v2。
5. 如果 v2 使用新的版本化 URL，先通过 `registerRemotes(..., { force: true })` 替换注册。
6. 断言下一次 remote loading 重新请求 remote entry，并返回 v2 内容。

### installedChunks 与并发

Node SSR 下覆盖：

- Rspack Node chunk loading runtime：确认目标 chunk loading 状态可被受控清理。
- loaded chunk：只清目标 remote 的旧 generation chunk。
- pending chunk：`clearCache` 等待 settle 后清理。
- timeout chunk：等待超过内部上限后继续 clear，旧结果后续返回也不能写回。
- stale chunk：旧加载结果在 generation 失效后不能写回 module factory 或 `installedChunks`。
- generation chunk：`clearCache` 推进 generation 后，旧 chunk、动态 import 和 cache 清理只命中旧 generation。
- 同名同 chunk id：断言不能只按 remote name 或 chunk id 清理，不能误删后续正常加载产生的新状态或非目标业务 chunk。
- 非目标业务 chunk：不受目标 remote 清理影响。

### Node cache

Node SSR 下分别覆盖：

- CommonJS filesystem chunk：确认相关 `require.cache` 被清。
- remote URL chunk：当前 Rspack 仓库没有可控接入口，第一版记录为不保证清理。
- ESM 同 URL：记录为不保证清理。
- ESM root 版本化 URL：确认 clear 后下一次 Node loader 使用新 URL。

## 风险与边界

- `clearCache` 只保证后续加载走新 remote，不保证当前正在执行的请求立即切换。
- `clearCache` 保留 remote 注册；需要替换 remote 地址时必须使用 `registerRemotes(..., { force: true })`。
- 同 URL 更新只有在下一次请求确实拿到新内容时才成立；浏览器缓存、Service Worker、HTTP 代理缓存、Node fetch cache 和 ESM 同 URL 都可能继续返回旧内容。
- `clearCache` 不提前加载或验证新 remote；下一次 remote loading 失败不应归因于 clear 阶段。
- clear barrier 必须在 `finally` 中释放，清理失败、超时或抛错都不能永久阻塞同名 remote。
- 业务代码已经保存的旧 remote 或旧消费者引用无法由 runtime 强制回收。
- SSR 服务层如果缓存入口 render handler、路由 handler 或 page module 引用，需要在 `clearCache` 后同步清自己的缓存。
- SSR 消费者追溯可能清理到入口模块，但这是保证正确性的必要边界。
- 浏览器默认不做广泛消费者追溯，可能需要业务刷新页面或使用版本化动态边界。
- loaded shared 默认保留，可能意味着某些 shared 不会随 remote 版本一起更新。
- 如果业务要求 shared 也随 remote 更新，需要通过版本化 shared 或未来新增显式危险选项处理。
- ESM 同 URL 缓存不应被宣传为可清理能力。没有 root 到 child URL 依赖记录时，第一版只清已知 root URL。
- remote URL chunk 和 `nodeRuntimeImportCache` 不在当前 Rspack runtime 可控范围内，第一版不宣传这部分可清理；后续等 MF Node runtime plugin 暴露 adapter 后再补。

## 实施 Roadmap

### Phase 1: 锁定 SSR 最小用例

用例已落在 `tests/rspack-test/serialCases/container/mf-ssr-clear-cache`，并已纳入执行。

- [x] 增加 SSR remote 更新用例：首次请求 remote v1，`clearCache({ name })` 后再次请求 remote v2。
- [x] 增加多 expose / 多路由用例：`pageA` 消费 `remoteA/A`，`pageB` 消费 `remoteA/B`，清理 `remoteA` 后两个路由的后续请求都重新加载。
- [x] 增加消费者缓存用例：确认已执行过的消费者链路会在后续请求中重新执行。
- [x] 增加 pending old load 用例：`clearCache` 等待旧加载或把超时旧加载标记为 stale。
- [x] 增加 no-preload 用例：确认 `clearCache` 不提前请求 remote entry，也不提前执行 remote `get`。

### Phase 2: 补齐 bundler runtime 清理基础

- [x] 在 Rspack remote runtime metadata 中补充 remote 到 remote module、external module、consumer module 和 chunk 的索引。
- [x] 增加 remote 级别 clear adapter，负责失效旧 remote module 状态、旧 loading 状态和旧 factory 写回状态。
- [x] 清理 external module exports，让 remote entry 或 container 后续能重新加载。
- [x] 清理受影响 SSR 消费者 module exports，让后续请求重新执行消费链路。
- [x] 保证清理后不会出现消费者同步 `require` remote module 时 factory 缺失的状态。

### Phase 3: 接入 MF runtime 公共入口

- [x] 在 MF runtime 中公开 `clearCache({ name })`。
- [x] 拆开“保留注册的缓存清理”和“移除 remote 注册”两种语义。
- [x] 增加 remote barrier，让同名 remote 的新加载在清理期间等待。
- [x] 增加 remote generation，防止 stale old load 后续写回 runtime cache。
- [x] 在 clear 前保存旧目标和旧状态快照；clear 自身失败时停止后续流程并抛错。

### Phase 4: 补齐 Node、register 和浏览器边界

- [x] 清理 runtime-core Node remoteEntry loading promise 和 `loadScriptNode` 写入的全局 remote entry。
- [x] 在 Node loader 链路增加 `loadScriptNode` ESM module cache clear adapter，通过 root URL versioning 避开 SDK 内部 ESM cache。
- [x] 为 ESM remoteEntry 补 root URL 清理；第一版不记录 root 到 child URL 的依赖关系。
- [x] 在 Rspack 生成的 Node chunk loading runtime 中暴露受控清理入口，并由 MF runtime 按 remote chunk id 串联清理。
- [x] 在 Rspack 生成的 Node chunk loading runtime 中补充 remote generation 状态。
- [x] 在 readFileVm pending chunk load 中完成并验证 generation 检查点。
- [x] 在 Rspack 生成的 `require` chunk loading runtime 中清理目标 chunk 的 CommonJS `require.cache`。
- [x] 记录 remote URL chunk、`nodeRuntimeImportCache` 或等价动态 import 缓存当前不在 Rspack 可控范围，第一版不声明支持。
- [x] 让 `registerRemotes(..., { force: true })` 保存 `oldTarget`，并用内部 clear adapter 清旧缓存；失败时回滚注册并 reject。
- [x] 浏览器只做保守清理，不做广泛消费者追溯，也不提前加载新的 script/global remote entry。

### Phase 5: 补齐验收测试与风险覆盖

- [x] 覆盖 `registerRemotes(..., { force: true })` 不提前请求新 remote，并用 `oldTarget` 清旧缓存。
- [x] 覆盖 stale old remote load 后续返回时不污染后续 remote module factory 和 remote entry 缓存。
- [x] 覆盖 remote 已加载 chunk 在 `clearCache` 后清理 `installedChunks` 状态并推进 generation。
- [x] 覆盖 readFileVm pending chunk：`clearCache` 等待旧 chunk settle 后再清理。
- [x] 覆盖 stale old chunk load 后续返回时不写回 module factory 或 `installedChunks`。
- [x] 覆盖 stale old shared load 后续返回时不写回 shared 缓存。
- [x] 覆盖 shared safe 策略中的未加载可清、已加载保留、加载中旧结果不写回。
- [x] 覆盖 already waiting old SSR request 仍能拿到旧加载的成功结果或错误。
- [x] 覆盖 repeated clear / reload 后可控缓存数量没有明显持续增长。
- [x] 覆盖 ESM remoteEntry clear 后使用版本化 root URL，避免复用同 URL SDK ESM cache。
- [x] 覆盖 CommonJS filesystem chunk 在 `clearCache` 后清理 `require.cache`。
- [x] 记录 remote URL chunk 和 ESM child URL 不保证清理；ESM root URL 通过版本化 URL 触发后续重新加载。

## 结论

`clearCache` 应该是 MF runtime 的公共能力，而不是 bundler runtime、SDK Node loader 或 Node runtime plugin 的业务 API。

MF runtime 负责定义语义和清理顺序；bundler runtime 负责清 remote module 和受影响消费者；SDK Node loader 负责清 remoteEntry 加载缓存；Node runtime plugin 负责清 SSR Node 侧 chunk 与模块缓存。

这次方案的关键变化是：`clearCache` 只清缓存并保留 remote 注册；remote 更新不是只清生产者缓存，还必须让已经缓存的消费者链路在下一次 SSR 请求中重新执行。shared 第一版必须保守：未加载可清，已加载保留。这样能解决 remote 更新后的旧缓存残留问题，同时避免破坏 SSR 进程内的 shared 单例稳定性。
