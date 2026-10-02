// Static design previews. These are not live CauceDB screenshots.
// Source palette: https://github.com/RedYaafte/black-ember/blob/main/palette/black-ember.toml
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const out = dirname(fileURLToPath(import.meta.url));
mkdirSync(out, { recursive: true });

const variants = [
  {
    slug: '01-black-ember-classic',
    title: '01  BLACK EMBER / CLASSIC',
    description: 'Canonical palette · full-row selection · warm data grid',
    bg: '#11100f', app: '#171614', panel: '#171614', raised: '#35312c',
    alt: '#201e1b', fg: '#e8dfd0', bright: '#fff7eb', muted: '#a69a8b',
    subtle: '#a69a8b', grid: '#8b8176', line: '#625a50',
    accent: '#e2a35f', accent2: '#bd8857', good: '#9caa76',
    selected: '#625a50', selectedText: '#fff7eb', headerBg: '#35312c',
    focusMode: 'row', surfaceMode: 'flat',
  },
  {
    slug: '02-black-ember-precision',
    title: '02  BLACK EMBER / PRECISION',
    description: 'Black Ember base · clear headers · cell focus with row guide',
    bg: '#11100f', app: '#171614', panel: '#1c1a17', raised: '#35312c',
    alt: '#211f1c', fg: '#e8dfd0', bright: '#fff7eb', muted: '#a69a8b',
    subtle: '#a69a8b', grid: '#8b8176', line: '#625a50',
    accent: '#e2a35f', accent2: '#bd8857', good: '#9caa76',
    selected: '#35312c', selectedText: '#fff7eb', headerBg: '#35312c',
    focusMode: 'cell', surfaceMode: 'layered',
  },
  {
    slug: '03-cauce-ember',
    title: '03  CAUCE EMBER / NEW',
    description: 'Black Ember-inspired · deep sage canvas · amber and sage roles',
    bg: '#101411', app: '#151a17', panel: '#1b211d', raised: '#29332c',
    alt: '#202720', fg: '#e8e7dc', bright: '#fffaf0', muted: '#b5bbae',
    subtle: '#a6b3a6', grid: '#66766a', line: '#506255',
    accent: '#f1b76b', accent2: '#cf9257', good: '#a9c7b2',
    selected: '#304037', selectedText: '#fffaf0', headerBg: '#29332c',
    focusMode: 'cell', surfaceMode: 'layered',
  },
];

const e = (value) => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');
const rows = [
  ['10041', 'ACME NORTH', 'PAID', '2026-09-30 09:12', '1,280.00', 'MARA', 'NORTH'],
  ['10042', 'LUMEN LAB', 'PAID', '2026-09-30 10:45', '840.50', 'ALEX', 'WEST'],
  ['10043', 'SOLARIS', 'PENDING', '2026-09-30 11:08', '2,090.00', 'NORA', 'CENTRAL'],
  ['10044', 'ATLAS CO', 'PAID', '2026-09-30 13:21', '475.00', 'MARA', 'EAST'],
  ['10045', 'MESA TECH', 'PENDING', '2026-10-01 08:30', '3,405.75', 'ELI', 'NORTH'],
  ['10046', 'NOVA GROUP', 'PAID', '2026-10-01 09:54', '1,120.00', 'ALEX', 'SOUTH'],
  ['10047', 'IRIS WORKS', 'DRAFT', '2026-10-01 10:06', '925.20', 'NORA', 'WEST'],
  ['10048', 'CEDAR LTD', 'PAID', '2026-10-01 11:41', '6,020.00', 'ELI', 'CENTRAL'],
  ['10049', 'EMBER INC', 'PENDING', '2026-10-01 12:08', '780.30', 'MARA', 'EAST'],
];
const columns = [
  ['#', 42], ['ORDER_ID', 121], ['CUSTOMER', 200], ['STATUS', 140],
  ['CREATED_AT', 270], ['TOTAL_USD', 160], ['OWNER', 140], ['REGION', 157],
];

