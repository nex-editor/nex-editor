import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "nex-editor",
  description: "Headless editor core with WASM and platform shells",
  themeConfig: {
    nav: [
      { text: "Home", link: "/" },
      { text: "Protocol", link: "/protocol-v1" },
      { text: "Docs", link: "/state" },
      { text: "About", link: "/about" },
    ],

    sidebar: [
      {
        text: "Docs",
        items: [
          { text: "Cross-Platform Protocol V1", link: "/protocol-v1" },
          { text: "State", link: "/state" },
        ],
      },
    ],

    socialLinks: [
      { icon: "github", link: "https://github.com/vuejs/vitepress" },
    ],
  },
});
