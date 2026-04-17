const {existsSync} = require('node:fs')
const {join} = require('node:path')

function isMusl() {
    if (process.platform !== 'linux') {
        return false
    }

    if (typeof process.report?.getReport === 'function') {
        const report = process.report.getReport()
        return !report.header.glibcVersionRuntime
    }

    return true
}

function resolveTarget() {
    const {platform, arch} = process

    if (platform === 'linux' && arch === 'x64') {
        return isMusl()
            ? ['linux-x64-musl', 'aam-nodejs-linux-x64-musl']
            : ['linux-x64-gnu', 'aam-nodejs-linux-x64-gnu']
    }

    const table = {
        'darwin:x64': ['darwin-x64', 'aam-nodejs-darwin-x64'],
        'darwin:arm64': ['darwin-arm64', 'aam-nodejs-darwin-arm64'],
        'win32:x64': ['win32-x64-msvc', 'aam-nodejs-win32-x64-msvc'],
    }

    const target = table[`${platform}:${arch}`]
    if (target) {
        return target
    }

    throw new Error(`Unsupported platform for aam-nodejs: ${platform} ${arch}`)
}

const [targetSuffix, packageName] = resolveTarget()
const localCandidates = [
    join(__dirname, 'aam_rs_node.node'),
    join(__dirname, `aam_rs_node.${targetSuffix}.node`),
]

let nativeBinding = null
let loadError = null

for (const candidate of localCandidates) {
    if (!existsSync(candidate)) {
        continue
    }

    try {
        nativeBinding = require(candidate)
        break
    } catch (error) {
        loadError = error
    }
}

if (!nativeBinding) {
    try {
        nativeBinding = require(packageName)
    } catch (error) {
        loadError = error
    }
}

if (!nativeBinding) {
    throw loadError ?? new Error(`Failed to load native binding for ${packageName}`)
}

module.exports = nativeBinding

