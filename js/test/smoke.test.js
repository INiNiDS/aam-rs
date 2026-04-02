const test = require('node:test')
const assert = require('node:assert/strict')

// Подтягиваем то, что реально экспортировано
const {AAM, parse, version} = require('..')

test('parse and inspect values', () => {
    const cfg = parse(`
host = localhost
port = 8080
point = { x = 10, y = 20 }
`)

    assert.equal(typeof version(), 'string')

    assert.equal(cfg.get('host'), 'localhost')

    assert.deepEqual(cfg.reverseSearch('8080'), ['port'])

    assert.deepEqual(cfg.find('host'), {host: 'localhost'})
    assert.deepEqual(cfg.find('localhost'), {host: 'localhost'})
    assert.equal(cfg.get('point'), '{ x = 10, y = 20 }')
})

test('mutable instance lifecycle', () => {
    const cfg = new AAM()

    assert.equal(cfg.isClosed(), false)

    cfg.close()

    assert.equal(cfg.isClosed(), true)
    assert.equal(cfg.get('any'), null)
})

test('deepSearch matches by key pattern', () => {
    const cfg = parse(`
root_path = srv_app
active_path = root_path
mode = active
`)

    assert.deepEqual(cfg.deepSearch('path'), {
        root_path: 'srv_app',
        active_path: 'root_path',
    })
})

test('reverseSearch supports lookup', () => {
    const cfg = parse('username = admin')

    assert.equal(cfg.get('username'), 'admin')

    assert.deepEqual(cfg.reverseSearch('admin'), ['username'])
})

test('parse throws on invalid assignment syntax', () => {
    assert.throws(() => parse('invalid_line_without_equals'))
})