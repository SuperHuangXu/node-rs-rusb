const { find } = require('.')

async function main() {
  const res = await find({
    pid: 1558,
    vid: 1507
  })
  console.log(res)
}

main()