// Static comparisons of today's CauceDB layout with Black Ember / Classic.
// Palette source: https://github.com/RedYaafte/black-ember/blob/main/palette/black-ember.toml
import { writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const output = dirname(fileURLToPath(import.meta.url));
const base = {
  deep: '#11100f', canvas: '#171614', surface: '#35312c',
  overlay: '#625a50', text: '#e8dfd0', bright: '#fff7eb',
  muted: '#a69a8b', grid: '#8b8176', gold: '#e2a35f',
  amber: '#bd8857', green: '#9caa76', rowTint: '#201e1b',
};
const variants = [
  {
    name: '01-terminal-classic', label: '01  TERMINAL CLASSIC',
    description: 'Closest to the current TUI · corrected text contrast · full-row selection',
    canvas: base.canvas, sidebar: base.canvas, editor: base.canvas,
    border: base.overlay, focusBorder: base.gold, grid: base.grid,
    selected: base.overlay, selectedText: base.bright,
    activeCell: 'none', tab: base.surface, rowAlt: base.canvas,
    heading: base.gold, muted: base.muted, syntax: base.gold,
    footer: base.surface, footerText: base.text,
  },
  {
    name: '02-amber-cursor', label: '02  AMBER CURSOR',
    description: 'Current pane geometry · Black Ember surfaces · unmistakable cell focus',
    canvas: base.canvas, sidebar: base.canvas, editor: base.canvas,
    border: base.overlay, focusBorder: base.gold, grid: base.grid,
    selected: base.surface, selectedText: base.bright,
    activeCell: 'fill', tab: base.surface, rowAlt: base.rowTint,
    heading: base.gold, muted: base.muted, syntax: base.gold,
    footer: base.surface, footerText: base.bright,
  },
  {
    name: '03-quiet-grid', label: '03  QUIET GRID',
    description: 'Current layout · deeper workspace · precise outlines and calmer accents',
    canvas: base.deep, sidebar: base.canvas, editor: base.deep,
    border: base.overlay, focusBorder: base.amber, grid: base.grid,
    selected: base.surface, selectedText: base.bright,
    activeCell: 'outline', tab: base.surface, rowAlt: base.canvas,
    heading: base.amber, muted: base.muted, syntax: base.amber,
    footer: base.canvas, footerText: base.text,
  },
];
const esc = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');
const data = [
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
  ['#', 42], ['ORDER_ID', 120], ['CUSTOMER', 181], ['STATUS', 128],
  ['CREATED_AT', 239], ['TOTAL_USD', 169], ['OWNER', 150], ['REGION', 238],
];

function render(v) {
  const out = [];
  const rect = (x,y,w,h,fill,stroke='none',sw=1) => out.push(`<rect x="${x}" y="${y}" width="${w}" height="${h}" fill="${fill}" stroke="${stroke}" stroke-width="${sw}"/>`);
  const line = (x1,y1,x2,y2,color,width=1) => out.push(`<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${color}" stroke-width="${width}"/>`);
  const text = (x,y,value,color=base.text,size=15,weight=400,anchor='start') => out.push(`<text x="${x}" y="${y}" fill="${color}" font-family="DejaVu Sans Mono,monospace" font-size="${size}" font-weight="${weight}" text-anchor="${anchor}" xml:space="preserve">${esc(value)}</text>`);
  const pane = (x,y,w,h,label,focused=false) => {
    rect(x,y,w,h,v.canvas,focused?v.focusBorder:v.border);
    rect(x+11,y-9,Math.min(w-22,label.length*9.2+22),18,v.canvas);
    text(x+18,y+5,label,focused?v.focusBorder:base.muted,15,focused?700:400);
  };
  out.push('<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="1000" viewBox="0 0 1600 1000">');
  rect(0,0,1600,1000,v.canvas);
  text(22,36,v.label,base.gold,22,700);
  text(24,63,v.description,base.text,14);
  rect(18,78,1564,862,v.canvas,v.border);

  // Current application hierarchy: single-line masthead, 2 left panes,
  // broad SQL editor, results directly beneath it, status and key footer.
  text(38,114,'CAUCEDB /',v.heading,16,700);
  text(156,114,'DEV_ORACLE',base.text,15);
  text(311,114,'TRANSACTION · CHECK / COMMIT / ROLLBACK',v.heading,15);
  line(18,132,1582,132,v.border);
  pane(30,157,249,133,'1  Connections');
  rect(42,183,225,30,v.selected);
  text(50,205,'› ● DEV_ORACLE',v.selectedText,15,700);
  text(49,241,'Oracle 21c · dev-db.internal',base.muted,12);
  pane(30,309,249,543,'2  APP_DEV');
  const objects = [
    ['ORDERS','table'],['CUSTOMERS','table'],['ORDER_ITEMS','table'],
    ['PAYMENTS','table'],['ACTIVE_ORDERS','view'],['F_TOTAL','function'],
    ['F_TAX','function'],['PKG_REPORTING','package'],
  ];
  objects.forEach(([name,kind],i) => {
    const y=341+i*59;
    if(i===0) rect(41,y-21,226,52,v.selected);
    text(49,y,`${i===0?'›':' '} ${name}`,i===0?v.selectedText:base.text,14,i===0?700:400);
    text(63,y+22,kind,i===0?v.selectedText:base.muted,13);
  });
  text(43,835,'/ filter    F3 schema',base.muted,13);

  rect(290,140,1279,31,v.canvas);
  rect(300,143,167,27,v.tab);
  text(311,163,'1 orders.sql *',base.bright,14,700);
  text(484,163,'2 scratch.sql',base.muted,14);
  pane(290,184,1279,340,'3  SQL editor   ·   Ctrl+←/→ tabs',true);
  rect(291,185,1277,338,v.editor);
  rect(301,213,43,300,v.canvas);
  const editorLines = [
    [['SELECT',v.syntax],[' order_id, customer, status,',base.text]],
    [['       created_at, total_usd, owner, region',base.text]],
    [['FROM',v.syntax],['   app_dev.orders',base.text]],
    [['WHERE',v.syntax],['  created_at >= ',base.text],["DATE '2026-09-30'",base.green]],
    [['ORDER BY',v.syntax],[' created_at DESC;',base.text]],
  ];
  editorLines.forEach((parts,i) => {
    const y=236+i*28;
    text(332,y,String(i+1),base.muted,14,400,'end');
    let x=361;
    parts.forEach(([part,color])=>{text(x,y,part,color,15);x+=part.length*9.04;});
  });
  text(361,480,'-- Sample query · no production data',base.muted,13);
  // Redraw the pane border so its title and active boundary stay visible.
  rect(290,184,1279,340,'none',v.focusBorder);
  rect(301,175,319,18,v.canvas);
  text(308,189,'3  SQL editor   ·   Ctrl+←/→ tabs',v.focusBorder,15,700);

  pane(290,550,1279,302,'4  Results  1/1');
  const gx=301, gy=568, widths=columns.map(c=>c[1]);
  const gw=widths.reduce((sum,w)=>sum+w,0), headerH=33, rowH=24;
  rect(gx,gy,gw,headerH+data.length*rowH,v.canvas,v.grid);
  rect(gx+1,gy+1,gw-2,headerH-1,base.surface);
  data.forEach((row,i)=>{
    const y=gy+headerH+i*rowH;
    rect(gx+1,y,gw-2,rowH,i===2?v.selected:i%2?v.rowAlt:v.canvas);
    let x=gx;
    const cells=[String(i+1).padStart(2,'0'),...row];
    cells.forEach((cell,j)=>{
      const active=i===2 && j===2;
      const color=i===2?v.selectedText:(j===3 && cell==='PAID'?base.green:base.text);
      text(x+10,y+17,cell,color,12,active?700:400);
      x+=widths[j];
    });
  });
  let x=gx;
  columns.forEach(([label,w],i)=>{
    text(x+10,gy+23,label,base.bright,12,700);
    if(i) line(x,gy,x,gy+headerH+data.length*rowH,v.grid);
    x+=w;
  });
  line(gx,gy+headerH,gx+gw,gy+headerH,v.grid,1.5);
  data.forEach((_,i)=>line(gx,gy+headerH+(i+1)*rowH,gx+gw,gy+headerH+(i+1)*rowH,v.grid));
  if(v.activeCell!=='none') {
    const cellX=gx+widths[0]+widths[1], cellY=gy+headerH+2*rowH;
    if(v.activeCell==='fill') {
      rect(cellX+1,cellY+1,widths[2]-2,rowH-2,base.gold);
      text(cellX+10,cellY+17,'SOLARIS',base.deep,12,700);
    } else {
      rect(cellX+1,cellY+1,widths[2]-2,rowH-2,'none',base.gold,2);
    }
  }
  text(302,845,'row 3/9 · column 3/8',v.heading,13);
  text(40,885,'Statement 1 · 9 rows · 45 ms',base.text,14);
  rect(19,905,1562,34,v.footer);
  text(37,928,'↑↓ Rows    ←→ Columns    Enter Cell    [ ] Results',v.footerText,13,700);
  text(1558,928,'F5 Run    F6 Script    F7 Commit    F9 Rollback    F1 Help',v.heading,13,700,'end');
  text(24,971,`Canvas ${v.canvas}   ·   Text ${base.text}   ·   Grid ${v.grid}   ·   Focus ${v.focusBorder}`,base.muted,12);
  out.push('</svg>');
  writeFileSync(join(output,`${v.name}.svg`),out.join('\n'));
}
variants.forEach(render);
