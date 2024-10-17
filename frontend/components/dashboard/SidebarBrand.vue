<template>
    <NuxtLink id="sidebar-brand" :to="localePath('dashboard')">
      <img src="@/assets/navbar-duck.png" alt="Duty Duck logo" />
      <span>Duty Duck</span>
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
    background-color: white;
    box-shadow: rgba(154, 161, 171, 0.15) 0px 0px 35px 0px;
    position: relative;
    overflow: hidden;
    color: $primary;
    display: flex;
    align-items: center;
    font-family: $headings-font-family;
    padding-left: $spacer;

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