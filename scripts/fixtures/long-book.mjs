// Original short chapters exercise the three-digit contents boundary.
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
export function writeLongBook(root) {
  mkdirSync(path.join(root, 'content'), { recursive: true });
  const chapters = Array.from({ length: 100 }, (_, index) => {
    const number = index + 1;
    const id = `topic-${number}`;
    writeFileSync(path.join(root, 'content', id + '.md'),
      `# Topic ${number}\n\nA clear question gives this chapter its purpose. Its example connects an observation to a conclusion.\n\nThis is the original text of topic ${number}.\n`);
    return { id, source: id + '.md' };
  });
  const config = { version: 1, renderer: 'portable', book: {
    version: 1, title: 'One hundred connected topics', authors: ['Zellij'],
    chapters: [
      { id: 'first-part', title: 'First sequence', chapters: chapters.slice(0, 50) },
      { id: 'second-part', title: 'Second sequence', chapters: chapters.slice(50) },
    ],
  } };
  writeFileSync(path.join(root, 'zel.yaml'), JSON.stringify(config, null, 2) + '\n');
  return config;
}
