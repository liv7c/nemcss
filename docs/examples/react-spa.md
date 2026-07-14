# React SPA (Vite)

This guide walks through setting up NemCSS in a React SPA project using Vite.

## Step 1: Create a React + Vite project

::: code-group
```sh [npm]
npm create vite@latest my-app -- --template react
cd my-app
```
```sh [pnpm]
pnpm create vite@latest my-app -- --template react
cd my-app
```
```sh [yarn]
yarn create vite my-app --template react
cd my-app
```
:::

## Step 2: Install `@nemcss/vite`

::: code-group
```sh [npm]
npm install -D @nemcss/vite
```
```sh [pnpm]
pnpm add -D @nemcss/vite
```
```sh [yarn]
yarn add -D @nemcss/vite
```
```sh [bun]
bun add -D @nemcss/vite
```
:::

## Step 3: Add the plugin to your Vite config

```js
// vite.config.js
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { nemcss } from '@nemcss/vite'

export default defineConfig({
  plugins: [react(), nemcss()],
})
```

## Step 4: Initialize nemcss

::: code-group
```sh [npx]
npx nemcss init
```
```sh [pnpm dlx]
pnpm dlx nemcss init
```
```sh [yarn dlx]
yarn dlx nemcss init
```
:::

This creates a minimal `nemcss.config.json` and an empty `design-tokens/` folder.

## Step 5: Add design tokens

Create a color and a spacing token file with `new-token-file`. It registers each one in your config as it creates it:

```sh
npx nemcss new-token-file colors --prefix color --values "hsl(0, 0%, 100%),hsl(0, 0%, 0%)" --names "white,black"
npx nemcss new-token-file spacings --prefix spacing --unit rem --values "0.5,1,1.5" --names "sm,md,lg"
```

Add a `padding` utility to the spacings entry, and a `text` semantic group so `text-default` and `text-muted` are available, by editing `nemcss.config.json`:

```json
{
  "theme": {
    "spacings": {
      "prefix": "spacing",
      "source": "design-tokens/spacings.json",
      "utilities": [{ "prefix": "p", "property": "padding" }]
    }
  },
  "semantic": {
    "text": {
      "property": "color",
      "tokens": {
        "default": "{colors.black}",
        "muted": "{colors.white}"
      }
    }
  }
}
```

## Step 6: Configure content paths

Edit `nemcss.config.json` to point at your React source files:

```json
{
  "content": ["src/**/*.tsx", "src/**/*.jsx", "src/**/*.ts"],
  "tokensDir": "design-tokens"
}
```

## Step 7: Add the directives to your CSS

Open `src/index.css` and add both directives at the top:

```css
@nemcss base;
@nemcss utilities;
```

Make sure `src/index.css` is imported in `src/main.jsx` (it is by default in the Vite React template):

```jsx
// src/main.jsx
import './index.css'
```

## Step 8: Use the generated classes

```jsx
// src/App.jsx
export default function App() {
  return (
    <main className="p-lg">
      <h1 className="text-default">Hello NemCSS</h1>
      <p className="text-muted">Styled with design tokens.</p>
    </main>
  )
}
```

## Step 9: Start the dev server

::: code-group
```sh [npm]
npm run dev
```
```sh [pnpm]
pnpm dev
```
```sh [yarn]
yarn dev
```
:::

HMR is enabled by default. Your CSS updates without a full page reload when you change a token file, your config, or any content file.
