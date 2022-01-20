const { loadBinding } = require('@node-rs/helper')

/**
 * __dirname means load native addon from current dir
 * 'rusb' is the name of native addon
 * the second arguments was decided by `napi.name` field in `package.json`
 * the third arguments was decided by `name` field in `package.json`
 * `loadBinding` helper will load `rusb.[PLATFORM].node` from `__dirname` first
 * If failed to load addon, it will fallback to load from `rusb-[PLATFORM]`
 */
// module.exports = loadBinding(__dirname, 'rusb', 'rusb')
const lib = loadBinding(__dirname, 'rusb', 'rusb')
const mitt = require('mitt')

const emitter = mitt()

emitter.find = lib.find
emitter.startMonitoring = function () {
  lib.startMonitoring(function (err, msg) {
    if (err) {
      throw err
    }
    const device = msg.device
    if (msg.action === 'arrived') {
      emitter.emit('add:' + device.vendorId + ':' + device.productId, device)
      emitter.emit('add:' + device.vendorId, device)
      emitter.emit('add', device)

      emitter.emit('change:' + device.vendorId + ':' + device.productId, device)
      emitter.emit('change:' + device.vendorId, device)
      emitter.emit('change', device)
    } else if (msg.action === 'left') {
      emitter.emit('remove:' + device.vendorId + ':' + device.productId, device)
      emitter.emit('remove:' + device.vendorId, device)
      emitter.emit('remove', device)

      emitter.emit('change:' + device.vendorId + ':' + device.productId, device)
      emitter.emit('change:' + device.vendorId, device)
      emitter.emit('change', device)
    }
  })
}
emitter.stopMonitoring = lib.stopMonitoring

module.exports = emitter
