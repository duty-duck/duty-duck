// @ts-check

import * as sass from "sass";
import { defineConfig } from "@rspack/cli";
import { RspackManifestPlugin } from "rspack-manifest-plugin";

const config = defineConfig({
  entry: {
    index: ["./assets/src/index.js", "./assets/src/index.scss"],
  },
  output: {
    path: "./assets/out",
    filename: "[name].js",
    clean: true,
  },
  module: {
    rules: [
      {
        test: /\.(sass|scss)$/,
        use: [
          {
            loader: 'postcss-loader',
            options: {
              postcssOptions: {
                plugins: {
                  autoprefixer: {},
                },
              },
            },
          },
          {
            loader: "sass-loader",
            options: {
              implementation: sass
            },
          },
        ],
        type: "css/auto",
      },
      
    ],
  },
  plugins: [
    new RspackManifestPlugin({
      publicPath: "/assets",
    }),
  ],
});

export default config;
