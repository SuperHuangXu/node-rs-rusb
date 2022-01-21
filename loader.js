const { existsSync } = require('fs')
const { join } = require('path')

const { platform, arch } = process

const NAPI_NAME = 'rusb'
const NPM_PACKAGE_NAME = '@bubblex/rusb'

let nativeBinding = null
let localFileExisted = false
let loadError = null

switch (platform) {
  case 'win32':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(join(__dirname, `${NAPI_NAME}.win32-x64-msvc.node`))
        try {
          if (localFileExisted) {
            nativeBinding = require(`./${NAPI_NAME}.win32-x64-msvc.node`)
          } else {
            nativeBinding = require(`${NPM_PACKAGE_NAME}-win32-x64-msvc`)
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'ia32':
        localFileExisted = existsSync(join(__dirname, `${NAPI_NAME}.win32-ia32-msvc.node`))
        try {
          if (localFileExisted) {
            nativeBinding = require(`./${NAPI_NAME}.win32-ia32-msvc.node`)
          } else {
            nativeBinding = require(`${NPM_PACKAGE_NAME}-win32-ia32-msvc`)
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(join(__dirname, `${NAPI_NAME}.win32-arm64-msvc.node`))
        try {
          if (localFileExisted) {
            nativeBinding = require(`./${NAPI_NAME}.win32-arm64-msvc.node`)
          } else {
            nativeBinding = require(`${NPM_PACKAGE_NAME}-win32-arm64-msvc`)
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on Windows: ${arch}`)
    }
    break
  case 'darwin':
    switch (arch) {
      case 'x64':
        localFileExisted = existsSync(join(__dirname, `${NAPI_NAME}.darwin-x64.node`))
        try {
          if (localFileExisted) {
            nativeBinding = require(`./${NAPI_NAME}.darwin-x64.node`)
          } else {
            nativeBinding = require(`${NPM_PACKAGE_NAME}-darwin-x64`)
          }
        } catch (e) {
          loadError = e
        }
        break
      case 'arm64':
        localFileExisted = existsSync(join(__dirname, `${NAPI_NAME}.darwin-arm64.node`))
        try {
          if (localFileExisted) {
            nativeBinding = require(`./${NAPI_NAME}.darwin-arm64.node`)
          } else {
            nativeBinding = require(`${NPM_PACKAGE_NAME}-darwin-arm64`)
          }
        } catch (e) {
          loadError = e
        }
        break
      default:
        throw new Error(`Unsupported architecture on macOS: ${arch}`)
    }
    break
  default:
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`)
}

if (!nativeBinding) {
  if (loadError) {
    throw loadError
  }
  throw new Error(`Failed to load native binding`)
}

module.exports = nativeBinding
