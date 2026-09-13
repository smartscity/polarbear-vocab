import type { Preview } from "@storybook/react-vite";

import { I18nProvider } from "../src/lib/i18n";
import "../src/styles.css";

const preview: Preview = {
  decorators: [
    (Story) => <I18nProvider><Story /></I18nProvider>,
  ],
  parameters: {
    controls: { expanded: true },
    layout: "fullscreen",
  },
};

export default preview;
