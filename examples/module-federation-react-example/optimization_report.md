# Module Federation Optimization Report

Generated: 2025-08-19T00:54:33.568Z

## Summary

- **Total Original Size:** 34.49 MB
- **Total Optimized Size:** 21.16 MB
- **Total Size Saved:** 13.34 MB (38.66%)
- **Total Modules Analyzed:** 5887
- **Total Modules Pruned:** 4019

## Detailed Results by Library

### react-chartjs-2

#### host/node_modules_pnpm_react-chartjs-2_5_3_0_chart_js_4_5_0_react_18_3_1_node_modules_react-chartj-c8072e1.js

- **Original Size:** 12,584 bytes
- **Optimized Size:** 13,085 bytes
- **Size Reduction:** -3.98%
- **Status:** Skipped (No modules pruned from 1 pushes)

#### remote/node_modules_pnpm_react-chartjs-2_5_3_0_chart_js_4_5_0_react_18_3_1_node_modules_react-chartj-c8072e0.3db0018e57efedc9.js

- **Original Size:** 12,588 bytes
- **Optimized Size:** 13,089 bytes
- **Size Reduction:** -3.98%
- **Status:** Skipped (No modules pruned from 1 pushes)

### antd

#### host/node_modules_pnpm_antd_5_27_0_date-fns_4_1_0_react-dom_18_3_1_react_18_3_1\_\_react_18_3_1_node-45059f0.js

- **Original Size:** 8,580,171 bytes
- **Optimized Size:** 6,325,195 bytes
- **Size Reduction:** 26.28%
- **Module Pruning:**
  - Original Modules: 1369
  - Modules Kept: 932
  - Modules Removed: 437

**Removed Modules:**

- `../../../node_modules/.pnpm/@ant-design+fast-color@2.0.6/node_modules/@ant-design/fast-color/es/types.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/DeleteOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/DownloadOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/FileTextOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/FileTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/PaperClipOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/PictureTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/RotateLeftOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/RotateRightOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/StarFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/SwapOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/UpOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/VerticalAlignTopOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/WarningFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ZoomInOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ZoomOutOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/DeleteOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/DownloadOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/FileTextOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/FileTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/PaperClipOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/PictureTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/RotateLeftOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/RotateRightOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/StarFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/SwapOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/UpOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/VerticalAlignTopOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/WarningFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/ZoomInOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons@5.6.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@ant-design/icons/es/icons/ZoomOutOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/arrows.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/default-props.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/dots.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/index.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/initial-state.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/inner-slider.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/slider.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/track.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/utils/innerSliderUtils.js`
- `../../../node_modules/.pnpm/@rc-component+async-validator@5.0.4/node_modules/@rc-component/async-validator/es/interface.js`
- `../../../node_modules/.pnpm/@rc-component+color-picker@2.0.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/color-picker/es/interface.js`
- `../../../node_modules/.pnpm/@rc-component+mini-decimal@1.1.0/node_modules/@rc-component/mini-decimal/es/BigIntDecimal.js`
- `../../../node_modules/.pnpm/@rc-component+mini-decimal@1.1.0/node_modules/@rc-component/mini-decimal/es/MiniDecimal.js`
- `../../../node_modules/.pnpm/@rc-component+mini-decimal@1.1.0/node_modules/@rc-component/mini-decimal/es/NumberDecimal.js`
- `../../../node_modules/.pnpm/@rc-component+mini-decimal@1.1.0/node_modules/@rc-component/mini-decimal/es/index.js`
- `../../../node_modules/.pnpm/@rc-component+mini-decimal@1.1.0/node_modules/@rc-component/mini-decimal/es/numberUtil.js`
- `../../../node_modules/.pnpm/@rc-component+mini-decimal@1.1.0/node_modules/@rc-component/mini-decimal/es/supportUtil.js`
- `../../../node_modules/.pnpm/@rc-component+mutate-observer@1.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/mutate-observer/es/MutateObserver.js`
- `../../../node_modules/.pnpm/@rc-component+mutate-observer@1.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/mutate-observer/es/index.js`

