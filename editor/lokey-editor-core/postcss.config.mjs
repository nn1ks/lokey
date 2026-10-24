// Used by tsup's postcss plugin to compile the `@apply` / `@reference` directives in
// `*.module.css` files at library build time, so consumers get plain CSS.
export default {
  plugins: {
    '@tailwindcss/postcss': {},
  },
}