function render(t) {
  const s = [];
  const R = (x, y, w, h, fill, stroke = 'none', sw = 1) => s.push(`<rect x="${x}" y="${y}" width="${w}" height="${h}" fill="${fill}" stroke="${stroke}" stroke-width="${sw}"/>`);
  const L = (x1, y1, x2, y2, color, width = 1) => s.push(`<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${color}" stroke-width="${width}"/>`);
  const T = (x, y, value, color = t.fg, size = 15, weight = 400, anchor = 'start') => s.push(`<text x="${x}" y="${y}" fill="${color}" font-family="DejaVu Sans Mono, monospace" font-size="${size}" font-weight="${weight}" text-anchor="${anchor}" xml:space="preserve">${e(value)}</text>`);
  const chip = (x, y, w, label, fill, color) => { R(x, y, w, 25, fill); T(x + 10, y + 18, label, color, 13, 700); };

  s.push(`<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="1000" viewBox="0 0 1600 1000">`);
  R(0, 0, 1600, 1000, t.bg);
  T(28, 40, t.title, t.accent, 24, 700);
  T(30, 65, t.description, t.fg, 15);

  // Application shell and top chrome.
  R(26, 86, 1548, 836, t.app, t.grid);
  R(27, 87, 1546, 47, t.raised);
  T(45, 117, 'CAUCEDB', t.bright, 20, 700);
  T(176, 116, '/', t.muted, 18);
  T(202, 116, 'DEV_ORACLE', t.accent, 16, 700);
  chip(1174, 98, 109, '● ONLINE', t.panel, t.good);
  chip(1291, 98, 256, 'TRANSACTION  •  CLEAN', t.panel, t.fg);
  L(27, 134, 1573, 134, t.grid);

  // Connections and schema browser.
  R(27, 135, 292, 741, t.panel);
  L(319, 135, 319, 876, t.grid);
  T(46, 164, '01  CONNECTIONS', t.muted, 14, 700);
  R(39, 179, 267, 36, t.raised);
  R(39, 179, 4, 36, t.accent);
  T(55, 203, '●  DEV_ORACLE', t.bright, 15, 700);
  T(55, 235, 'Oracle 21c  ·  dev-db.internal', t.muted, 12);
  L(39, 253, 306, 253, t.line);
  T(46, 282, '02  SCHEMA  /  APP_DEV', t.muted, 14, 700);
  const objects = [
    ['▾', 'TABLES', '24'], [' ', 'ORDERS', ''], [' ', 'CUSTOMERS', ''],
    [' ', 'ORDER_ITEMS', ''], [' ', 'PAYMENTS', ''], ['▸', 'VIEWS', '08'],
    ['▸', 'FUNCTIONS', '13'], ['▸', 'PACKAGES', '05'],
  ];
  let oy = 315;
  for (const [icon, name, count] of objects) {
    const isCurrent = name === 'ORDERS';
    if (isCurrent) { R(39, oy - 22, 267, 30, t.raised); R(39, oy - 22, 4, 30, t.accent); }
    T(icon === ' ' ? 76 : 54, oy, icon === ' ' ? '▦' : icon, isCurrent ? t.accent : t.muted, 15);
    T(icon === ' ' ? 102 : 79, oy, name, isCurrent ? t.bright : t.fg, 15, isCurrent ? 700 : 400);
    if (count) T(288, oy, count, t.muted, 13, 400, 'end');
    oy += 40;
  }
  L(39, 636, 306, 636, t.line);
  T(46, 666, 'OBJECT DETAILS', t.muted, 14, 700);
  T(46, 697, 'ORDERS', t.bright, 16, 700);
  T(46, 725, 'Table · 24 columns', t.fg, 14);
  T(46, 750, 'Last inspected: 12:08', t.muted, 13);
  T(46, 839, 'Enter  inspect object', t.muted, 13);

  // SQL workspace.
  R(320, 135, 1253, 38, t.app);
  R(331, 141, 199, 31, t.raised);
  R(331, 170, 199, 3, t.accent);
  T(346, 162, 'orders.sql  ●', t.bright, 14, 700);
  T(548, 162, 'scratch.sql', t.muted, 14);
  T(1547, 162, 'Ctrl+←/→  tabs', t.muted, 12, 400, 'end');
  L(320, 173, 1573, 173, t.line);
  T(341, 203, '03  SQL EDITOR', t.muted, 14, 700);
  T(1544, 203, 'F5  RUN SELECTION     F6  RUN FILE', t.accent, 13, 700, 'end');
  R(331, 215, 1230, 265, t.surfaceMode === 'flat' ? t.app : t.panel, t.line);
  R(332, 216, 44, 263, t.raised);
  const sql = [
    [['SELECT', t.accent], [' order_id, customer, status,', t.fg]],
    [['       created_at, total_usd, owner, region', t.fg]],
    [['FROM', t.accent], ['   app_dev.orders', t.fg]],
    [['WHERE', t.accent], ['  created_at >= ', t.fg], ["DATE '2026-09-30'", t.good]],
    [['ORDER BY', t.accent], [' created_at DESC;', t.fg]],
  ];
  sql.forEach((parts, i) => {
    const y = 245 + i * 34;
    T(365, y, String(i + 1), t.muted, 14, 400, 'end');
    let x = 392;
    for (const [part, color] of parts) {
      T(x, y, part, color, 16);
      x += part.length * 9.64;
    }
  });
  T(392, 455, '-- Preview uses synthetic data', t.muted, 13);

  // Query result with explicit boundaries in both axes.
  T(340, 520, '04  RESULTS', t.bright, 15, 700);
  chip(485, 499, 101, '9 ROWS', t.raised, t.accent);
  T(1545, 520, '45 ms   ·   8 columns   ·   sample data', t.muted, 13, 400, 'end');
  const gx = 331, gy = 539, gh = 315;
  const gw = columns.reduce((sum, [, w]) => sum + w, 0);
  const rowH = 29, headH = 42;
  R(gx, gy, gw, gh, t.app, t.grid);
  R(gx + 1, gy + 1, gw - 2, headH - 1, t.headerBg);
  let cy = gy + headH;
  rows.forEach((_, i) => {
    const fill = i === 2 ? t.selected : (i % 2 ? t.alt : t.panel);
    R(gx + 1, cy, gw - 2, rowH, fill);
    if (i === 2) R(gx + 1, cy, 4, rowH, t.accent);
    cy += rowH;
  });
  let cx = gx;
  columns.forEach(([label, width], i) => {
    T(cx + 12, gy + 27, label, t.bright, 13, 700);
    if (i) L(cx, gy, cx, gy + gh, t.grid);
    cx += width;
  });
  L(gx, gy + headH, gx + gw, gy + headH, t.grid, 1.5);
  rows.forEach((row, i) => {
    const y = gy + headH + i * rowH;
    const values = [String(i + 1).padStart(2, '0'), ...row];
    let x = gx;
    values.forEach((value, j) => {
      const activeCell = i === 2 && j === 2 && t.focusMode === 'cell';
      const status = j === 3;
      const color = activeCell ? t.app : i === 2 ? t.selectedText : status && value === 'PAID' ? t.good : t.fg;
      T(x + 12, y + 20, value, color, 13, activeCell ? 700 : 400);
      x += columns[j][1];
    });
    L(gx, y + rowH, gx + gw, y + rowH, t.grid);
  });
  if (t.focusMode === 'cell') {
    const cx2 = gx + columns[0][1] + columns[1][1];
    const cy2 = gy + headH + 2 * rowH;
    R(cx2 + 1, cy2 + 1, columns[2][1] - 2, rowH - 2, t.accent);
    T(cx2 + 12, cy2 + 20, rows[2][1], t.app, 13, 700);
    R(gx + 1, cy2 + 1, 4, rowH - 2, t.accent);
  }
  // The focused item is indicated by position, fill and text contrast, not hue alone.
  T(340, 872, 'ROW 03 / 09   ·   CUSTOMER   ·   SQL executed successfully', t.fg, 13);
  R(27, 877, 1546, 44, t.raised);
  T(45, 905, '↑↓ ROWS     ←→ COLUMNS     ENTER CELL     [ ] RESULTS', t.bright, 13, 700);
  T(1546, 905, 'F7 COMMIT     F9 ROLLBACK     F1 HELP', t.accent, 13, 700, 'end');

  // Palette key keeps the proposals comparable and implementable.
  T(30, 957, 'TOKENS', t.muted, 13, 700);
  const chips = [
    ['Canvas', t.app], ['Surface', t.raised], ['Text', t.fg],
    ['Grid', t.grid], ['Focus', t.accent], ['Success', t.good],
  ];
  let px = 110;
  for (const [name, value] of chips) {
    R(px, 939, 18, 18, value, t.grid);
    T(px + 27, 954, `${name} ${value}`, t.fg, 12);
    px += 244;
  }
  s.push('</svg>');
  writeFileSync(join(out, `${t.slug}.svg`), s.join('\n'));
}

variants.forEach(render);
