import { Grid, Space } from '@arco-design/web-react'
import React from 'react'
import Announcement from './announcement'
import Carousel from './carousel'
import ContentPercentage from './content-percentage'
import Docs from './docs'
import './mock'
import Overview from './overview'
import PopularContents from './popular-contents'
import Shortcuts from './shortcuts'
import styles from './style/index.module.less'

const { Row, Col } = Grid

const gutter = 16

function Workplace() {
  return (
    <Space size={16} align="start">
      <Space size={16} direction="vertical">
        <Overview />
        <Row gutter={gutter}>
          <Col span={12}>
            <PopularContents />
          </Col>
          <Col span={12}>
            <ContentPercentage />
          </Col>
        </Row>
      </Space>
      <Space className={styles.right} size={16} direction="vertical">
        <Shortcuts />
        <Carousel />
        <Announcement />
        <Docs />
      </Space>
    </Space>
  )
}

export default Workplace
