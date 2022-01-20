const usbDetect = require('.')

async function main() {
  setTimeout(() => {
    usbDetect.stopMonitoring()
  }, 5000)

  usbDetect.on('add', function (device) {
    console.log('add', device)
  })
  usbDetect.on('add:10473', function (device) {
    console.log('add 10473', device)
  })
  usbDetect.on('add:10473:394', function (device) {
    console.log('add 10473:394', device)
  })
  usbDetect.on('remove', function (device) {
    console.log('remove', device)
  })
  usbDetect.on('remove:10473', function (device) {
    console.log('remove 10473', device)
  })
  usbDetect.on('remove:10473:394', function (device) {
    console.log('remove 10473:394', device)
  })
  usbDetect.on('change', function (device) {
    console.log('change', device)
  })

  usbDetect.startMonitoring()

  const res = await usbDetect.find({
    pid: 1558,
    vid: 1507,
  })

  console.log(res)
}

main()
