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
test('generateCss returns an object that contains the baseCSS with the :root custom properties block', (t) => {
  const { baseCss } = generateCss(FIXTURE_CONFIG_PATH, null)

  t.is(typeof baseCss, 'string')
  t.true(baseCss.includes(':root {'))
  t.true(baseCss.includes('--color-primary: #fccd26;'))
  t.true(baseCss.includes('--color-secondary: #171406;'))
  t.true(baseCss.includes('--spacing-sm: 0.5rem;'))
  t.true(baseCss.includes('--spacing-md: 1rem;'))
})

test('generateCss returns all utilities when no filter is applied', (t) => {
  const { utilitiesCss } = generateCss(FIXTURE_CONFIG_PATH, null)

  t.is(typeof utilitiesCss, 'string')

  t.true(utilitiesCss.includes('.text-primary'))
  t.true(utilitiesCss.includes('.text-secondary'))
  t.true(utilitiesCss.includes('.bg-primary'))
  t.true(utilitiesCss.includes('.bg-secondary'))
  t.true(utilitiesCss.includes('.p-sm'))
  t.true(utilitiesCss.includes('.p-md'))
})

test('generateCss only outputs rules for the requested classes', (t) => {
  const { utilitiesCss } = generateCss(FIXTURE_CONFIG_PATH, ['text-primary'])

  t.true(utilitiesCss.includes('.text-primary'))
  t.false(utilitiesCss.includes('.text-secondary'))
  t.false(utilitiesCss.includes('.bg-primary'))
  t.false(utilitiesCss.includes('.bg-secondary'))
  t.false(utilitiesCss.includes('.p-sm'))
  t.false(utilitiesCss.includes('.p-md'))
})
