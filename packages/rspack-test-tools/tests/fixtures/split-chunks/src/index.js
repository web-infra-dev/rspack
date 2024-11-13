import { createElement } from "react";
import { render } from "react-dom";
import { fiftyK } from "./50k";
import { fiftyK as fiftyK_1 } from "./50k-1";
import { fiftyK as fiftyK_2 } from "./50k-2";
import { fiftyK as fiftyK_3 } from "./50k-3";
import { fiftyK as fiftyK_4 } from "./50k-4";
import { fiftyK as fiftyK_5 } from "./50k-5";
import { fiftyK as fiftyK_6 } from "./50k-6";
import { fiftyK as fiftyK_7 } from "./50k-7";
import { fiftyK as fiftyK_8 } from "./50k-8";

import {
  fiftyK1,
  fiftyK2,
  fiftyK3,
  fiftyK4,
  fiftyK5,
  fiftyK6,
  fiftyK7,
  fiftyK8,
} from "./400k";

import axios from "axios";
import dayjs from "dayjs";
import * as ReactUse from "react-use";
import * as ahooks from "ahooks";

window.lib = {
  axios,
  dayjs,
  ReactUse,
  ahooks,
  fiftyK,
  fiftyK1,
  fiftyK2,
  fiftyK3,
  fiftyK4,
  fiftyK5,
  fiftyK6,
  fiftyK7,
  fiftyK8,
  fiftyK_1,
  fiftyK_2,
  fiftyK_3,
  fiftyK_4,
  fiftyK_5,
  fiftyK_6,
  fiftyK_7,
  fiftyK_8,
};

render(
  createElement("div", null, [
    createElement("h1", null, `Hello World  ${__BUNDLE__}`),
    createElement("p", null, "see more detail in devtool/network"),
  ]),
  document.getElementById("root")
);
