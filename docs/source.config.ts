import {
  transformerNotationErrorLevel,
  transformerNotationHighlight,
} from '@shikijs/transformers';
import { remarkMdxMermaid } from 'fumadocs-core/mdx-plugins';
import { defineConfig } from 'fumadocs-mdx/config';

export default defineConfig({
  mdxOptions: {
    rehypeCodeOptions: {
      themes: {
        light: 'gruvbox-light-hard',
        dark: 'gruvbox-dark-hard',
      },
      transformers: [
        transformerNotationHighlight(),
        transformerNotationErrorLevel({
          classMap: {
            error: 'line-error',
            warning: 'line-warning',
          },
        }),
      ],
    },
    remarkPlugins: [remarkMdxMermaid]
  },
});
