import { hydrateRoot } from 'react-dom/client';
import { decodeNode, type Wire } from './decode';
import { setFileLocation, type FileLocation } from './location';
const payload = JSON.parse(document.getElementById('zellij-file-page')!.textContent!) as { location: FileLocation; tree: Wire; diagrams: boolean };
setFileLocation(payload.location);
(globalThis as any).__zellijDiagrams = String(payload.diagrams);
hydrateRoot(document.getElementById('zellij-file-root')!, decodeNode(payload.tree), { identifierPrefix: 'zel-file-' });