... and 387 more removed modules

**Kept Modules:** 932 modules retained

#### remote/vendors-node*modules_pnpm_antd_5_27_0_date-fns_4_1_0_react-dom_18_3_1_react_18_3_1\_\_react_18*-b5c5a3.a4f624a087559709.js

- **Original Size:** 8,195,723 bytes
- **Optimized Size:** 5,987,619 bytes
- **Size Reduction:** 26.94%
- **Module Pruning:**
  - Original Modules: 1233
  - Modules Kept: 827
  - Modules Removed: 406

**Removed Modules:**

- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/arrows.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/default-props.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/dots.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/index.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/initial-state.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/inner-slider.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/slider.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/track.js`
- `../../../node_modules/.pnpm/@ant-design+react-slick@1.1.2_react@18.3.1/node_modules/@ant-design/react-slick/es/utils/innerSliderUtils.js`
- `../../../node_modules/.pnpm/@rc-component+async-validator@5.0.4/node_modules/@rc-component/async-validator/es/interface.js`
- `../../../node_modules/.pnpm/@rc-component+color-picker@2.0.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/color-picker/es/interface.js`
- `../../../node_modules/.pnpm/@rc-component+mini-decimal@1.1.0/node_modules/@rc-component/mini-decimal/es/BigIntDecimal.js`
- `../../../node_modules/.pnpm/@rc-component+mini-decimal@1.1.0/node_modules/@rc-component/mini-decimal/es/MiniDecimal.js`
- `../../../node_modules/.pnpm/@rc-component+mini-decimal@1.1.0/node_modules/@rc-component/mini-decimal/es/NumberDecimal.js`
- `../../../node_modules/.pnpm/@rc-component+mini-decimal@1.1.0/node_modules/@rc-component/mini-decimal/es/index.js`
- `../../../node_modules/.pnpm/@rc-component+mini-decimal@1.1.0/node_modules/@rc-component/mini-decimal/es/numberUtil.js`
- `../../../node_modules/.pnpm/@rc-component+mini-decimal@1.1.0/node_modules/@rc-component/mini-decimal/es/supportUtil.js`
- `../../../node_modules/.pnpm/@rc-component+mutate-observer@1.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/mutate-observer/es/MutateObserver.js`
- `../../../node_modules/.pnpm/@rc-component+mutate-observer@1.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/mutate-observer/es/index.js`
- `../../../node_modules/.pnpm/@rc-component+mutate-observer@1.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/mutate-observer/es/useMutateObserver.js`
- `../../../node_modules/.pnpm/@rc-component+mutate-observer@1.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/mutate-observer/es/wrapper.js`
- `../../../node_modules/.pnpm/@rc-component+qrcode@1.0.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/qrcode/es/QRCodeCanvas.js`
- `../../../node_modules/.pnpm/@rc-component+qrcode@1.0.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/qrcode/es/QRCodeSVG.js`
- `../../../node_modules/.pnpm/@rc-component+qrcode@1.0.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/qrcode/es/hooks/useQRCode.js`
- `../../../node_modules/.pnpm/@rc-component+qrcode@1.0.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/qrcode/es/index.js`
- `../../../node_modules/.pnpm/@rc-component+qrcode@1.0.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/qrcode/es/libs/qrcodegen.js`
- `../../../node_modules/.pnpm/@rc-component+qrcode@1.0.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/qrcode/es/utils.js`
- `../../../node_modules/.pnpm/@rc-component+tour@1.15.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/tour/es/Mask.js`
- `../../../node_modules/.pnpm/@rc-component+tour@1.15.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/tour/es/Tour.js`
- `../../../node_modules/.pnpm/@rc-component+tour@1.15.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/tour/es/TourStep/DefaultPanel.js`
- `../../../node_modules/.pnpm/@rc-component+tour@1.15.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/tour/es/TourStep/index.js`
- `../../../node_modules/.pnpm/@rc-component+tour@1.15.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/tour/es/hooks/useClosable.js`
- `../../../node_modules/.pnpm/@rc-component+tour@1.15.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/tour/es/hooks/useTarget.js`
- `../../../node_modules/.pnpm/@rc-component+tour@1.15.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/tour/es/index.js`
- `../../../node_modules/.pnpm/@rc-component+tour@1.15.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/tour/es/placements.js`
- `../../../node_modules/.pnpm/@rc-component+tour@1.15.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/@rc-component/tour/es/util.js`
- `../../../node_modules/.pnpm/antd@5.27.0_date-fns@4.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/antd/es/_util/ActionButton.js`
- `../../../node_modules/.pnpm/antd@5.27.0_date-fns@4.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/antd/es/_util/hooks/usePatchElement.js`
- `../../../node_modules/.pnpm/antd@5.27.0_date-fns@4.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/antd/es/_util/styleChecker.js`
- `../../../node_modules/.pnpm/antd@5.27.0_date-fns@4.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/antd/es/_util/throttleByAnimationFrame.js`
- `../../../node_modules/.pnpm/antd@5.27.0_date-fns@4.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/antd/es/_util/transKeys.js`
- `../../../node_modules/.pnpm/antd@5.27.0_date-fns@4.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/antd/es/affix/index.js`
- `../../../node_modules/.pnpm/antd@5.27.0_date-fns@4.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/antd/es/affix/style/index.js`
- `../../../node_modules/.pnpm/antd@5.27.0_date-fns@4.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/antd/es/affix/utils.js`
- `../../../node_modules/.pnpm/antd@5.27.0_date-fns@4.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/antd/es/anchor/Anchor.js`
- `../../../node_modules/.pnpm/antd@5.27.0_date-fns@4.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/antd/es/anchor/AnchorLink.js`
- `../../../node_modules/.pnpm/antd@5.27.0_date-fns@4.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/antd/es/anchor/context.js`
- `../../../node_modules/.pnpm/antd@5.27.0_date-fns@4.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/antd/es/anchor/index.js`
- `../../../node_modules/.pnpm/antd@5.27.0_date-fns@4.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/antd/es/anchor/style/index.js`
- `../../../node_modules/.pnpm/antd@5.27.0_date-fns@4.1.0_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/antd/es/app/App.js`

... and 356 more removed modules

**Kept Modules:** 827 modules retained

### @reduxjs/toolkit

#### host/node_modules_pnpm_reduxjs_toolkit_2_8_2_react-redux_9_2_0\_\_types_react_18_3_23_react_18_3_1_r-b7a319.js

- **Original Size:** 167,059 bytes
- **Optimized Size:** 125,395 bytes
- **Size Reduction:** 24.94%
- **Module Pruning:**
  - Original Modules: 5
  - Modules Kept: 4
  - Modules Removed: 1

**Removed Modules:**

- `../../../node_modules/.pnpm/reselect@5.1.1/node_modules/reselect/dist/reselect.mjs`

**Kept Modules:** 4 modules retained

#### remote/vendors-node*modules_pnpm_reduxjs_toolkit_2_8_2_react-redux_9_2_0\_\_types_react_18_3_23_react*-3c49cc.cf30e018de44fba2.js

- **Original Size:** 167,063 bytes
- **Optimized Size:** 125,399 bytes
- **Size Reduction:** 24.94%
- **Module Pruning:**
  - Original Modules: 5
  - Modules Kept: 4
  - Modules Removed: 1

**Removed Modules:**

- `../../../node_modules/.pnpm/reselect@5.1.1/node_modules/reselect/dist/reselect.mjs`

**Kept Modules:** 4 modules retained

### chart.js

#### host/node_modules_pnpm_chart_js_4_5_0_node_modules_chart_js_dist_chart_js.js

- **Original Size:** 739,847 bytes
- **Optimized Size:** 942,815 bytes
- **Size Reduction:** -27.43%
- **Status:** Skipped (No modules pruned from 1 pushes)

#### remote/vendors-node_modules_pnpm_chart_js_4_5_0_node_modules_chart_js_dist_chart_js.36d3ee2ca889dd63.js

- **Original Size:** 739,859 bytes
- **Optimized Size:** 942,827 bytes
- **Size Reduction:** -27.43%
- **Status:** Skipped (No modules pruned from 1 pushes)

### react-dom

#### host/node_modules_pnpm_react-dom_18_3_1_react_18_3_1_node_modules_react-dom_index_js-\_47d21.js

- **Original Size:** 1,354,126 bytes
- **Optimized Size:** 1,652,057 bytes
- **Size Reduction:** -22.00%
- **Status:** Skipped (No modules pruned from 1 pushes)

#### remote/vendors-node_modules_pnpm_react-dom_18_3_1_react_18_3_1_node_modules_react-dom_index_js.48dbc16582969103.js

- **Original Size:** 1,354,131 bytes
- **Optimized Size:** 1,652,062 bytes
- **Size Reduction:** -22.00%
- **Status:** Skipped (No modules pruned from 1 pushes)

### lodash-es

#### host/node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.js

- **Original Size:** 1,532,669 bytes
- **Optimized Size:** 1,564,039 bytes
- **Size Reduction:** -2.05%
- **Status:** Skipped (No modules pruned from 1 pushes)

#### remote/vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.109c234c81309e95.js

- **Original Size:** 1,532,681 bytes
- **Optimized Size:** 1,564,051 bytes
- **Size Reduction:** -2.05%
- **Status:** Skipped (No modules pruned from 1 pushes)

### @ant-design/icons

#### host/node_modules_pnpm_ant-design_icons_5_6_1_react-dom_18_3_1_react_18_3_1\_\_react_18_3_1_node_mod-feb44f0.js

- **Original Size:** 5,672,537 bytes
- **Optimized Size:** 312,352 bytes
- **Size Reduction:** 94.49%
- **Module Pruning:**
  - Original Modules: 1702
  - Modules Kept: 64
  - Modules Removed: 1638

**Removed Modules:**

- `../../../node_modules/.pnpm/@ant-design+fast-color@2.0.6/node_modules/@ant-design/fast-color/es/types.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AccountBookFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AccountBookOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AccountBookTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AimOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlertFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlertOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlertTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlibabaOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlignCenterOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlignLeftOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlignRightOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlipayCircleFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlipayCircleOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlipayOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlipaySquareFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AliwangwangFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AliwangwangOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AliyunOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AmazonCircleFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AmazonOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AmazonSquareFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AndroidFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AndroidOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AntCloudOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AntDesignOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ApartmentOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ApiFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ApiOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ApiTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AppleFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AppleOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AppstoreAddOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AppstoreFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AppstoreTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AreaChartOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ArrowLeftOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ArrowRightOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ArrowsAltOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AudioFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AudioMutedOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AudioOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AudioTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AuditOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/BackwardFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/BackwardOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/BaiduOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/BankFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/BankOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/BankTwoTone.js`

... and 1588 more removed modules

**Kept Modules:** 64 modules retained

#### remote/vendors-node*modules_pnpm_ant-design_icons_5_6_1_react-dom_18_3_1_react_18_3_1\_\_react_18_3_1*-e996c6.5bacf7c70887b33b.js

- **Original Size:** 5,266,179 bytes
- **Optimized Size:** 183,005 bytes
- **Size Reduction:** 96.52%
- **Module Pruning:**
  - Original Modules: 1566
  - Modules Kept: 34
  - Modules Removed: 1532

**Removed Modules:**

- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AccountBookFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AccountBookOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AccountBookTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AimOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlertFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlertOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlertTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlibabaOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlignCenterOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlignLeftOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlignRightOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlipayCircleFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlipayCircleOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlipayOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AlipaySquareFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AliwangwangFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AliwangwangOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AliyunOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AmazonCircleFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AmazonOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AmazonSquareFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AndroidFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AndroidOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AntCloudOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AntDesignOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ApartmentOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ApiFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ApiOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ApiTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AppleFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AppleOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AppstoreAddOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AppstoreFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AppstoreTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AreaChartOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ArrowLeftOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ArrowRightOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/ArrowsAltOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AudioFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AudioMutedOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AudioOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AudioTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/AuditOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/BackwardFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/BackwardOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/BaiduOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/BankFilled.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/BankOutlined.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/BankTwoTone.js`
- `../../../node_modules/.pnpm/@ant-design+icons-svg@4.4.2/node_modules/@ant-design/icons-svg/es/asn/BarcodeOutlined.js`

