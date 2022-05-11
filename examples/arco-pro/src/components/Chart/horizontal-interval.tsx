import React from 'react';
import { Chart, Tooltip, Interval, Axis, Coordinate, G2 } from 'bizcharts';
import { Spin } from '@arco-design/web-react';
import CustomTooltip from './customer-tooltip';

function HorizontalInterval({
  data,
  loading,
  height,
}: {
  data: any[];
  loading: boolean;
  height?: number;
}) {
  G2.registerShape('interval', 'border-radius', {
    draw(cfg, container) {
      const points = cfg.points as unknown as { x: string; y: number };
      let path = [];
      path.push(['M', points[0].x, points[0].y]);
      path.push(['L', points[1].x, points[1].y]);
      path.push(['L', points[2].x, points[2].y]);
      path.push(['L', points[3].x, points[3].y]);
      path.push('Z');
      path = this.parsePath(path); // 将 0 - 1 转化为画布坐标

      const group = container.addGroup();
      const radius = (path[1][2] - path[2][2]) / 2;
      group.addShape('rect', {
        attrs: {
          x: path[0][1], // 矩形起始点为左上角
          y: path[0][2] - radius * 2,
          width: path[1][1] - path[0][1],
          height: path[1][2] - path[2][2],
          fill: cfg.color,
          radius: radius,
        },
      });
      return group;
    },
  });

  return (
    <Spin loading={loading} style={{ width: '100%' }}>
      <Chart
        height={height || 370}
        padding="auto"
        data={data}
        autoFit
        className={'chart-wrapper'}
      >
        <Coordinate transpose />
        <Interval
          color="#4086FF"
          position="name*count"
          size={10}
          shape="border-radius"
        />
        <Tooltip>
          {(title, items) => {
            return <CustomTooltip title={title} data={items} />;
          }}
        </Tooltip>
        <Axis
          name="count"
          label={{
            formatter(text) {
              return `${Number(text) / 1000}k`;
            },
          }}
        />
      </Chart>
    </Spin>
  );
}

export default HorizontalInterval;
