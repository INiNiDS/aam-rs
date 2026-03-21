const test = require('node:test')
const assert = require('node:assert/strict')

const {AAML, parse, version} = require('..')

test('parse and inspect values', () => {
    const cfg = parse(`
host = localhost
port = 8080
paths = [assets, cache]
point = { x = 10, y = 20 }
`)

    assert.equal(typeof version(), 'string')
    assert.equal(cfg.findObj('host'), 'localhost')
    assert.equal(cfg.findKey('8080'), 'port')
    assert.deepEqual(cfg.findList('paths'), ['assets', 'cache'])
    assert.deepEqual(cfg.findObject('point'), {x: '10', y: '20'})
})

test('mutable instance lifecycle', () => {
    const cfg = new AAML()

    cfg.merge('theme = dark')
    cfg.mergeContent('font = mono')

    assert.deepEqual(cfg.keys().sort(), ['font', 'theme'])
    assert.deepEqual(cfg.toMap(), {font: 'mono', theme: 'dark'})
    assert.equal(cfg.isClosed(), false)

    cfg.close()

    assert.equal(cfg.isClosed(), true)
    assert.equal(cfg.findObj('theme'), null)
})

test('findDeep resolves chained aliases', () => {
    const cfg = parse(`
root = /srv/app
active = root
current = active
`)

    assert.equal(cfg.findDeep('current'), '/srv/app')
})

test('findObj supports reverse lookup fallback', () => {
    const cfg = parse('username = admin')

    assert.equal(cfg.findObj('username'), 'admin')
    assert.equal(cfg.findObj('admin'), 'username')
})

test('parse throws on invalid assignment syntax', () => {
    assert.throws(() => parse('invalid_line_without_equals'))
})


