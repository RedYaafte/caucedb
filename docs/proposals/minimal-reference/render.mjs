// Static, synthetic CauceDB previews inspired by the supplied minimal TUI image.
// This is code-native vector artwork, not a capture of the running application.
import { writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const dest = dirname(fileURLToPath(import.meta.url));
const reference = {
  bg: '#1b1e1f', rail: '#1d2021', surface: '#25292a',
  text: '#c8c0ae', bright: '#f1e9d8', muted: '#a99f90',
  line: '#77746c', softLine: '#555650', accent: '#ed660c',
  good: '#a7bd8d', darkInk: '#1b1e1f', selected: '#35312d',
};
const ember = {
  bg: '#171614', rail: '#171614', surface: '#35312c',
  text: '#e8dfd0', bright: '#fff7eb', muted: '#a69a8b',
  line: '#8b8176', softLine: '#625a50', accent: '#e2a35f',
  good: '#9caa76', darkInk: '#11100f', selected: '#35312c',
};
const variants = [
  {
    file: '01-reference-workspace', title: '01  REFERENCE / WORKSPACE',
    subtitle: 'Image-led · one work area · quiet side navigation · orange active tab',
    theme: reference, layout: 'single', editorHeight: 286, selected: 'row',
  },
  {
    file: '02-reference-data-first', title: '02  REFERENCE / DATA FIRST',
    subtitle: 'Image-led · compact editor · generous results · one decisive highlight',
    theme: reference, layout: 'data', editorHeight: 205, selected: 'row',
  },
  {
    file: '03-ember-minimal-precision', title: '03  BLACK EMBER / MINIMAL PRECISION',
    subtitle: 'Reference shell + Black Ember palette + precise cell navigation',
    theme: ember, layout: 'hybrid', editorHeight: 249, selected: 'cell',
  },
];
const escapeXml = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');
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
const cols = [
  ['#', 46], ['ORDER_ID', 121], ['CUSTOMER', 191], ['STATUS', 130],
  ['CREATED_AT', 244], ['TOTAL_USD', 176], ['OWNER', 150], ['REGION', 172],
];

function render(v) {
  const c = v.theme, svg = [];
  const r = (x,y,w,h,fill,stroke='none',sw=1) => svg.push(`<rect x="${x}" y="${y}" width="${w}" height="${h}" fill="${fill}" stroke="${stroke}" stroke-width="${sw}"/>`);
  const l = (x1,y1,x2,y2,color,width=1) => svg.push(`<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${color}" stroke-width="${width}"/>`);
  const t = (x,y,value,color=c.text,size=15,weight=400,anchor='start') => svg.push(`<text x="${x}" y="${y}" fill="${color}" font-family="DejaVu Sans Mono,monospace" font-size="${size}" font-weight="${weight}" text-anchor="${anchor}" xml:space="preserve">${escapeXml(value)}</text>`);
  const cap = (x,y,value,color=c.muted) => t(x,y,value,color,13,700);
  const tabY=156, contentY=204, contentX=316, contentW=1248;
  const resultY=contentY+v.editorHeight+21, resultH=873-resultY;
  svg.push('<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="1000" viewBox="0 0 1600 1000">');
  r(0,0,1600,1000,c.bg);
  t(25,39,v.title,c.accent,22,700);
  t(26,65,v.subtitle,c.text,14);
  // A single terminal surface, never a fabricated desktop window.
  r(18,84,1564,859,c.bg,c.softLine);
  r(19,85,266,856,c.rail);
  l(285,110,285,900,c.softLine);
  t(39,117,'◇ CAUCEDB',c.bright,16,700);
  t(39,154,'Connections',c.muted,13,700);
  r(34,169,237,34,c.selected);
  t(44,192,'● DEV_ORACLE',c.bright,14,700);
  t(44,224,'Oracle 21c  ·  connected',c.muted,12);
  l(35,246,270,246,c.softLine);
  cap(39,275,'SCHEMA  /  APP_DEV');
  const nav = [
    ['▾','Tables','24'],[' ','ORDERS',''],[' ','CUSTOMERS',''],
    [' ','ORDER_ITEMS',''],[' ','PAYMENTS',''],['▸','Views','08'],
    ['▸','Functions','13'],['▸','Packages','05'],
  ];
  nav.forEach(([icon,name,count],i)=>{
    const y=308+i*39;
    if(name==='ORDERS') r(35,y-22,236,31,c.selected);
    t(44,y,icon,c.muted,14);
    t(icon===' '?67:61,y,name,name==='ORDERS'?c.bright:c.text,14,name==='ORDERS'?700:400);
    if(count) t(263,y,count,c.muted,12,400,'end');
  });
  l(35,625,270,625,c.softLine);
  cap(39,655,'FILES');
  t(44,688,'orders.sql  *',c.bright,14,700);
  t(44,719,'scratch.sql',c.muted,14);
  t(39,829,'/  Filter objects',c.muted,12);
  t(39,853,'F3 Switch schema',c.muted,12);
  t(39,877,'Enter Inspect',c.muted,12);

  // One tab strip and one main frame copy the reference's information economy.
  r(286,85,1295,56,c.bg);
  t(313,118,'DEV_ORACLE',c.bright,15,700);
  t(469,118,'/  APP_DEV',c.muted,14);
  t(1554,118,'AUTOCOMMIT OFF',c.muted,13,400,'end');
  l(286,141,1581,141,c.softLine);
  r(305,tabY,167,29,c.accent);
  t(318,tabY+21,'orders.sql  *',c.darkInk,14,700);
  t(491,tabY+21,'scratch.sql',c.muted,14);
  t(1554,tabY+21,'Ctrl+←/→ tabs',c.muted,12,400,'end');
  if(v.layout==='single' || v.layout==='hybrid') r(305,191,1258,683,c.bg,c.softLine);
  if(v.layout==='data') {
    l(305,191,1563,191,c.softLine);
    r(305,resultY-12,1258,resultH+12,c.bg,c.softLine);
  }
  cap(327,223,'SQL EDITOR',v.layout==='hybrid'?c.accent:c.bright);
  t(1543,223,'F5 Statement   F6 Script   Ctrl+S Save',c.accent,12,700,'end');
  const sql = [
    [['SELECT',c.accent],[' order_id, customer, status,',c.text]],
    [['       created_at, total_usd, owner, region',c.text]],
    [['FROM',c.accent],['   app_dev.orders',c.text]],
    [['WHERE',c.accent],['  created_at >= ',c.text],["DATE '2026-09-30'",c.good]],
    [['ORDER BY',c.accent],[' created_at DESC;',c.text]],
  ];
  sql.forEach((parts,i)=>{
    const y=262+i*30;
    t(340,y,String(i+1),c.muted,13,400,'end');
    let x=371;
    parts.forEach(([segment,color])=>{t(x,y,segment,color,15);x+=segment.length*9.04;});
  });
  if(v.layout==='data') {
    t(371,404,'-- Synthetic preview',c.muted,12);
  } else {
    t(371,resultY-37,'-- Synthetic preview',c.muted,12);
  }
  l(305,resultY-1,1563,resultY-1,c.softLine);
  cap(327,resultY+27,'RESULTS',v.layout==='hybrid'?c.accent:c.bright);
  t(448,resultY+27,'9 rows  ·  8 columns  ·  45 ms',c.muted,12);
  t(1542,resultY+27,'[ ] result sets    Enter cell',c.muted,12,400,'end');

  const gridX=316, gridY=resultY+42, headerH=32;
  const rowH=v.layout==='data'?35:v.layout==='hybrid'?29:26;
  const gridW=cols.reduce((sum,[,w])=>sum+w,0);
  const gridH=headerH+rows.length*rowH;
  r(gridX,gridY,gridW,gridH,c.bg,c.line);
  r(gridX+1,gridY+1,gridW-2,headerH-1,c.surface);
  rows.forEach((row,i)=>{
    const y=gridY+headerH+i*rowH;
    const isSelected=i===2;
    const fill=isSelected?c.selected:i%2===1?c.rail:c.bg;
    r(gridX+1,y,gridW-2,rowH,fill);
    if(v.layout==='data' && isSelected) r(gridX+1,y,gridW-2,rowH,c.accent);
    let x=gridX;
    [String(i+1).padStart(2,'0'),...row].forEach((value,j)=>{
      const onAccent=v.layout==='data' && isSelected;
      let color=onAccent?c.darkInk:isSelected?c.bright:c.text;
      if(!onAccent && j===3 && value==='PAID' && !isSelected) color=c.good;
      t(x+10,y+Math.floor(rowH/2)+5,value,color,12,onAccent?700:400);
      x+=cols[j][1];
    });
  });
  let x=gridX;
  cols.forEach(([label,w],i)=>{
    t(x+10,gridY+22,label,c.bright,12,700);
    if(i) l(x,gridY,x,gridY+gridH,c.line);
    x+=w;
  });
  l(gridX,gridY+headerH,gridX+gridW,gridY+headerH,c.line,1.5);
  rows.forEach((_,i)=>l(gridX,gridY+headerH+(i+1)*rowH,gridX+gridW,gridY+headerH+(i+1)*rowH,c.line));
  if(v.selected==='cell') {
    const cellX=gridX+cols[0][1]+cols[1][1], cellY=gridY+headerH+2*rowH;
    r(cellX+1,cellY+1,cols[2][1]-2,rowH-2,c.accent);
    t(cellX+10,cellY+Math.floor(rowH/2)+5,'SOLARIS',c.darkInk,12,700);
  }
  t(316,867,'row 3/9   ·   column 3/8',c.muted,12);
  r(19,901,1562,27,c.surface);
  t(34,920,'SQL  /  orders.sql     9 rows     45 ms',c.bright,12,700);
  t(1556,920,'F7 Commit    F9 Rollback    F1 Help',c.accent,12,700,'end');
  t(26,972,'↑↓ Rows    ←→ Columns    Enter Cell    Ctrl+O Open    Ctrl+Q Quit',c.muted,12);
  svg.push('</svg>');
  writeFileSync(join(dest,`${v.file}.svg`),svg.join('\n'));
}
variants.forEach(render);
