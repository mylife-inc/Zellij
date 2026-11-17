// Original short manuscripts shared by native and hosted print acceptance.
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
export const languages = {
  ar: { title: 'دفتر المعرفة', subtitle: 'أفكار واضحة بلغة جميلة', author: 'ليلى منصور', first: 'بداية الرحلة', second: 'أسئلة وأجوبة', part: 'أفكار مترابطة', appendix: 'مرجع صغير', contents: 'المحتويات', chapter: 'باب', text: 'تبدأ المعرفة بسؤال واضح، وتنمو بالتأمل والملاحظة. نكتب الأفكار بلغة دقيقة ونراجعها بهدوء، حتى يجد القارئ طريقه بين الأمثلة والنتائج.', note: 'هذه ملاحظة توضح الفكرة دون أن تقطع تسلسل القراءة.', detail: 'تجربة مشتركة', row: ['الفكرة', 'القيمة', 'الطاقة', 'ثابتة'], font: 'amiri' },
  fr: { title: 'Un carnet de découvertes', subtitle: 'Des idées et des liens', author: 'Camille Martin', first: 'Le début du voyage', second: 'Questions et réponses', part: 'Idées reliées', appendix: 'Petit repère', contents: 'Table des matières', chapter: 'Chapitre', text: 'La connaissance commence par une question précise. Nous écrivons avec clarté, observons les détails et relions chaque résultat à une expérience vérifiable. Une lecture attentive éclaire les idées.', note: 'Une note conserve un détail utile sans interrompre la lecture.', detail: 'Une expérience partagée', row: ['Idée', 'Valeur', 'Énergie', 'Constante'], font: 'libertinus' },
  de: { title: 'Ein Buch der Entdeckungen', subtitle: 'Gedanken und Zusammenhänge', author: 'Mara Weber', first: 'Der Anfang der Reise', second: 'Fragen und Antworten', part: 'Verbundene Gedanken', appendix: 'Kleine Übersicht', contents: 'Inhaltsverzeichnis', chapter: 'Kapitel', text: 'Wissen beginnt mit einer klaren Frage. Wir schreiben sorgfältig, beobachten die Einzelheiten und verbinden jede Erklärung mit einem überprüfbaren Beispiel. Das Lesen eröffnet neue Möglichkeiten.', note: 'Diese Fußnote bewahrt eine hilfreiche Ergänzung.', detail: 'Ein gemeinsamer Versuch', row: ['Gedanke', 'Wert', 'Energie', 'Konstant'] },
  es: { title: 'Un cuaderno de descubrimientos', subtitle: 'Ideas y conexiones', author: 'Lucía Moreno', first: 'El comienzo del viaje', second: 'Preguntas y respuestas', part: 'Ideas relacionadas', appendix: 'Una referencia breve', contents: 'Índice', chapter: 'Capítulo', text: 'El conocimiento comienza con una pregunta clara. Escribimos con cuidado, observamos los detalles y relacionamos cada conclusión con un ejemplo verificable. La lectura nos permite descubrir nuevas ideas.', note: 'Esta nota conserva un detalle útil para la lectura.', detail: 'Un experimento compartido', row: ['Idea', 'Valor', 'Energía', 'Constante'] },
};
export function writeLanguageBook(root, language = 'ar', edition = 'technical') {
  const x = languages[language];
  const config = { version: 1, renderer: 'portable', markdown: { math: true }, pdf: { enabled: true, ...(x.font ? {style: {'body-font': x.font, 'title-font': x.font}} : {}) }, book: {version: 1, title: x.title, subtitle: x.subtitle, authors: [x.author], language, direction: language === 'ar' ? 'rtl' : 'ltr', edition, chapters: [{id:'part',title:x.part,chapters:[{id:'first',source:'chapters/01-first.md'},{id:'second',source:'chapters/02-second.md'}]}], appendices:[{id:'reference',source:'reference.md'}]}};
  mkdirSync(path.join(root,'content/chapters'), {recursive:true});
  writeFileSync(path.join(root,'zel.yaml'), JSON.stringify(config,null,2)+'\n');
  const [a,b,c,d] = x.row;
  writeFileSync(path.join(root,'content/chapters/01-first.md'), `# ${x.first}\n\n${x.text}\n\n**${x.detail}** — *${x.note}*\n\n${x.text}[^n]\n\n[^n]: ${x.note}\n\n## ${x.detail}\n\n${x.text}\n\n| ${a} | ${b} |\n| --- | --- |\n| ${c} | ${d} |\n\n1. ${x.text}\n2. ${x.note}\n\nEnglish names: Zellij and Shebka, version 10.\n\n\`zel build --pdf\`\n\n\`\`\`rust\nlet energy = mass * speed.powi(2);\n\`\`\`\n\n$E = mc^2$\n\n$$\n\\frac{a+b}{2} = x\n$$\n\n[${x.second}](02-second.md).\n`);
  writeFileSync(path.join(root,'content/chapters/02-second.md'), `# ${x.second}\n\n${Array(7).fill(x.text).join('\n\n')}\n\n[${x.first}](01-first.md) · [${x.appendix}](../reference.md)\n`);
  writeFileSync(path.join(root,'content/reference.md'), `# ${x.appendix}\n\n${x.note}\n\n[${x.first}](chapters/01-first.md)\n`);
  return config;
}

// Poppler inserts word-like gaps at some non-joining Arabic glyphs. Preserve
// character order while ignoring layout whitespace and Unicode presentation forms.
export function normalizedPrintText(text) {
  return text.normalize('NFKC').replace(/[\u202a-\u202e\u2066-\u2069\s]/g, '');
}
export function requiredPrintTerms(language) {
  const x = languages[language];
  return [x.title, x.first, x.second, x.contents, x.chapter, x.detail, x.note, 'Zellij', 'Shebka'];
}
