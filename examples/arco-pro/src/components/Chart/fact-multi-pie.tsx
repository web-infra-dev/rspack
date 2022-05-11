import React from 'react';
import { Chart, Legend, Facet } from 'bizcharts';
import useBizTheme from '@/utils/useChartTheme';

interface FactMultiPieProps {
  data: any[];
  loading: boolean;
  height: number;
}
function FactMultiPie(props: FactMultiPieProps) {
  return (
    <Chart
      theme={useBizTheme()}
      forceUpdate
      autoFit
      data={props.data}
      height={props.height || 400}
      padding={[0, 0, 10, 0]}
    >
      <Legend visible={true} />
      <Facet
        fields={['category']}
        type="rect"
        showTitle={false}
        eachView={(view, facet) => {
          const data = facet.data;
          view.coordinate({
            type: 'theta',
            cfg: {
              radius: 0.8,
              innerRadius: 0.7,
            },
          });
          view
            .interval()
            .adjust('stack')
            .position('value')
            .color('type', [
              '#249eff',
              '#846BCE',
              '#21CCFF',
              ' #86DF6C',
              '#0E42D2',
            ])
            .label('value', {
              content: (content) => {
                return `${(content.value * 100).toFixed(2)} %`;
              },
            }),
            view.annotation().text({
              position: ['50%', '46%'],
              content: data[0].category,
              style: {
                fontSize: 14,
                fontWeight: 500,
                textAlign: 'center',
              },
              offsetY: 10,
            });
          view.interaction('element-single-selected');
        }}
      />
    </Chart>
  );
}

export default FactMultiPie;
