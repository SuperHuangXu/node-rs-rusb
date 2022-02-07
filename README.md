## @bubblex/rusb
> @bubblex/rusb allows you to listen for insert/remove events of USB devices on your system. [rusb](https://github.com/a1ien/rusb) binding to Node.js.

## Without node-gyp

`@bubblex/rusb` was prebuilt into binary already, so you don't need fighting with `node-gyp` and c++ toolchain.

## Install

```bash
npm install @bubblex/rusb
# or
yarn add @bubblex/rusb
```

## Usage
```javascript
const usbDetect = require('@bubblex/rusb')

async function main() {
  usbDetect.startMonitoring()

  usbDetect.on('add', function (device) {
    console.log('add', device)
  })
  usbDetect.on('add:vid', function (device) {
    console.log('add', device)
  })
  usbDetect.on('add:vid:pid', function (device) {
    console.log('add', device)
  })
  usbDetect.on('remove', function (device) {
    console.log('remove', device)
  })
  usbDetect.on('remove:vid', function (device) {
    console.log('remove', device)
  })
  usbDetect.on('remove:vid:pid', function (device) {
    console.log('remove', device)
  })
  usbDetect.on('change', function (device) {
    console.log('change', device)
  })
  usbDetect.on('change:vid', function (device) {
    console.log('change', device)
  })
  usbDetect.on('change:vid:pid', function (device) {
    console.log('change', device)
  })

  setTimeout(() => {
    // Allow the process to exit
    usbDetect.stopMonitoring()
  }, 6000)

  let devices = await usbDetect.find()

  devices = await usbDetect.find({
    vid: 1507,
  })

  devices = await usbDetect.find({
    pid: 1558,
  })

  devices = await usbDetect.find({
    pid: 1558,
    vid: 1507,
  })
  
  console.log(devices)
}

main()

```

Reference [node-usb-detection](https://github.com/MadLittleMods/node-usb-detection)