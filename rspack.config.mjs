// @ts-check

import * as sass from "sass-embedded";
import { defineConfig } from "@rspack/cli";
import rspack from "@rspack/core";
import { RspackManifestPlugin } from "rspack-manifest-plugin";

const config = defineConfig({
  entry: {
    index: ["./assets/index.js", "./assets/index.scss"],
  },
  output: {
    path: "./target/assets",
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
    new rspack.CopyRspackPlugin({
      patterns: [
        { 
          from: "assets/static",
          to: "static"
        }
      ]
    })
  ],
});

export default config;
