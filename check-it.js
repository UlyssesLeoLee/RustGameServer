const fs = require('fs');
const path = require('path');
const crates = ['player-service','economy-service','match-service','social-service','admin-service','gm-backend','rgs-certgen'];
for (const c of crates) {
    const testsDir = `D:/RustGameServer/crates/${c}/tests`;
    if (!fs.existsSync(testsDir)) {
        console.log(`${c.padEnd(20)} : NO tests/ dir`);
        continue;
    }
    const files = fs.readdirSync(testsDir).filter(f => f.startsWith('integration_') || f.startsWith('it_'));
    for (const f of files) {
        const o = fs.readFileSync(`${testsDir}/${f}`, 'utf8');
        const fns = (o.match(/fn\s+\w+/g) || []).length;
        console.log(`${c.padEnd(20)} : ${f.padEnd(35)} ${fns} fn`);
    }
}