... and 1482 more removed modules

**Kept Modules:** 34 modules retained

### react-router-dom

#### host/node_modules_pnpm_react-router-dom_7_8_1_react-dom_18_3_1_react_18_3_1\_\_react_18_3_1_node_mod-e31d020.js

- **Original Size:** 539,492 bytes
- **Optimized Size:** 444,887 bytes
- **Size Reduction:** 17.54%
- **Module Pruning:**
  - Original Modules: 7
  - Modules Kept: 3
  - Modules Removed: 4

**Removed Modules:**

- `../../../node_modules/.pnpm/cookie@1.0.2/node_modules/cookie/dist/index.js`
- `../../../node_modules/.pnpm/set-cookie-parser@2.7.1/node_modules/set-cookie-parser/lib/set-cookie.js`
- `../../../node_modules/.pnpm/react-router@7.8.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/react-router/dist/development/chunk-IFMMFE4R.mjs`
- `../../../node_modules/.pnpm/react-router@7.8.1_react-dom@18.3.1_react@18.3.1__react@18.3.1/node_modules/react-router/dist/development/dom-export.mjs`

**Kept Modules:** 3 modules retained

### react

#### host/node_modules_pnpm_react_18_3_1_node_modules_react_index_js.js

- **Original Size:** 114,099 bytes
- **Optimized Size:** 138,140 bytes
- **Size Reduction:** -21.07%
- **Status:** Skipped (No modules pruned from 1 pushes)

