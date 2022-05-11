import * as turf from '@turf/turf';

export function keepMapRatio(mapData, c) {
  if (mapData && turf) {
    // 获取数据外接矩形，计算宽高比
    const bbox = turf.bbox(mapData);
    const width = bbox[2] - bbox[0];
    const height = bbox[3] - bbox[1];
    const ratio = height / width;

    const cWidth = c.width;
    const cHeight = c.height;
    const cRatio = cHeight / cWidth;

    let scale: {
      x: {
        range: number[];
      };
      y: {
        range: number[];
      };
    };

    if (cRatio >= ratio) {
      const halfDisRatio = (cRatio - ratio) / 2 / cRatio;
      scale = {
        x: {
          range: [0, 1],
        },
        y: {
          range: [halfDisRatio, 1 - halfDisRatio],
        },
      };
    } else {
      const halfDisRatio = ((1 / cRatio - 1 / ratio) / 2) * cRatio;
      scale = {
        y: {
          range: [0, 1],
        },
        x: {
          range: [halfDisRatio, 1 - halfDisRatio],
        },
      };
    }
    const curScaleXRange = c.getScaleByField('x').range;
    const curScaleYRange = c.getScaleByField('y').range;

    if (
      curScaleXRange[0] !== scale.x.range[0] ||
      curScaleXRange[1] !== scale.x.range[1] ||
      curScaleYRange[0] !== scale.y.range[0] ||
      curScaleYRange[1] !== scale.y.range[1]
    ) {
      setTimeout(() => {
        c.scale(scale);
        c.render(true);
      }, 1);
    }
  }
}
