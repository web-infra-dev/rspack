import React from 'react';
import { Chart, Line, Axis, Tooltip, Legend, Slider } from 'bizcharts';
import { Spin } from '@arco-design/web-react';
import CustomTooltip from './customer-tooltip';
import useBizTheme from '@/utils/useChartTheme';

const lineColor = ['#21CCFF', '#313CA9', '#249EFF'];
function PeriodLine({ data, loading }: { data: any[]; loading: boolean }) {
  return (
    <Spin loading={loading} style={{ width: '100%' }}>
      <Chart
        theme={useBizTheme()}
        forceUpdate
        height={370}
        padding={[10, 20, 120, 60]}
        data={data}
        autoFit
        scale={{ time: 'time' }}
        className={'chart-wrapper'}
      >
        <Line shape="smooth" position="time*rate" color={['name', lineColor]} />
        <Tooltip crosshairs={{ type: 'x' }} showCrosshairs shared>
          {(title, items) => {
            return <CustomTooltip title={title} data={items} />;
          }}
        </Tooltip>
        <Axis
          name="rate"
          label={{
            formatter(text) {
              return `${Number(text)} %`;
            },
          }}
        />
        <Legend
          name="name"
          marker={(_, index) => {
            return {
              symbol: 'circle',
              style: {
                fill: lineColor[index],
                r: 4,
              },
            };
          }}
        />
        <Slider
          foregroundStyle={{
            borderRadius: ' 4px',
            fill: 'l (180) 0:rgba(206, 224, 255, 0.9) 1:rgba(146, 186, 255, 0.8)',
            opacity: 0.3,
          }}
          trendCfg={{
            data: data.map((item) => item.rate),
            isArea: true,
            areaStyle: {
              fill: 'rgba(4, 135, 255, 0.15)',
              opacity: 1,
            },
            backgroundStyle: {
              fill: '#F2F3F5',
            },
            lineStyle: {
              stroke: 'rgba(36, 158, 255, 0.3)',
              lineWidth: 2,
            },
          }}
          handlerStyle={{
            fill: '#ffffff',
            opacity: 1,
            width: 22,
            height: 22,
            stroke: '#165DFF',
          }}
        />
      </Chart>
    </Spin>
  );
}

export default PeriodLine;
