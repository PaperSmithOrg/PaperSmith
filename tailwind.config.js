import themer from "tailwindcss-themer";
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html",
    "./src/**/*.html",
    "./tailwind.css",
  ],
  theme: {
    fontFamily: {
      inherit: ['"inherit"'],
      standard: ['"Arial"', '"sans-serif"'],
    },
    extend: {
      transitionProperty: {
        "max-height": "max-height",
      },
      height: {
        'notepad': 'calc(100vh - 125px)',
      },
    },
  },
  corePlugins: {
    preflight: false,
  },
  variants: {},
  plugins: [
    themer({
      defaultTheme: {
        extend: {
          colors: {
            primary: "#cba6f7",
            secondary: "#74c7ec",
            accent: "#94e2d5",
            text: "#cdd6f4",
            subtext: "#a6adc8",
            base: "#1e1e2e",
            mantle: "#181825",
            crust: "#11111b",
          },
        },
      },
      themes: [
        {
          name: "light",
          extend: {
            colors: {
              primary: "#8839ef",
              secondary: "#209fb5",
              accent: "#179299",
              text: "#4c4f69",
              subtext: "#6c6f85",
              base: "#eff1f5",
              mantle: "#e6e9ef",
              crust: "#dce0e8",
            },
          },
        },
        {
          name: "lightdark",
          extend: {
            colors: {
              primary: "#ca9ee6",
              secondary: "#85c1dc",
              accent: "#81c8be",
              text: "#c6d0f5",
              subtext: "#a5adce",
              base: "#303446",
              mantle: "#292c3c",
              crust: "#232634",
            },
          },
        },
        {
          name: "medium",
          extend: {
            colors: {
              primary: "#c6a0f6",
              secondary: "#7dc4e4",
              accent: "#8bd5ca",
              text: "#cad3f5",
              subtext: "#a5adcb",
              base: "#24273a",
              mantle: "#1e2030",
              crust: "#181926",
            },
          },
        },
        {
          name: "dark",
          extend: {
            colors: {
              primary: "#cba6f7",
              secondary: "#74c7ec",
              accent: "#94e2d5",
              text: "#cdd6f4",
              subtext: "#a6adc8",
              base: "#1e1e2e",
              mantle: "#181825",
              crust: "#11111b",
            },
          },
        },
        {
          name: "verydark",
          extend: {
            colors: {
              primary: "#1DD65F",
              secondary: "#1DD65F",
              accent: "#1DD65F",
              text: "#ffffff",
              subtext: "#B2B2B2",
              base: "#1e1e1e",
              mantle: "#111111",
              crust: "#000000",
            },
          },
        },
      ],
    }),
    require('@tailwindcss/forms'),
  ],
};
