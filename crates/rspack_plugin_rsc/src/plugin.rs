// server 中，解析到 'use client' 后，
// 解析为 content 为 SSR 组件，
// 解析为 ast 过程中，可以发现 'use client'
// 这个模块需要特殊处理，则替换模块 source 为 register 函数 client reference
// 父模块新增对这个模块的 async dependency block
