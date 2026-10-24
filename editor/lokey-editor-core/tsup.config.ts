import { readdir, readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { defineConfig } from 'tsup'
import * as preset from 'tsup-preset-solid'

/**
 * Re-adds `import './<name>.css'` to emitted JS files that have a sibling stylesheet.
 *
 * esbuild with the `local-css` loader emits `import "./index.css"`, but tsup strips that import
 * while keeping the generated class-name mapping. Without it, nothing loads the stylesheet.
 */
async function linkStylesheets(dir: string): Promise<void> {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      await linkStylesheets(full)
      continue
    }
    if (!entry.name.endsWith('.js')) continue

    const cssName = entry.name.replace(/\.js$/, '.css')
    const cssPath = path.join(dir, cssName)
    try {
      await readFile(cssPath)
    } catch {
      continue
    }

    const source = await readFile(full, 'utf8')
    const importLine = `import './${cssName}';`
    if (source.includes(importLine)) continue

    await writeFile(full, `${importLine}\n${source}`)
  }
}

const preset_options: preset.PresetOptions = {
  // array or single object
  entries: [
    // default entry (index)
    {
      // entries with '.tsx' extension will have `solid` export condition generated
      entry: 'src/index.ts',
      // will generate a separate development entry
      dev_entry: true,
    },
    {
      entry: 'src/components/index.ts',
      name: 'components',
    },
    {
      entry: 'src/components/keymap/index.ts',
      name: 'components/keymap',
    },
    {
      entry: 'src/components/led-array/index.ts',
      name: 'components/led-array',
    },
    {
      entry: 'src/components/start/index.ts',
      name: 'components/start',
    },
    {
      entry: 'src/contexts/index.ts',
      name: 'contexts',
    },
    {
      entry: 'src/protocol/index.ts',
      name: 'protocol',
    },
  ],
  // Set to `true` to remove all `console.*` calls and `debugger` statements in prod builds
  drop_console: true,
  // `src/styles/global.css` starts with `@import 'tailwindcss'`, which esbuild can't resolve (the
  // package only exposes a `style` condition). Leave it in the output so the consuming app's
  // Tailwind/Vite pipeline processes it.
  esbuild_plugins: [],
  modify_esbuild_options: options => {
    options.external = [...(options.external ?? []), 'tailwindcss']
    return options
  },
  // Set to `true` to generate a CommonJS build alongside ESM
  // cjs: true,
}

export default defineConfig(config => {
  const watching = !!config.watch

  const parsed_options = preset.parsePresetOptions(preset_options, watching)

  return preset.generateTsupOptions(parsed_options).map(option => ({
    ...option,
    // esbuild's `local-css` loader compiles `*.module.css` to scoped class names and injects the
    // `{ root: 'PageTabs_root' }` mapping into the emitted JS, so consumers don't need CSS-modules
    // support. tsup's postcss plugin takes this from the top-level option rather than
    // `esbuildOptions`, which is why it is set here.
    loader: { ...(option.loader ?? {}), '.css': 'local-css' },
    onSuccess: async () => {
      await linkStylesheets(option.outDir ?? 'dist')
      await option.onSuccess?.()
    },
  }))
})
