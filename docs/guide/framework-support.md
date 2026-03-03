# Framework Support

NemCSS scans your content files to extract the utility class names your project actually uses. The extractor understands the class notation of all major frameworks. No additional configuration needed.

## HTML

Standard `class` attributes:

```html
<div class="text-primary p-md">...</div>
```

## React

Static and dynamic class notations:

```jsx
// className attribute
<div className="text-primary p-md" />

// conditional expression
<div className={isActive ? 'text-primary' : 'text-secondary'} />

// clsx / classnames / cn
<div className={clsx('text-primary', { 'p-md': hasPadding })} />
<div className={cn(['text-primary', 'p-md'])} />

// cva (Class Variance Authority)
const button = cva('text-primary', {
  variants: {
    size: { sm: 'p-sm', lg: 'p-lg' }
  }
})
```

## Vue

Static and bound class notations:

```vue
<!-- static -->
<div class="text-primary p-md" />

<!-- bound: string -->
<div :class="'text-primary'" />

<!-- bound: array -->
<div :class="['text-primary', 'p-md']" />

<!-- bound: object -->
<div :class="{ 'text-primary': isActive, 'p-md': true }" />

<!-- clsx / cn -->
<div :class="clsx('text-primary', { 'p-md': hasPadding })" />
```

## Svelte

Static, dynamic, and directive notations:

```svelte
<!-- static -->
<div class="text-primary p-md" />

<!-- expression -->
<div class={isActive ? 'text-primary' : 'text-secondary'} />

<!-- class directive -->
<div class:text-primary={isActive} />
<div class:p-md={hasPadding} />

<!-- clsx / cn -->
<div class={clsx('text-primary', { 'p-md': hasPadding })} />
```

## Astro

```astro
<!-- static -->
<div class="text-primary p-md" />

<!-- class:list with a string -->
<div class:list="text-primary p-md" />

<!-- class:list with an array -->
<div class:list={['text-primary', 'p-md']} />

<!-- class:list with an object -->
<div class:list={{ 'text-primary': isActive, 'p-md': true }} />
```

## Solid.js

```jsx
// className
<div className="text-primary p-md" />

// classList
<div classList={{ 'text-primary': isActive(), 'p-md': true }} />
```

## Dynamically constructed class names

NemCSS extracts class names by statically scanning your content files. It does not execute your code, so **class names that are constructed at runtime will not be detected** and the corresponding CSS will not be generated.

Avoid patterns like these:

```js
// string interpolation
`text-${color}`
`p-${size}`

// computed property names
const cls = 'text-' + variant
```

Instead, write out the full class names explicitly so the extractor can find them:

```js
// use a lookup object
const colorClass = { primary: 'text-primary', secondary: 'text-secondary' }
const sizeClass = { sm: 'p-sm', md: 'p-md', lg: 'p-lg' }

// or use conditional expressions with full names
clsx({ 'text-primary': isPrimary, 'text-secondary': isSecondary })
```

This is a fundamental constraint of static analysis: the scanner reads your source files as text and never runs your code.

## Utility libraries

The extractor recognizes `clsx`, `classnames`, `cn`, and `cva` calls in any framework, including arrays, objects, and nested combinations:

```js
clsx('text-primary', 'p-md')
clsx(['text-primary', { 'p-md': true }])
classnames({ 'text-primary': true, 'p-md': hasPadding })
cn('text-primary', isActive && 'bg-primary')
```
