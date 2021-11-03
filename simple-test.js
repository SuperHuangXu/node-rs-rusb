const { getDeviceList } = require('./index')

async function main() {
  const res = await getDeviceList()
  console.log(res)
}

main()