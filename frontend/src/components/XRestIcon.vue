<template>
  <svg
    width="200"
    height="150"
    viewBox="0 0 200 150"
    xmlns="http://www.w3.org/2000/svg"
    class="fill-current"
  >
    <defs>
      <clipPath id="cut-left-side">
        <rect x="0" y="0" width="55" height="150" />
      </clipPath>
    </defs>

    <text
      x="20"
      y="110"
      font-family="'Inter Variable', sans-serif"
      font-weight="bold"
      font-size="100"
      fill="currentColor"
      clip-path="url(#cut-left-side)"
    >
      X
    </text>

    <text
      x="45"
      y="110"
      font-family="'Inter Variable', sans-serif"
      font-weight="bold"
      font-size="100"
      fill="currentColor"
    >
      R
    </text>
  </svg>
</template>

<style scoped>
.xrest-icon {
  display: inline-block;
  vertical-align: middle;
}
</style>
