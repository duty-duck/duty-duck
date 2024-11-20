<template>
    <NuxtLink id="sidebar-brand" :to="localePath('dashboard')">
      <img src="@/assets/navbar-duck.png" alt="Duty Duck logo" />
      <span class="d-none d-xxl-block">Duty Duck</span>
    </NuxtLink>
  </template>
  
  <script setup lang="ts">
    const localePath = useLocalePath();
  </script>
  
  <style lang="scss" scoped>
  @import "~/assets/main.scss";
  
  #sidebar-brand {
    text-decoration: none;
    height: $navbar-height;
    position: relative;
    overflow: hidden;
    color: $primary;
    display: flex;
    align-items: center;
    font-family: $headings-font-family;
    border-bottom: 1px solid rgb(234 236 241);

    &:hover {
      img {
        bottom: -5px;
      }
    }
  
    span {
      position: relative;
      left: 10px;
    }
  
    img {
      transition: bottom 100ms ease-in-out;
      height: $navbar-height - 5px;
      position: relative;
      bottom: -12px;
    }
  }
  </style>