import React from 'react';
import {
  Chart,
  Line,
  Axis,
  Area,
  Tooltip,
  Coordinate,
  Legend,
} from 'bizcharts';
import CustomTooltip from './customer-tooltip';
import { Spin } from '@arco-design/web-react';
import DataSet from '@antv/data-set';

interface AreaPolarProps {
  data: any[];
  loading: boolean;
  fields: string[];
  height: number;
}
function AreaPolar(props: AreaPolarProps) {
  const { data, loading, fields, height } = props;

  const { DataView } = DataSet;
  const dv = new DataView().source(data);
  dv.transform({
    type: 'fold',
    fields: fields, // 展开字段集
    key: 'category', // key字段
    value: 'score', // value字段
  });

  return (
    <Spin loading={loading} style={{ width: '100%' }}>
      <Chart
        height={height || 400}
        padding={0}
        data={dv.rows}
        autoFit
        scale={{
          score: {
            min: 0,
            max: 80,
          },
        }}
        interactions={['legend-highlight']}
        className={'chart-wrapper'}
      >
        <Coordinate type="polar" radius={0.8} />
        <Tooltip shared>
          {(title, items) => {
            return <CustomTooltip title={title} data={items} />;
          }}
        </Tooltip>
        <Line
          position="item*score"
          size="2"
          color={['category', ['#313CA9', '#21CCFF', '#249EFF']]}
        />
        <Area
          position="item*score"
          tooltip={false}
          color={[
            'category',
            [
              'rgba(49, 60, 169, 0.4)',
              'rgba(33, 204, 255, 0.4)',
              'rgba(36, 158, 255, 0.4)',
            ],
          ]}
        />
        <Axis name="score" label={false} />
        <Legend
          position="right"
          marker={(_, index) => {
            return {
              symbol: 'circle',
              style: {
                r: 4,
                lineWidth: 0,
                fill: ['#313CA9', '#21CCFF', '#249EFF'][index],
              },
            };
          }}
          name="category"
        />
      </Chart>
    </Spin>
  );
}

export default AreaPolar;
