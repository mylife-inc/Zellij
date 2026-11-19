import '@shebka/zellij/styles.css';
import './legacy-anchors.css';
import { createZellijRootLayout } from '@shebka/zellij';

export const metadata = {
  icons: { icon: '/_zellij/assets/logo.png' },
};

export default createZellijRootLayout();
