// https://vitepress.dev/guide/custom-theme
import type { Theme } from 'vitepress'
import { MermaidTheme } from '@unify-js/vitepress-mermaid'
import '@unify-js/vitepress-mermaid/style.css'
import './style.css'

export default {
  extends: MermaidTheme,
} satisfies Theme
