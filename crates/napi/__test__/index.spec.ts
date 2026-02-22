import test from 'ava'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { extractClasses, generateCss } from '../index.js'

const __dirname = dirname(fileURLToPath(import.meta.url))
const FIXTURE_CONFIG_PATH = join(__dirname, 'fixtures', 'nemcss.config.json')

// --- extractClasses ---
test('extractClasses returns class names from a class attribute', (t) => {
  const html = `<div class="text-primary bg-secondary highlight"></div>`

  const classes = extractClasses(html)

  t.true(Array.isArray(classes))
  t.true(classes.includes('text-primary'))
  t.true(classes.includes('bg-secondary'))
  t.true(classes.includes('highlight'))
})

test('extractClasses handles className and class:list attributes', (t) => {
  const html = `
   <div className="text-secondary"></div>
   <span class:list={['text-primary', 'bg-secondary', 'highlight']}></span>
  `

  const classes = extractClasses(html)

  t.true(Array.isArray(classes))
  t.true(classes.includes('text-secondary'))
  t.true(classes.includes('text-primary'))
  t.true(classes.includes('bg-secondary'))
  t.true(classes.includes('highlight'))
})

test('extractClasses returns an empty array for content with no classes', (t) => {
  const html = `<div></div>`

  const classes = extractClasses(html)

  t.deepEqual(classes, [])
})

// --- generateCss ---
test('generateCss returns CSS from all tokens when no filter is applied', (t) => {
  const css = generateCss(FIXTURE_CONFIG_PATH, null)

  t.is(typeof css, 'string')

  t.true(css.includes('.text-primary'))
  t.true(css.includes('.text-secondary'))
  t.true(css.includes('.bg-primary'))
  t.true(css.includes('.bg-secondary'))
  t.true(css.includes('.p-sm'))
  t.true(css.includes('.p-md'))
})

test('generateCss only outputs rules for the requested classes', (t) => {
  const css = generateCss(FIXTURE_CONFIG_PATH, ['text-primary'])

  t.true(css.includes('.text-primary'))
  t.false(css.includes('.text-secondary'))
  t.false(css.includes('.bg-primary'))
  t.false(css.includes('.bg-secondary'))
  t.false(css.includes('.p-sm'))
  t.false(css.includes('.p-md'))
})
