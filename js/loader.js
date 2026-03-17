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

// High complexity
function resolveTarget() {
    const {platform, arch} = process

    if (platform === 'linux' && arch === 'x64') {
        return isMusl()
            ? ['linux-x64-musl', 'aam-rs-linux-x64-musl']
            : ['linux-x64-gnu', 'aam-rs-linux-x64-gnu']
    }

    if (platform === 'darwin' && arch === 'x64') {
        return ['darwin-x64', 'aam-rs-darwin-x64']
    }

    if (platform === 'darwin' && arch === 'arm64') {
        return ['darwin-arm64', 'aam-rs-darwin-arm64']
    }

    if (platform === 'win32' && arch === 'x64') {
        return ['win32-x64-msvc', 'aam-rs-win32-x64-msvc']
    }

    throw new Error(`Unsupported platform for aam-rs: ${platform} ${arch}`)
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

