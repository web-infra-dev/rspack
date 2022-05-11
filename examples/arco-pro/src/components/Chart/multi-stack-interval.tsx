import React from 'react';
import { Chart, Tooltip, Interval, Axis, Legend } from 'bizcharts';
import { Spin } from '@arco-design/web-react';
import CustomTooltip from './customer-tooltip';

function MultiInterval({ data, loading }: { data: any[]; loading: boolean }) {
  return (
    <Spin loading={loading} style={{ width: '100%' }}>
      <Chart
        height={370}
        padding="auto"
        data={data}
        autoFit
        className={'chart-wrapper'}
      >
        <Interval
          adjust="stack"
          color={['name', ['#81E2FF', '#00B2FF', '#246EFF']]}
          position="time*count"
          size={16}
          style={{
            radius: [2, 2, 0, 0],
          }}
        />
        <Tooltip crosshairs={{ type: 'x' }} showCrosshairs shared>
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
        <Legend name="name" marker={{ symbol: 'circle' }} />
      </Chart>
    </Spin>
  );
}

export default MultiInterval;
