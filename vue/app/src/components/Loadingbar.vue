<template>
  <div class="h-0.5 w-full overflow-hidden bg-gray-200">
    <div
      class="h-full w-1/2 animate-[loading_1.5s_ease-in-out_infinite] bg-linear-to-r from-blue-500 to-blue-400"
    ></div>
  </div>
</template>

<style>
@keyframes loading {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(200%);
  }
}
</style>
