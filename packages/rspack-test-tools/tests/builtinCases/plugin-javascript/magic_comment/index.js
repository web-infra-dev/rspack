import(/* webpackChunkName: "normal" */ "./normal");
import(/* webpackChunkName: "with.dot" */ "./with_dot");
import(/* webpackChunkName: "./sub/fold" */ "./sub_fold");
import(/* webpackChunkName: './sub/single' */ "./single_quote");

import(/* webpackChunkName: `./sub/other` */ "./other");
import(/* webpackChunkName: "./user/[id]" */ "./user/1");
import(/* webpackChunkName: `user/[id]/page`*/ "./user/page/2");
import(/* webpackChunkName: `user/(id)/page`*/ "./user/page/3");

import(/* 'bug_' */ "./bug_only_single_quote.js");
