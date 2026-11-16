// Same authored print overrides are exercised natively and in Next images.
export const printStyles = {
  technical: { accent: '#493B72', 'body-font': 'serif', 'title-font': 'sans', 'chapter-style': 'study', 'line-spacing': 'relaxed', margins: { left: 22, right: 20, top: 22, bottom: 24 } },
  study: { accent: '#1D5960', 'body-font': 'sans', 'title-font': 'serif', 'chapter-style': 'classic', 'line-spacing': 'spacious', margins: { left: 20, right: 18, top: 20, bottom: 22 } },
};
export function withPrintStyle(config, style) {
  if (!/^pdf:\s*$/m.test(config)) throw new Error('Fixture must contain a PDF settings block');
  return config.replace(/^pdf:\s*$/m, 'pdf:\n  style: ' + JSON.stringify(style));
}
