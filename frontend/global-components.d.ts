// src/global-components.d.ts
import {RouterLink} from 'vue-router'

declare module 'vue' {
  export interface GlobalComponents {
    RouterLink: typeof RouterLink,
  }
}