#### remote/vendors-node_modules_pnpm_react_18_3_1_node_modules_react_index_js.1a04e81ff105870e.js

- **Original Size:** 114,111 bytes
- **Optimized Size:** 138,152 bytes
- **Size Reduction:** -21.07%
- **Status:** Skipped (No modules pruned from 1 pushes)

### react-redux

#### host/node_modules_pnpm_react-redux_9_2_0\_\_types_react_18_3_23_react_18_3_1_redux_5_0_1_node_module-c6d9df1.js

- **Original Size:** 45,842 bytes
- **Optimized Size:** 24,649 bytes
- **Size Reduction:** 46.23%
- **Status:** Skipped (No modules pruned from 1 pushes)

### dayjs

#### host/node_modules_pnpm_dayjs_1_11_13_node_modules_dayjs_dayjs_min_js.js

- **Original Size:** 14,718 bytes
- **Optimized Size:** 19,086 bytes
- **Size Reduction:** -29.68%
- **Status:** Skipped (No modules pruned from 1 pushes)

#### remote/vendors-node_modules_pnpm_dayjs_1_11_13_node_modules_dayjs_dayjs_min_js.376dd385a4efc4d8.js

- **Original Size:** 14,730 bytes
- **Optimized Size:** 19,098 bytes
- **Size Reduction:** -29.65%
- **Status:** Skipped (No modules pruned from 1 pushes)
