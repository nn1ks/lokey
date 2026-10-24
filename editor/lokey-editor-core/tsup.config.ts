import { defineConfig } from 'tsup'
import * as preset from 'tsup-preset-solid'

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

  return preset.generateTsupOptions(parsed_options)
})
