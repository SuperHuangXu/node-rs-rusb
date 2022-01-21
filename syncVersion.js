const fs = require('fs')
const path = require('path')
const pkgVersion = require('./package.json').version

;(async function () {
  const npmFolder = fs.readdirSync('npm')
  const packages = npmFolder.map((folder) => path.resolve('./npm', folder, 'package.json'))
  for (const pkg of packages) {
    const jsonFile = fs.readFileSync(pkg)
    const json = JSON.parse(jsonFile)
    json.version = pkgVersion
    fs.writeFileSync(pkg, JSON.stringify(json, null, 2))
  }
})()
