<script setup lang="ts">
const { data: buildInfo, error: serverError } = await useServer("/");
const { public: { packageVersion, commitRef } } = useRuntimeConfig();
</script>
<template>
   <BButton variant="link" icon v-b-toggle.software-info>
      <Icon name="icon-park-twotone:branch-two" />
      Show software info
   </BButton>
   <BCollapse id="software-info">
      <h3>About this software</h3>
      <div class="row">
         <div class="col-md-6">
            <h4>Client facts</h4>
            <ul>
               <li>
                  <b>Version</b>: {{ packageVersion }}
               </li>
               <li>
                  <b>Commit</b>: {{ commitRef }}
               </li>
            </ul>
         </div>
         <div class="col-md-6">
            <h4>Server facts</h4>
            <ul>
               <li>
                  <b>Git version</b>: {{ buildInfo['gitVersion'] }}
               </li>
               <li>
                  <b>Commit</b>: {{ buildInfo['gitCommitHash'] }}
               </li>
               <li>
                  <b>BuildTime</b>: {{ buildInfo['buildTime'] }}
               </li>
            </ul>
         </div>
      </div>
   </BCollapse>
</template>
