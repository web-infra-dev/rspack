export function a0() {
  a2('*a0*')
  return <view></view>
}

function A1() {
  return <view>{'*a1*'}</view>
}

export function a2() {
  return (
    <view>
      <A1 bar={'*a2*'} />
    </view>
  )
}

