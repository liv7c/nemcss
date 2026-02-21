import { Bench } from 'tinybench'
import { mkdirSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { extractClasses, generateCss } from '../index.js'

// --- fixture generation ---

type FixtureOptions = {
  numCategories: number
  numTokens: number
  numUtilities: number
}

type Fixture = {
  configPath: string
  classNames: string[]
  dir: string
}

function generateFixture({ numCategories, numTokens, numUtilities }: FixtureOptions): Fixture {
  const dir = join(tmpdir(), `nemcss-bench-${numCategories}-${numTokens}-${Date.now()}`)
  mkdirSync(join(dir, 'design-tokens'), { recursive: true })

  const theme: Record<string, unknown> = {}
  const classNames: string[] = []

  for (let i = 0; i < numCategories; i++) {
    const prefix = `prefix-${i}`
    const tokenFile = `design-tokens/category-${i}.json`

    const items = Array.from({ length: numTokens }, (_, j) => ({
      name: `${prefix}-${j}`,
      value: `value-${j}`,
    }))

    writeFileSync(join(dir, tokenFile), JSON.stringify({ title: `category-${i}`, items }))

    const utilities = Array.from({ length: numUtilities }, (_, j) => {
      const utilPrefix = `${prefix}-util-${j}`
      for (const item of items) classNames.push(`${utilPrefix}-${item.name}`)
      return { prefix: utilPrefix, property: 'color' }
    })

    theme[`category-${i}`] = { prefix, source: tokenFile, utilities }
  }

  const configPath = join(dir, 'nemcss.config.json')
  writeFileSync(configPath, JSON.stringify({ content: ['src/**/*.html'], theme }))

  return { configPath, classNames, dir }
}

function generateHtml(classNames: string[], numElements: number): string {
  return Array.from({ length: numElements }, (_, i) => {
    const cls1 = classNames[i % classNames.length]
    const cls2 = classNames[(i + 1) % classNames.length]
    return `<div class="${cls1} ${cls2}">Item ${i}</div>`
  }).join('\n')
}

// --- scenarios, mirroring the Rust benchmark ---

const scenarios = [
  { label: 'small', numCategories: 2, numTokens: 10, numUtilities: 2 },
  { label: 'realistic', numCategories: 5, numTokens: 15, numUtilities: 5 },
  { label: 'large', numCategories: 10, numTokens: 20, numUtilities: 10 },
]

// Generate all fixtures before the bench loop so file I/O doesn't skew timings
const fixtures = scenarios.map((s) => ({ ...s, fixture: generateFixture(s) }))

const b = new Bench({ time: 2000 })

for (const { label, fixture } of fixtures) {
  const { configPath, classNames } = fixture
  const html = generateHtml(classNames, classNames.length)

  b.add(`extractClasses — ${label} (${classNames.length} classes)`, () => {
    extractClasses(html)
  })

  b.add(`generateCss — ${label} (${classNames.length} classes, no filter)`, () => {
    generateCss(configPath, null)
  })

  b.add(`generateCss — ${label} (${classNames.length} classes, filtered to half)`, () => {
    generateCss(configPath, classNames.slice(0, Math.floor(classNames.length / 2)))
  })
}

await b.run()

console.table(b.table())

for (const { fixture } of fixtures) {
  rmSync(fixture.dir, { recursive: true, force: true })
}
