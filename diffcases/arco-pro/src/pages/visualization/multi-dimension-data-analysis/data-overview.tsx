// 数据总览
import MultiAreaLine from '@/components/Chart/multi-area-line'
import useLocale from '@/utils/useLocale'
import { Card, Grid, Skeleton, Statistic, Typography } from '@arco-design/web-react'
import { IconEdit, IconHeart, IconThumbUp, IconUser } from '@arco-design/web-react/icon'
import axios from 'axios'
import React, { useEffect, useMemo, useState } from 'react'
import locale from './locale'
import styles from './style/data-overview.module.less'

const { Title } = Typography
export default () => {
  const t = useLocale(locale)
  const [overview, setOverview] = useState([])
  const [lineData, setLineData] = useState([])
  const [loading, setLoading] = useState(false)

  const getData = async () => {
    setLoading(true)
    const { data } = await axios
      .get('/api/multi-dimension/overview')
      .finally(() => setLoading(false))

    const { overviewData, chartData } = data
    setLineData(chartData)
    setOverview(overviewData)
  }

  useEffect(() => {
    getData()
  }, [])

  const formattedData = useMemo(() => {
    return [
      {
        title: t['multiDAnalysis.dataOverview.contentProduction'],
        icon: <IconEdit />,
        value: overview[0],
        background: 'rgb(var(--orange-2))',
        color: 'rgb(var(--orange-6))',
      },
      {
        title: t['multiDAnalysis.dataOverview.contentClicks'],
        icon: <IconThumbUp />,
        value: overview[1],
        background: 'rgb(var(--cyan-2))',
        color: 'rgb(var(--cyan-6))',
      },
      {
        title: t['multiDAnalysis.dataOverview.contextExposure'],
        value: overview[2],
        icon: <IconHeart />,
        background: 'rgb(var(--arcoblue-1))',
        color: 'rgb(var(--arcoblue-6))',
      },
      {
        title: t['multiDAnalysis.dataOverview.activeUsers'],
        value: overview[3],
        icon: <IconUser />,
        background: 'rgb(var(--purple-1))',
        color: 'rgb(var(--purple-6))',
      },
    ]
  }, [t, overview])

  return (
    <Grid.Row justify="space-between">
      {formattedData.map((item, index) => (
        <Grid.Col span={24 / formattedData.length} key={`${index}`}>
          <Card className={styles.card} title={null}>
            <Title heading={6}>{item.title}</Title>
            <div className={styles.content}>
              <div
                style={{ backgroundColor: item.background, color: item.color }}
                className={styles['content-icon']}
              >
                {item.icon}
              </div>
              {loading
                ? (
                  <Skeleton
                    animation
                    text={{ rows: 1, className: styles['skeleton'] }}
                    style={{ width: '120px' }}
                  />
                )
                : <Statistic value={item.value} groupSeparator />}
            </div>
          </Card>
        </Grid.Col>
      ))}
      <Grid.Col span={24}>
        <MultiAreaLine data={lineData} loading={loading} />
      </Grid.Col>
    </Grid.Row>
  )
}
